//! Providers answer questions about one kind of [`Source`](crate::refs::Source).
//!
//! Each capability is its own trait, so a target implements only what it can
//! answer, and a [`Registry`] maps a [`SourceKind`] to the implementation.

pub mod defaults;
pub mod depsdev;
pub mod git;
pub mod gitea;
pub mod github;
pub mod gitlab;
mod http;
pub mod jira;
pub mod redmine;
pub mod repology;

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::package::{IndexKind, PackageRef};
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

/// Lists the versions a package index knows.
#[async_trait]
pub trait PackageIndex: Send + Sync {
    /// Every version of the package; empty when the repo lacks it.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the index cannot say, or knows no such project.
    async fn versions(&self, package: &PackageRef) -> Result<Vec<String>, CheckError>;
}

/// A provider started, or why it could not start.
type Started<T> = Result<Arc<T>, CheckError>;

/// The providers a run may use, keyed by source kind.
///
/// A provider that failed to start is kept as its [`CheckError`], so every
/// blocker that needs it fails with the reason; the others are not affected.
#[derive(Clone, Default)]
pub struct Registry {
    trackers: HashMap<SourceKind, Started<dyn Tracker>>,
    code_hosts: HashMap<SourceKind, Started<dyn CodeHost>>,
    indexes: HashMap<IndexKind, Started<dyn PackageIndex>>,
}

impl Registry {
    /// Registers the tracker for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_tracker(mut self, kind: SourceKind, tracker: Arc<dyn Tracker>) -> Self {
        self.trackers.insert(kind, Ok(tracker));
        self
    }

    /// Records why the tracker for a source kind could not start.
    #[must_use]
    pub fn with_tracker_error(mut self, kind: SourceKind, error: CheckError) -> Self {
        self.trackers.insert(kind, Err(error));
        self
    }

    /// Registers the code host for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_code_host(mut self, kind: SourceKind, host: Arc<dyn CodeHost>) -> Self {
        self.code_hosts.insert(kind, Ok(host));
        self
    }

    /// Records why the code host for a source kind could not start.
    #[must_use]
    pub fn with_code_host_error(mut self, kind: SourceKind, error: CheckError) -> Self {
        self.code_hosts.insert(kind, Err(error));
        self
    }

    /// Registers a package index, replacing any earlier one.
    #[must_use]
    pub fn with_index(mut self, kind: IndexKind, index: Arc<dyn PackageIndex>) -> Self {
        self.indexes.insert(kind, Ok(index));
        self
    }

    /// Records why the package index of a kind could not start.
    #[must_use]
    pub fn with_index_error(mut self, kind: IndexKind, error: CheckError) -> Self {
        self.indexes.insert(kind, Err(error));
        self
    }

    /// The tracker for a source kind.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn tracker(&self, kind: SourceKind) -> Result<&dyn Tracker, CheckError> {
        lookup(&self.trackers, kind, "items")
    }

    /// The code host for a source kind.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn code_host(&self, kind: SourceKind) -> Result<&dyn CodeHost, CheckError> {
        lookup(&self.code_hosts, kind, "history")
    }

    /// The package index of a kind.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn index(&self, kind: IndexKind) -> Result<&dyn PackageIndex, CheckError> {
        lookup(&self.indexes, kind, "packages")
    }
}

/// The provider of `kind`, or the reason there is none, naming `kind`.
fn lookup<'a, K, T>(
    slots: &'a HashMap<K, Started<T>>,
    kind: K,
    what: &str,
) -> Result<&'a T, CheckError>
where
    K: Copy + Eq + Hash + fmt::Display,
    T: ?Sized,
{
    match slots.get(&kind) {
        Some(Ok(provider)) => Ok(provider.as_ref()),
        Some(Err(error)) => Err(CheckError::Unsupported(format!("{kind}: {error}"))),
        None => Err(CheckError::Unsupported(format!(
            "no provider for {kind} {what}"
        ))),
    }
}

/// Whether a tracker's status or resolution `name` is one of `phrases`,
/// matched on whole words, case-insensitively: `Won't Fix` matches
/// `won't fix`, while `Abandoned` does not match `done`.
pub(crate) fn names_any(name: &str, phrases: &[&str]) -> bool {
    let words = |text: &str| {
        text.to_lowercase()
            .split(|char: char| !char.is_alphanumeric() && char != '\'')
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let name = words(name);
    phrases.iter().any(|phrase| {
        let phrase = words(phrase);
        !phrase.is_empty()
            && name
                .windows(phrase.len())
                .any(|window| window == phrase.as_slice())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn broken() -> CheckError {
        CheckError::Unsupported("API URL `x`: bad".to_owned())
    }

    /// What an ask answers: the error's text, or `started`.
    fn answer<T: ?Sized>(result: Result<&T, CheckError>) -> String {
        result
            .err()
            .map_or_else(|| "started".to_owned(), |error| error.to_string())
    }

    #[test]
    fn a_start_up_error_reaches_every_ask_naming_the_kind() {
        let registry = Registry::default()
            .with_tracker_error(SourceKind::GitHub, broken())
            .with_code_host_error(SourceKind::GitHub, broken())
            .with_index_error(IndexKind::DepsDev, broken());
        for _ in 0..2 {
            let github = "github: API URL `x`: bad";
            assert_eq!(answer(registry.tracker(SourceKind::GitHub)), github);
            assert_eq!(answer(registry.code_host(SourceKind::GitHub)), github);
            assert_eq!(
                answer(registry.index(IndexKind::DepsDev)),
                "deps.dev: API URL `x`: bad"
            );
        }
    }

    #[test]
    fn a_kind_nobody_registered_has_no_provider() {
        let registry = Registry::default().with_tracker_error(SourceKind::GitHub, broken());
        assert_eq!(
            answer(registry.tracker(SourceKind::GitLab)),
            "no provider for gitlab items"
        );
        assert_eq!(
            answer(registry.code_host(SourceKind::GitHub)),
            "no provider for github history"
        );
        assert_eq!(
            answer(registry.index(IndexKind::Repology)),
            "no provider for repology packages"
        );
    }
}
