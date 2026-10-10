//! Turns a blocker's condition into an [`Outcome`].

use crate::manifest::{Condition, StateCondition};
use crate::outcome::{Outcome, Status};
use crate::provider::{ItemState, Registry};

/// Checks a condition through the registered providers.
///
/// A provider error becomes a [`Status::Failed`] outcome, never a resolved
/// one.
pub async fn evaluate(condition: &Condition, registry: &Registry) -> Outcome {
    match condition {
        Condition::State(state) => evaluate_state(state, registry).await,
    }
}

async fn evaluate_state(condition: &StateCondition, registry: &Registry) -> Outcome {
    let item = &condition.item;
    let state = match registry.tracker(item.source.kind()) {
        Ok(tracker) => tracker.item_state(item).await,
        Err(error) => Err(error),
    };
    match state {
        Ok(state) => {
            let (status, what) = describe(state);
            Outcome::new(status, format!("{item} {what}"))
        }
        Err(error) => Outcome::new(Status::Failed, error.to_string()),
    }
}

/// The status an item state maps to, and how to say it.
const fn describe(state: ItemState) -> (Status, &'static str) {
    match state {
        ItemState::Open => (Status::Waiting, "is open"),
        ItemState::Merged => (Status::Bumpable, "merged"),
        ItemState::Completed => (Status::Bumpable, "closed as completed"),
        ItemState::ClosedUnmerged => (Status::Dead, "closed unmerged"),
        ItemState::NotPlanned => (Status::Dead, "closed as not planned"),
        ItemState::Duplicate => (Status::Dead, "closed as duplicate"),
        ItemState::ClosedNoReason => (Status::Dead, "closed without a reason; check it by hand"),
    }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
