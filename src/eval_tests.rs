use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use super::*;
use crate::manifest::StateCondition;
use crate::provider::{CheckError, ItemState, Registry, Tracker};
use crate::refs::{ItemRef, SourceKind};

struct FakeTracker(HashMap<u64, Result<ItemState, CheckError>>);

#[async_trait]
impl Tracker for FakeTracker {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        self.0
            .get(&item.number)
            .cloned()
            .unwrap_or_else(|| Err(CheckError::NotFound(item.to_string())))
    }
}

async fn outcome_for(state: Result<ItemState, CheckError>) -> Outcome {
    let tracker = FakeTracker(HashMap::from([(1, state)]));
    let registry = Registry::default().with_tracker(SourceKind::GitHub, Arc::new(tracker));
    let item: ItemRef = "github:o/r#1".parse().expect("valid ref");
    evaluate(&Condition::State(StateCondition { item }), &registry).await
}

#[tokio::test]
async fn open_waits() {
    let outcome = outcome_for(Ok(ItemState::Open)).await;
    assert_eq!(outcome.status, Status::Waiting);
    assert_eq!(outcome.detail, "github:o/r#1 is open");
}

#[tokio::test]
async fn merged_and_completed_are_bumpable() {
    let merged = outcome_for(Ok(ItemState::Merged)).await;
    assert_eq!(merged.status, Status::Bumpable);
    assert_eq!(merged.detail, "github:o/r#1 merged");
    let completed = outcome_for(Ok(ItemState::Completed)).await;
    assert_eq!(completed.status, Status::Bumpable);
    assert_eq!(completed.detail, "github:o/r#1 closed as completed");
}

#[tokio::test]
async fn rejections_are_dead_never_bumpable() {
    for (state, detail) in [
        (ItemState::ClosedUnmerged, "github:o/r#1 closed unmerged"),
        (ItemState::NotPlanned, "github:o/r#1 closed as not planned"),
        (ItemState::Duplicate, "github:o/r#1 closed as duplicate"),
        (
            ItemState::ClosedNoReason,
            "github:o/r#1 closed without a reason; check it by hand",
        ),
    ] {
        let outcome = outcome_for(Ok(state)).await;
        assert_eq!(outcome.status, Status::Dead, "{detail}");
        assert_eq!(outcome.detail, detail);
    }
}

#[tokio::test]
async fn a_provider_error_fails() {
    let outcome = outcome_for(Err(CheckError::RateLimited("github".to_owned()))).await;
    assert_eq!(outcome.status, Status::Failed);
    assert_eq!(outcome.detail, "github: rate limited");
}

#[tokio::test]
async fn a_missing_provider_fails() {
    let item: ItemRef = "github:o/r#1".parse().expect("valid ref");
    let outcome = evaluate(
        &Condition::State(StateCondition { item }),
        &Registry::default(),
    )
    .await;
    assert_eq!(outcome.status, Status::Failed);
    assert!(outcome.detail.contains("no provider"), "{}", outcome.detail);
}
