//! Providers answer questions about one kind of [`Source`](crate::refs::Source).
//!
//! Each capability is its own trait, so a target implements only what it can
//! answer, and a [`Registry`] maps a [`SourceKind`] to the implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::refs::{ItemRef, SourceKind};

/// What an issue, PR or MR is now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    /// Still open.
    Open,
    /// A PR or MR that merged.
    Merged,
    /// An issue closed as completed (fixed).
    Completed,
    /// A PR or MR closed without merging.
    ClosedUnmerged,
    /// An issue closed as not planned (wontfix).
    NotPlanned,
    /// An issue closed as a duplicate.
    Duplicate,
    /// Closed, and the tracker gives no reason.
    ClosedNoReason,
}

/// Why a check could not answer. Never read as resolved.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CheckError {
    /// The item, repo or package does not exist (or is private).
    #[error("{0}: not found")]
    NotFound(String),
    /// The token was refused, or one is needed.
    #[error("{0}: authentication failed")]
    Auth(String),
    /// The API asked us to slow down.
    #[error("{0}: rate limited")]
    RateLimited(String),
    /// Transport or server trouble.
    #[error("{0}")]
    Network(String),
    /// Nobody answers for this kind of reference.
    #[error("{0}")]
    Unsupported(String),
}

/// Reads the state of issues and PRs.
#[async_trait]
pub trait Tracker: Send + Sync {
    /// The item's current state.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the tracker cannot say.
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError>;
}

/// The providers a run may use, keyed by source kind.
#[derive(Clone, Default)]
pub struct Registry {
    trackers: HashMap<SourceKind, Arc<dyn Tracker>>,
}

impl Registry {
    /// Registers the tracker for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_tracker(mut self, kind: SourceKind, tracker: Arc<dyn Tracker>) -> Self {
        self.trackers.insert(kind, tracker);
        self
    }

    /// The tracker for a source kind.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered.
    pub fn tracker(&self, kind: SourceKind) -> Result<&dyn Tracker, CheckError> {
        self.trackers
            .get(&kind)
            .map(AsRef::as_ref)
            .ok_or_else(|| CheckError::Unsupported(format!("no provider for {kind:?} items")))
    }
}
