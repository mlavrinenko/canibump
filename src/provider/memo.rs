//! Providers that answer each question once per run.
//!
//! A chain may name one PR in several steps, and several blockers may name
//! the same issue. The registry wraps every tracker and code host here, so
//! each item's state and kind (one answer) and each merge commit is fetched
//! once, however many ask and whichever provider answers. Askers that arrive while the request is
//! in flight wait for it instead of sending their own. A failure is kept
//! like an answer: a run never asks twice, and every asker hears the same
//! reason.
//!
//! `contains` and `tags` pass straight through. Their keys are a commit and
//! a ref, and a binary search rarely probes one twice.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use tokio::sync::OnceCell;

use super::{CheckError, CodeHost, GitRef, Item, ItemState, Reach, Tracker};
use crate::refs::{ItemRef, Source};

/// The answer to one question about one item, asked once.
type Slot<V> = Arc<OnceCell<Result<V, CheckError>>>;

/// One answer per item.
struct Memo<V> {
    slots: Mutex<HashMap<ItemRef, Slot<V>>>,
}

impl<V: Clone> Memo<V> {
    fn new() -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
        }
    }

    /// The answer for `item`. The first ask awaits `request`; later ones,
    /// even while it is in flight, share its outcome and drop theirs unpolled.
    async fn ask(
        &self,
        item: &ItemRef,
        request: impl Future<Output = Result<V, CheckError>>,
    ) -> Result<V, CheckError> {
        let slot = {
            // A panic elsewhere cannot leave the map half-written.
            let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
            Arc::clone(slots.entry(item.clone()).or_default())
        };
        slot.get_or_init(|| request).await.clone()
    }
}

/// A tracker that reads each item once, its state and kind together.
struct MemoTracker {
    inner: Arc<dyn Tracker>,
    items: Memo<Item>,
}

/// `inner`, asked about each item's state once.
pub(super) fn tracker(inner: Arc<dyn Tracker>) -> Arc<dyn Tracker> {
    Arc::new(MemoTracker {
        inner,
        items: Memo::new(),
    })
}

#[async_trait]
impl Tracker for MemoTracker {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        Ok(self.item(item).await?.state)
    }

    async fn item(&self, item: &ItemRef) -> Result<Item, CheckError> {
        self.items.ask(item, self.inner.item(item)).await
    }
}

/// A code host that reads each merge commit once.
struct MemoCodeHost {
    inner: Arc<dyn CodeHost>,
    commits: Memo<Option<String>>,
}

/// `inner`, asked for each merge commit once.
pub(super) fn code_host(inner: Arc<dyn CodeHost>) -> Arc<dyn CodeHost> {
    Arc::new(MemoCodeHost {
        inner,
        commits: Memo::new(),
    })
}

#[async_trait]
impl CodeHost for MemoCodeHost {
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError> {
        self.commits.ask(item, self.inner.merge_commit(item)).await
    }

