//! Providers answer questions about one kind of [`Source`](crate::refs::Source).
//!
//! Each capability is its own trait, so a target implements only what it can
//! answer, and a [`Registry`] maps a [`SourceKind`] to the implementation.

pub mod git;
pub mod github;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::refs::{ItemRef, Source, SourceKind};

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

/// A branch or tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRef {
    /// A branch name.
    Branch(String),
    /// A tag name.
    Tag(String),
}

impl GitRef {
    /// The branch or tag name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Branch(name) | Self::Tag(name) => name,
        }
    }
}

/// Answers what a repository's history holds.
#[async_trait]
pub trait CodeHost: Send + Sync {
    /// The merge commit of a merged PR or MR; `None` when it has none.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the host cannot say.
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError>;

    /// Whether `target` contains `commit` in its history.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the host cannot say, or the ref does not exist.
    async fn contains(
        &self,
        source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<bool, CheckError>;

    /// Tag names starting with `prefix`, in any order.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the host cannot say.
    async fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError>;
}

/// The providers a run may use, keyed by source kind.
#[derive(Clone, Default)]
pub struct Registry {
    trackers: HashMap<SourceKind, Arc<dyn Tracker>>,
    code_hosts: HashMap<SourceKind, Arc<dyn CodeHost>>,
}

impl Registry {
    /// Registers the tracker for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_tracker(mut self, kind: SourceKind, tracker: Arc<dyn Tracker>) -> Self {
        self.trackers.insert(kind, tracker);
        self
    }

    /// Registers the code host for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_code_host(mut self, kind: SourceKind, host: Arc<dyn CodeHost>) -> Self {
        self.code_hosts.insert(kind, host);
        self
    }

    /// The code host for a source kind.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered.
    pub fn code_host(&self, kind: SourceKind) -> Result<&dyn CodeHost, CheckError> {
        self.code_hosts
            .get(&kind)
            .map(AsRef::as_ref)
            .ok_or_else(|| CheckError::Unsupported(format!("no provider for {kind:?} history")))
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
