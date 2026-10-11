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
mod lazy;
mod memo;
pub mod redmine;
pub mod repology;

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use self::lazy::Lazy;
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

/// Whether an item is an issue or a pull or merge request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// An issue: it closes, but has no merge commit.
    Issue,
    /// A pull or merge request.
    Pull,
}

/// What a tracker says of an item in one answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    /// Its current state.
    pub state: ItemState,
    /// Its kind, when the answer tells; `None` when only the state does.
    pub kind: Option<ItemKind>,
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

    /// The item's state and, when the same answer tells, its kind. The
    /// default knows the state alone; a tracker that overrides it answers
    /// [`item_state`](Self::item_state) from it, so one request serves both.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the tracker cannot say.
    async fn item(&self, item: &ItemRef) -> Result<Item, CheckError> {
        let state = self.item_state(item).await?;
        Ok(Item { state, kind: None })
    }
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

/// Where a ref stands against a commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// The ref's history holds the commit, or the ref is the commit.
    Holds,
    /// Not yet: the ref is an ancestor of the commit, a point on its history
    /// from before it, so nothing the ref holds holds the commit either.
    Before,
    /// Neither holds the other: the ref is on another branch. This is also
    /// the answer when the host cannot see the commit at all.
    Apart,
}

impl Reach {
    /// Whether the ref holds the commit.
    #[must_use]
    pub const fn holds(self) -> bool {
        matches!(self, Self::Holds)
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

    /// Where `target` stands against `commit`.
    ///
    /// # Errors
    ///
    /// A [`CheckError`] when the host cannot say, or the ref does not exist.
    async fn contains(
        &self,
        source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<Reach, CheckError>;

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

/// The providers a run may use, keyed by source kind.
///
/// A provider starts when first asked for, at most once. One that fails to
/// start is kept as its [`CheckError`], so every blocker that needs it fails
/// with the reason; the others are not affected.
///
/// Every tracker and code host is handed out memoized: within one registry,
/// an item's state and a PR's merge commit are fetched once, however many
/// steps or blockers ask.
#[derive(Clone, Default)]
pub struct Registry {
    trackers: HashMap<SourceKind, Arc<Lazy<dyn Tracker>>>,
    code_hosts: HashMap<SourceKind, Arc<Lazy<dyn CodeHost>>>,
    indexes: HashMap<IndexKind, Arc<Lazy<dyn PackageIndex>>>,
}

impl Registry {
    /// Registers the tracker for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_tracker(self, kind: SourceKind, tracker: Arc<dyn Tracker>) -> Self {
        self.with_lazy_tracker(kind, move || Ok(Arc::clone(&tracker)))
    }

    /// Registers how to start the tracker for a source kind, replacing any
    /// earlier one. `start` runs on the first ask, and never again.
    #[must_use]
    pub fn with_lazy_tracker(
        mut self,
        kind: SourceKind,
        start: impl Fn() -> Result<Arc<dyn Tracker>, CheckError> + Send + Sync + 'static,
    ) -> Self {
        let start = move || start().map(memo::tracker);
        self.trackers.insert(kind, Arc::new(Lazy::new(start)));
        self
    }

    /// Registers the code host for a source kind, replacing any earlier one.
    #[must_use]
    pub fn with_code_host(self, kind: SourceKind, host: Arc<dyn CodeHost>) -> Self {
        self.with_lazy_code_host(kind, move || Ok(Arc::clone(&host)))
    }

    /// Registers how to start the code host for a source kind, replacing any
    /// earlier one. `start` runs on the first ask, and never again.
    #[must_use]
    pub fn with_lazy_code_host(
        mut self,
        kind: SourceKind,
        start: impl Fn() -> Result<Arc<dyn CodeHost>, CheckError> + Send + Sync + 'static,
    ) -> Self {
        let start = move || start().map(memo::code_host);
        self.code_hosts.insert(kind, Arc::new(Lazy::new(start)));
        self
    }

    /// Registers a package index, replacing any earlier one.
    #[must_use]
    pub fn with_index(self, kind: IndexKind, index: Arc<dyn PackageIndex>) -> Self {
        self.with_lazy_index(kind, move || Ok(Arc::clone(&index)))
    }

    /// Registers how to start the package index of a kind, replacing any
    /// earlier one. `start` runs on the first ask, and never again.
    #[must_use]
    pub fn with_lazy_index(
        mut self,
        kind: IndexKind,
        start: impl Fn() -> Result<Arc<dyn PackageIndex>, CheckError> + Send + Sync + 'static,
    ) -> Self {
        self.indexes.insert(kind, Arc::new(Lazy::new(start)));
        self
    }

    /// The tracker for a source kind, started if this is the first ask.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn tracker(&self, kind: SourceKind) -> Result<&dyn Tracker, CheckError> {
        lookup(&self.trackers, kind, "items")
    }

    /// The code host for a source kind, started if this is the first ask.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn code_host(&self, kind: SourceKind) -> Result<&dyn CodeHost, CheckError> {
        lookup(&self.code_hosts, kind, "history")
    }

    /// The package index of a kind, started if this is the first ask.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when none is registered, or it could not
    /// start.
    pub fn index(&self, kind: IndexKind) -> Result<&dyn PackageIndex, CheckError> {
        lookup(&self.indexes, kind, "packages")
    }
}

/// The provider of `kind`, started now if need be, or the reason there is
/// none, naming `kind`.
fn lookup<'a, K, T>(
    slots: &'a HashMap<K, Arc<Lazy<T>>>,
    kind: K,
    what: &str,
) -> Result<&'a T, CheckError>
where
    K: Copy + Eq + Hash + fmt::Display,
    T: ?Sized,
{
    match slots.get(&kind).map(|slot| slot.get()) {
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

/// Whether a status or resolution `name` says no, as in `Not Done` or
/// `Won't Do`, so a word like `done` in it does not mean the work landed.
pub(crate) fn negated(name: &str) -> bool {
    names_any(
        name,
        &[
            "not", "no", "never", "won't", "can't", "cannot", "didn't", "isn't", "don't",
        ],
    )
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
