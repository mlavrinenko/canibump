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
/// registry is built only when something needs a check, so a fully cached
/// run never looks up a token.
pub async fn check(
    manifest: &Manifest,
    previous: Option<&StateFile>,
    registry: impl FnOnce() -> Registry,
    now: Timestamp,
    refresh: bool,
) -> Vec<Entry> {
    let second = TimestampRound::new()
        .smallest(Unit::Second)
        .mode(RoundMode::Trunc);
    let now = now.round(second).unwrap_or(now);
    let mut registry = Some(registry);
    let mut built = None;
    let mut entries = Vec::with_capacity(manifest.blockers.len());
    for blocker in &manifest.blockers {
        if let Some(entry) = reusable(blocker, previous, now).filter(|_| !refresh) {
            entries.push(Entry {
                note: blocker.note.clone(),
                ..entry.clone()
            });
            continue;
        }
        let registry =
            built.get_or_insert_with(|| registry.take().map(|make| make()).unwrap_or_default());
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
