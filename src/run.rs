//! A check run: reuse fresh results, check the rest.

use futures_util::StreamExt;
use futures_util::stream;
use jiff::{RoundMode, SignedDuration, Timestamp, TimestampRound, Unit};

use crate::cache::{Entry, StateFile};
use crate::eval::evaluate;
use crate::manifest::{Blocker, Manifest};
use crate::provider::Registry;

/// How many blockers are checked at once. Whole blockers overlap; the steps
/// inside one stay in turn, since each hands values on to the next.
const IN_FLIGHT: usize = 8;

/// The cached result for a blocker, when it may stand in for a check.
fn reusable<'a>(
    blocker: &Blocker,
    previous: Option<&'a StateFile>,
    now: Timestamp,
) -> Option<&'a Entry> {
    previous?
        .entries
        .iter()
        .find(|entry| entry.id == blocker.id && entry.spec == blocker.spec)
        .filter(|entry| entry.is_fresh(now))
}

/// Checks every blocker, reusing fresh results unless `refresh`, at most
/// [`IN_FLIGHT`] at once; the entries come back in manifest order. The
/// registry starts a provider only when a check asks for it, so a fully
/// cached run starts none and never looks up a token.
pub async fn check(
    manifest: &Manifest,
    previous: Option<&StateFile>,
    registry: &Registry,
    now: Timestamp,
    refresh: bool,
) -> Vec<Entry> {
    let second = TimestampRound::new()
        .smallest(Unit::Second)
        .mode(RoundMode::Trunc);
    let now = now.round(second).unwrap_or(now);
    let ttl = manifest.settings.ttl;
    stream::iter(&manifest.blockers)
        .map(|blocker| async move {
            if let Some(entry) = reusable(blocker, previous, now).filter(|_| !refresh) {
                return Entry {
                    note: blocker.note.clone(),
                    ..entry.clone()
                };
            }
            let outcome = evaluate(&blocker.condition, registry).await;
            fresh_entry(blocker, outcome, now, ttl)
        })
        .buffered(IN_FLIGHT)
        .collect()
        .await
}

fn fresh_entry(
    blocker: &Blocker,
    outcome: crate::outcome::Outcome,
    now: Timestamp,
    ttl: SignedDuration,
) -> Entry {
    Entry {
        id: blocker.id.clone(),
        note: blocker.note.clone(),
        status: outcome.status,
        detail: outcome.detail,
        checked_at: now,
        fresh_until: Entry::freshness(outcome.status, now, ttl),
        spec: blocker.spec.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use async_trait::async_trait;

    use super::*;
    use crate::cache;
    use crate::provider::{CheckError, ItemState, Tracker};
    use crate::refs::{ItemRef, SourceKind};

    struct Open;

    #[async_trait]
    impl Tracker for Open {
        async fn item_state(&self, _item: &ItemRef) -> Result<ItemState, CheckError> {
            Ok(ItemState::Open)
        }
    }

    const MANIFEST: &str = "\
[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\n\n\
[[blocker]]\nid = \"b\"\nnote = \"n\"\nstate = \"github:o/r#2\"\n";

    /// A registry whose GitHub tracker counts how often it starts.
    fn counting(starts: &Arc<AtomicUsize>) -> Registry {
        let starts = Arc::clone(starts);
        Registry::default().with_lazy_tracker(SourceKind::GitHub, move || {
            starts.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(Open))
        })
    }

    async fn run(previous: Option<&StateFile>, refresh: bool) -> (Vec<Entry>, usize) {
        let manifest = Manifest::parse(MANIFEST).expect("manifest");
        let starts = Arc::new(AtomicUsize::new(0));
        let entries = check(
            &manifest,
            previous,
            &counting(&starts),
            Timestamp::now(),
            refresh,
        )
        .await;
        (entries, starts.load(Ordering::SeqCst))
    }

    #[tokio::test]
    async fn an_uncached_run_starts_a_provider_once_for_all_its_blockers() {
        let (entries, starts) = run(None, false).await;
        assert_eq!(entries.len(), 2);
        assert_eq!(starts, 1);
    }

    #[tokio::test]
    async fn a_fully_cached_run_starts_no_provider() {
        let (entries, _) = run(None, false).await;
        let previous = StateFile {
            version: cache::VERSION,
            manifest: PathBuf::from("canibump.toml"),
            entries,
        };
        let (again, starts) = run(Some(&previous), false).await;
        assert_eq!(again.len(), 2);
        assert_eq!(starts, 0, "every result was fresh");
        let (_, starts) = run(Some(&previous), true).await;
        assert_eq!(starts, 1, "--refresh checks again");
    }

    const DELAY: Duration = Duration::from_secs(5);

    /// A tracker that waits `delay(number)` of virtual time before it answers,
    /// and notes the most asks it ever had waiting together.
    struct Slow {
        delay: fn(u64) -> Duration,
        waiting: AtomicUsize,
        peak: AtomicUsize,
    }

    #[async_trait]
    impl Tracker for Slow {
        async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
            let waiting = self.waiting.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(waiting, Ordering::SeqCst);
            tokio::time::sleep((self.delay)(item.number)).await;
            self.waiting.fetch_sub(1, Ordering::SeqCst);
            Ok(ItemState::Open)
        }
    }

    /// Checks `count` blockers, `b1` to `bN`, against a [`Slow`] tracker: the
    /// entries, the peak of asks waiting together, and the virtual time taken.
    async fn timed(count: usize, delay: fn(u64) -> Duration) -> (Vec<Entry>, usize, Duration) {
        let text: String = (1..=count)
            .map(|number| {
                format!("[[blocker]]\nid = \"b{number}\"\nnote = \"n\"\nstate = \"github:o/r#{number}\"\n\n")
            })
            .collect();
        let manifest = Manifest::parse(&text).expect("manifest");
        let slow = Arc::new(Slow {
            delay,
            waiting: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        });
        let tracker: Arc<dyn Tracker> = slow.clone();
        let registry = Registry::default().with_tracker(SourceKind::GitHub, tracker);
        let started = tokio::time::Instant::now();
        let entries = check(&manifest, None, &registry, Timestamp::now(), false).await;
        (entries, slow.peak.load(Ordering::SeqCst), started.elapsed())
    }

    #[tokio::test(start_paused = true)]
    async fn blockers_wait_on_their_providers_together() {
        let (entries, _, took) = timed(5, |_| DELAY).await;
        assert_eq!(entries.len(), 5);
        assert!(
            took < DELAY * 2,
            "five blockers took {took:?}, one delay is {DELAY:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn no_more_blockers_than_the_limit_run_at_once() {
        let (entries, peak, _) = timed(IN_FLIGHT + 2, |_| DELAY).await;
        assert_eq!(entries.len(), IN_FLIGHT + 2);
        assert_eq!(peak, IN_FLIGHT);
    }

    #[tokio::test(start_paused = true)]
    async fn entries_keep_manifest_order_whichever_blocker_answers_first() {
        // The later the blocker, the sooner it answers.
        let (entries, _, _) = timed(4, |number| Duration::from_secs(10 - number)).await;
        let ids: Vec<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(ids, ["b1", "b2", "b3", "b4"]);
    }
}
