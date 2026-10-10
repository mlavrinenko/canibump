//! A check run: reuse fresh results, check the rest.

use jiff::{RoundMode, SignedDuration, Timestamp, TimestampRound, Unit};

use crate::cache::{Entry, StateFile};
use crate::eval::evaluate;
use crate::manifest::{Blocker, Manifest};
use crate::provider::Registry;

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

/// Checks every blocker, reusing fresh results unless `refresh`. The
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
    let mut entries = Vec::with_capacity(manifest.blockers.len());
    for blocker in &manifest.blockers {
        if let Some(entry) = reusable(blocker, previous, now).filter(|_| !refresh) {
            entries.push(Entry {
                note: blocker.note.clone(),
                ..entry.clone()
            });
            continue;
        }
        let outcome = evaluate(&blocker.condition, registry).await;
        entries.push(fresh_entry(blocker, outcome, now, manifest.settings.ttl));
    }
    entries
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
}
