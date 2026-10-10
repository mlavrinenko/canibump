//! Reference strings that name what a blocker waits on.
//!
//! A reference is `<scheme>:<locator>`, e.g. `github:serde-rs/serde#2590`.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Which provider answers for a [`Source`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SourceKind {
    /// github.com.
    GitHub,
    /// Any git remote.
    Git,
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
    /// A git remote by URL (or local path).
    Git {
        /// What `git clone` would take.
        url: String,
    },
}

impl Source {
    /// The provider kind that answers for this source.
    #[must_use]
    pub const fn kind(&self) -> SourceKind {
        match self {
            Self::GitHub { .. } => SourceKind::GitHub,
            Self::Git { .. } => SourceKind::Git,
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

/// A commit in a repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitRef {
    /// The repository.
    pub source: Source,
    /// The commit id as written.
    pub sha: String,
}

/// What a `reached` condition follows: a commit, or a PR's merge commit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reachable {
    /// A PR or MR; its merge commit once merged.
    Item(ItemRef),
    /// A commit.
    Commit(CommitRef),
}

impl Reachable {
    /// The repository it lives in.
    #[must_use]
    pub const fn source(&self) -> &Source {
        match self {
            Self::Item(item) => &item.source,
            Self::Commit(commit) => &commit.source,
        }
    }
}

/// Which package index answers for a [`PackageRef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexKind {
    /// repology.org: distro repos, incl. nixpkgs channels.
    Repology,
    /// deps.dev: registry packages.
    DepsDev,
}

/// deps.dev systems.
pub const DEPS_DEV_SYSTEMS: &[&str] = &["cargo", "npm", "pypi", "go", "maven", "nuget", "rubygems"];

/// A package in an index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageRef {
    /// A Repology project in one repo.
    Repology {
        /// The Repology project name.
        project: String,
        /// The repo, e.g. `nix_unstable`.
        repo: String,
        /// Only packages with this source name.
        srcname: Option<String>,
    },
    /// A registry package on deps.dev.
    DepsDev {
        /// `cargo`, `npm`, `pypi`, `go`, `maven`, `nuget` or `rubygems`.
        system: String,
        /// The package name as the registry spells it.
        name: String,
    },
}

impl PackageRef {
    /// Parses `repology:<project>` (with `repo`, optionally `srcname`) or
    /// `deps.dev:<system>/<name>`.
    ///
    /// # Errors
    ///
    /// [`RefError`] naming what is wrong.
    pub fn parse(
        text: &str,
        repo: Option<String>,
        srcname: Option<String>,
    ) -> Result<Self, RefError> {
        let (scheme, locator) = split_scheme(text)?;
        let invalid = |message: &str| RefError::Invalid(format!("`{text}` {message}"));
        match scheme {
            "repology" => {
                if locator.is_empty() {
                    return Err(RefError::Malformed {
                        text: text.to_owned(),
                        expected: "repology:<project>",
                    });
                }
                let repo =
                    repo.ok_or_else(|| invalid("needs `repo`, e.g. repo = \"nix_unstable\""))?;
                Ok(Self::Repology {
                    project: locator.to_owned(),
                    repo,
                    srcname,
                })
            }
            "deps.dev" => {
                if repo.is_some() {
                    return Err(invalid("takes no `repo`"));
                }
                if srcname.is_some() {
                    return Err(invalid("takes no `srcname`"));
                }
                let (system, name) = locator
                    .split_once('/')
                    .filter(|(_, name)| !name.is_empty())
                    .ok_or_else(|| RefError::Malformed {
                        text: text.to_owned(),
                        expected: "deps.dev:<system>/<name>",
                    })?;
                let system = system.to_ascii_lowercase();
                if !DEPS_DEV_SYSTEMS.contains(&system.as_str()) {
                    return Err(invalid(&format!(
                        "names system `{system}`; known: {}",
                        DEPS_DEV_SYSTEMS.join(", ")
                    )));
                }
                Ok(Self::DepsDev {
                    system,
                    name: name.to_owned(),
                })
            }
            other => Err(RefError::UnknownScheme {
                scheme: other.to_owned(),
                text: text.to_owned(),
                known: "repology, deps.dev",
            }),
        }
    }

