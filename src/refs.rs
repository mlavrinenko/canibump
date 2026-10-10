//! Reference strings that name what a blocker waits on.
//!
//! A reference is `<scheme>:<locator>`, e.g. `github:serde-rs/serde#2590`.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Schemes a reference may start with.
pub const SCHEMES: &[&str] = &["github"];

/// Which provider answers for a [`Source`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SourceKind {
    /// github.com.
    GitHub,
}

/// Where an item lives.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    /// A GitHub repository.
    GitHub {
        /// The user or organisation.
        owner: String,
        /// The repository name.
        repo: String,
    },
}

impl Source {
    /// The provider kind that answers for this source.
    #[must_use]
    pub const fn kind(&self) -> SourceKind {
        match self {
            Self::GitHub { .. } => SourceKind::GitHub,
        }
    }
}

/// An issue, pull request or merge request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemRef {
    /// The repository or tracker holding the item.
    pub source: Source,
    /// The item's number.
    pub number: u64,
}

/// A reference string that does not parse.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RefError {
    /// No `scheme:` prefix.
    #[error("`{0}` has no scheme; write it like `github:owner/repo#12`")]
    NoScheme(String),
    /// A scheme nobody answers for.
    #[error("unknown scheme `{scheme}` in `{text}`; known: {known}", known = SCHEMES.join(", "))]
    UnknownScheme {
        /// The scheme as written.
        scheme: String,
        /// The whole reference.
        text: String,
    },
    /// The locator after the scheme is not the expected shape.
    #[error("`{text}` is not `{expected}`")]
    Malformed {
        /// The whole reference.
        text: String,
        /// The shape the scheme wants.
        expected: &'static str,
    },
}

fn split_scheme(text: &str) -> Result<(&str, &str), RefError> {
    text.split_once(':')
        .ok_or_else(|| RefError::NoScheme(text.to_owned()))
}

/// Splits `owner/repo` into its two non-empty parts.
fn owner_repo(path: &str) -> Option<(String, String)> {
    let (owner, repo) = path.split_once('/')?;
    let valid = |part: &str| !part.is_empty() && !part.contains('/');
    (valid(owner) && valid(repo)).then(|| (owner.to_owned(), repo.to_owned()))
}

impl FromStr for ItemRef {
    type Err = RefError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (scheme, locator) = split_scheme(text)?;
        let malformed = || RefError::Malformed {
            text: text.to_owned(),
            expected: "github:owner/repo#number",
        };
        match scheme {
            "github" => {
                let (path, number) = locator.rsplit_once('#').ok_or_else(malformed)?;
                let (owner, repo) = owner_repo(path).ok_or_else(malformed)?;
                let number = number.parse().map_err(|_| malformed())?;
                Ok(Self {
                    source: Source::GitHub { owner, repo },
                    number,
                })
            }
            other => Err(RefError::UnknownScheme {
                scheme: other.to_owned(),
                text: text.to_owned(),
            }),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub { owner, repo } => write!(formatter, "github:{owner}/{repo}"),
        }
    }
}

impl fmt::Display for ItemRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.source, self.number)
    }
}

#[cfg(test)]
#[path = "refs_tests.rs"]
mod tests;
