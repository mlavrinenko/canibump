use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

#[test]
fn a_name_with_a_negator_is_negated() {
    for name in [
        "Not Done",
        "not fixed",
        "Won't Do",
        "Never fixed",
        "Cannot Reproduce",
    ] {
        assert!(negated(name), "{name}");
    }
    for name in [
        "Done",
        "Fixed",
        "Notable",
        "Nothing",
        "Known issue",
        "Undone",
    ] {
        assert!(!negated(name), "{name}");
    }
}

struct Quiet;

#[async_trait]
impl Tracker for Quiet {
    async fn item_state(&self, _item: &ItemRef) -> Result<ItemState, CheckError> {
        Ok(ItemState::Open)
    }
}

fn broken() -> CheckError {
    CheckError::Unsupported("API URL `x`: bad".to_owned())
}

/// A tracker factory that counts its starts, then succeeds or fails.
fn counting(
    starts: &Arc<AtomicUsize>,
    outcome: Result<(), CheckError>,
) -> impl Fn() -> Result<Arc<dyn Tracker>, CheckError> + Send + Sync + 'static {
    let starts = Arc::clone(starts);
    move || {
        starts.fetch_add(1, Ordering::SeqCst);
        outcome
            .clone()
            .map(|()| Arc::new(Quiet) as Arc<dyn Tracker>)
    }
}

/// What an ask answers: the error's text, or `started`.
fn answer<T: ?Sized>(result: Result<&T, CheckError>) -> String {
    result
        .err()
        .map_or_else(|| "started".to_owned(), |error| error.to_string())
}

#[test]
fn a_provider_starts_on_its_first_ask_and_only_once() {
    let starts = Arc::new(AtomicUsize::new(0));
    let registry =
        Registry::default().with_lazy_tracker(SourceKind::GitHub, counting(&starts, Ok(())));
    assert_eq!(starts.load(Ordering::SeqCst), 0, "nobody asked yet");
    assert_eq!(answer(registry.tracker(SourceKind::GitHub)), "started");
    assert_eq!(answer(registry.tracker(SourceKind::GitHub)), "started");
    assert_eq!(starts.load(Ordering::SeqCst), 1);
}

#[test]
fn asking_for_another_kind_starts_nothing() {
    let starts = Arc::new(AtomicUsize::new(0));
    let registry =
        Registry::default().with_lazy_tracker(SourceKind::GitHub, counting(&starts, Ok(())));
    assert_eq!(
        answer(registry.tracker(SourceKind::GitLab)),
        "no provider for gitlab items"
    );
    assert_eq!(starts.load(Ordering::SeqCst), 0);
}

#[test]
fn a_start_up_error_is_kept_and_handed_to_every_ask() {
    let starts = Arc::new(AtomicUsize::new(0));
    let registry = Registry::default()
        .with_lazy_tracker(SourceKind::GitHub, counting(&starts, Err(broken())))
        .with_lazy_code_host(SourceKind::GitHub, || Err(broken()))
        .with_lazy_index(IndexKind::DepsDev, || Err(broken()));
    for _ in 0..2 {
        let github = "github: API URL `x`: bad";
        assert_eq!(answer(registry.tracker(SourceKind::GitHub)), github);
        assert_eq!(answer(registry.code_host(SourceKind::GitHub)), github);
        assert_eq!(
            answer(registry.index(IndexKind::DepsDev)),
            "deps.dev: API URL `x`: bad"
        );
    }
    assert_eq!(starts.load(Ordering::SeqCst), 1, "a failure is not retried");
}

#[test]
fn a_kind_nobody_registered_has_no_provider() {
    let registry = Registry::default().with_lazy_tracker(SourceKind::GitHub, || Err(broken()));
    assert_eq!(
        answer(registry.code_host(SourceKind::GitHub)),
        "no provider for github history"
    );
    assert_eq!(
        answer(registry.index(IndexKind::Repology)),
        "no provider for repology packages"
    );
}

#[test]
fn one_lazy_provider_can_back_two_slots_and_starts_once() {
    let starts = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&starts);
    let shared = Arc::new(Lazy::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(Quiet))
    }));
    let (first, second) = (Arc::clone(&shared), shared);
    let registry = Registry::default()
        .with_lazy_tracker(SourceKind::GitHub, move || {
            first.get().clone().map(|quiet| quiet as Arc<dyn Tracker>)
        })
        .with_lazy_tracker(SourceKind::GitLab, move || {
            second.get().clone().map(|quiet| quiet as Arc<dyn Tracker>)
        });
    assert_eq!(answer(registry.tracker(SourceKind::GitHub)), "started");
    assert_eq!(answer(registry.tracker(SourceKind::GitLab)), "started");
    assert_eq!(starts.load(Ordering::SeqCst), 1);
}