    /// The index that answers for it.
    #[must_use]
    pub const fn index(&self) -> IndexKind {
        match self {
            Self::Repology { .. } => IndexKind::Repology,
            Self::DepsDev { .. } => IndexKind::DepsDev,
        }
    }
}

impl fmt::Display for PackageRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repology { project, repo, .. } => {
                write!(formatter, "repology:{project} in {repo}")
            }
            Self::DepsDev { system, name } => write!(formatter, "deps.dev:{system}/{name}"),
        }
    }
}

/// A reference string that does not parse.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RefError {
    /// No `scheme:` prefix.
    #[error("`{0}` has no scheme; write it like `github:owner/repo#12`")]
    NoScheme(String),
    /// A scheme nobody answers for.
    #[error("unknown scheme `{scheme}` in `{text}`; known: {known}")]
    UnknownScheme {
        /// The scheme as written.
        scheme: String,
        /// The whole reference.
        text: String,
        /// The schemes that fit here.
        known: &'static str,
    },
    /// A reference that parses but does not make sense.
    #[error("{0}")]
    Invalid(String),
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
        match parse(text, "github:owner/repo#number")? {
            Reachable::Item(item) => Ok(item),
            Reachable::Commit(_) => Err(RefError::Malformed {
                text: text.to_owned(),
                expected: "github:owner/repo#number",
            }),
        }
    }
}

impl FromStr for Reachable {
    type Err = RefError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse(
            text,
            "github:owner/repo#number, github:owner/repo@sha or git:<url>@sha",
        )
    }
}

/// A commit id: 7 to 40 hex digits.
fn is_sha(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.chars().all(|char| char.is_ascii_hexdigit())
}

/// Parses `<scheme>:<path>#<number>` or `<scheme>:<path>@<sha>`.
fn parse(text: &str, expected: &'static str) -> Result<Reachable, RefError> {
    let (scheme, locator) = split_scheme(text)?;
    let malformed = || RefError::Malformed {
        text: text.to_owned(),
        expected,
    };
    let source = |path: &str| match scheme {
        "github" => owner_repo(path)
            .map(|(owner, repo)| Source::GitHub { owner, repo })
            .ok_or_else(malformed),
        "git" if !path.is_empty() => Ok(Source::Git {
            url: path.to_owned(),
        }),
        "git" => Err(malformed()),
        other => Err(RefError::UnknownScheme {
            scheme: other.to_owned(),
            text: text.to_owned(),
            known: "github, git",
        }),
    };
    if let Some((path, sha)) = locator.rsplit_once('@') {
        let source = source(path)?;
        if !is_sha(sha) {
            return Err(malformed());
        }
        return Ok(Reachable::Commit(CommitRef {
            source,
            sha: sha.to_owned(),
        }));
    }
    if scheme == "git" {
        return Err(malformed());
    }
    let (path, number) = locator
        .rsplit_once('#')
        .ok_or_else(|| source(locator).err().unwrap_or_else(malformed))?;
    let source = source(path)?;
    let number = number.parse().map_err(|_| malformed())?;
    Ok(Reachable::Item(ItemRef { source, number }))
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub { owner, repo } => write!(formatter, "github:{owner}/{repo}"),
            Self::Git { url } => write!(formatter, "git:{url}"),
        }
    }
}

impl fmt::Display for ItemRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.source, self.number)
    }
}

impl fmt::Display for CommitRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.source, self.sha)
    }
}

impl fmt::Display for Reachable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(item) => item.fmt(formatter),
            Self::Commit(commit) => commit.fmt(formatter),
        }
    }
}

#[cfg(test)]
#[path = "refs_tests.rs"]
mod tests;