    async fn contains(
        &self,
        source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<Reach, CheckError> {
        self.inner.contains(source, commit, target).await
    }

    async fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        self.inner.tags(source, prefix).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;

    use crate::provider::{
        CheckError, CodeHost, GitRef, Item, ItemState, Reach, Registry, Tracker,
    };
    use crate::refs::{ItemRef, Source, SourceKind};

    /// Answers every question, counting how often it was asked.
    #[derive(Default)]
    struct Counting {
        fail: bool,
        states: AtomicUsize,
        commits: AtomicUsize,
        probes: AtomicUsize,
        tag_lists: AtomicUsize,
    }

    #[async_trait]
    impl Tracker for Counting {
        async fn item_state(&self, _item: &ItemRef) -> Result<ItemState, CheckError> {
            self.states.fetch_add(1, Ordering::SeqCst);
            // Hand the thread over, so a second asker arrives while this
            // request is still in flight.
            tokio::task::yield_now().await;
            if self.fail {
                return Err(CheckError::Network("down".to_owned()));
            }
            Ok(ItemState::Open)
        }
    }

    #[async_trait]
    impl CodeHost for Counting {
        async fn merge_commit(&self, _item: &ItemRef) -> Result<Option<String>, CheckError> {
            self.commits.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            Ok(Some("abc1234".to_owned()))
        }

        async fn contains(
            &self,
            _source: &Source,
            _commit: &str,
            _target: &GitRef,
        ) -> Result<Reach, CheckError> {
            self.probes.fetch_add(1, Ordering::SeqCst);
            Ok(Reach::Holds)
        }

        async fn tags(&self, _source: &Source, _prefix: &str) -> Result<Vec<String>, CheckError> {
            self.tag_lists.fetch_add(1, Ordering::SeqCst);
            Ok(vec!["v1".to_owned()])
        }
    }

    fn registry(fake: Counting) -> (Registry, Arc<Counting>) {
        let fake = Arc::new(fake);
        let registry = Registry::default()
            .with_tracker(SourceKind::GitHub, fake.clone())
            .with_code_host(SourceKind::GitHub, fake.clone());
        (registry, fake)
    }

    fn item(number: u64) -> ItemRef {
        format!("github:o/r#{number}").parse().expect("valid ref")
    }

    async fn state(registry: &Registry, number: u64) -> Result<ItemState, CheckError> {
        let tracker = registry.tracker(SourceKind::GitHub)?;
        tracker.item_state(&item(number)).await
    }

    #[tokio::test]
    async fn concurrent_asks_for_one_item_share_one_request() {
        let (registry, fake) = registry(Counting::default());
        let (first, second) = tokio::join!(state(&registry, 1), state(&registry, 1));
        assert_eq!((first, second), (Ok(ItemState::Open), Ok(ItemState::Open)));
        assert_eq!(fake.states.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_repeated_ask_is_answered_from_memory_and_another_item_is_asked() {
        let (registry, fake) = registry(Counting::default());
        for _ in 0..2 {
            assert_eq!(state(&registry, 1).await, Ok(ItemState::Open));
        }
        assert_eq!(fake.states.load(Ordering::SeqCst), 1);
        assert_eq!(state(&registry, 2).await, Ok(ItemState::Open));
        assert_eq!(fake.states.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_items_state_and_kind_share_one_request() {
        let (registry, fake) = registry(Counting::default());
        let tracker = registry.tracker(SourceKind::GitHub).expect("tracker");
        let first = item(1);
        let (item, state) = tokio::join!(tracker.item(&first), state(&registry, 1));
        let open = Item {
            state: ItemState::Open,
            kind: None,
        };
        assert_eq!((item, state), (Ok(open), Ok(ItemState::Open)));
        assert_eq!(fake.states.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_failure_is_remembered_too() {
        let (registry, fake) = registry(Counting {
            fail: true,
            ..Counting::default()
        });
        let down = Err(CheckError::Network("down".to_owned()));
        assert_eq!(state(&registry, 1).await, down);
        assert_eq!(state(&registry, 1).await, down);
        assert_eq!(fake.states.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_merge_commit_is_asked_once_apart_from_the_state_of_its_item() {
        let (registry, fake) = registry(Counting::default());
        let host = registry.code_host(SourceKind::GitHub).expect("code host");
        let (commit, pr) = (Ok(Some("abc1234".to_owned())), item(1));
        assert_eq!(state(&registry, 1).await, Ok(ItemState::Open));
        let (first, second) = tokio::join!(host.merge_commit(&pr), host.merge_commit(&pr));
        assert_eq!((first, second), (commit.clone(), commit));
        assert_eq!(fake.states.load(Ordering::SeqCst), 1);
        assert_eq!(fake.commits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn history_probes_pass_straight_through() {
        let (registry, fake) = registry(Counting::default());
        let host = registry.code_host(SourceKind::GitHub).expect("code host");
        let source = item(1).source;
        for _ in 0..2 {
            let target = GitRef::Tag("v1".to_owned());
            assert_eq!(
                host.contains(&source, "abc1234", &target).await,
                Ok(Reach::Holds)
            );
            assert_eq!(host.tags(&source, "v").await, Ok(vec!["v1".to_owned()]));
        }
        assert_eq!(fake.probes.load(Ordering::SeqCst), 2);
        assert_eq!(fake.tag_lists.load(Ordering::SeqCst), 2);
    }
}
