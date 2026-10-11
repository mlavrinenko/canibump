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
    /// gitlab.com or a self-hosted GitLab.
    GitLab,
    /// Gitea, Forgejo or Codeberg.
    Gitea,
    /// A Redmine instance.
    Redmine,
    /// A Jira instance.
    Jira,
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
    /// A GitLab project.
    GitLab {
        /// `gitlab.com` or a self-hosted host.
        host: String,
        /// `group/sub/project`.
        project: String,
    },
    /// A Gitea or Forgejo repository (Codeberg included).
    Gitea {
        /// The instance host.
        host: String,
        /// The user or organisation.
        owner: String,
        /// The repository name.
        repo: String,
    },
    /// A Redmine instance.
    Redmine {
        /// The instance host, with a path when it is served under one.
        host: String,
    },
    /// A Jira project.
    Jira {
        /// The instance host, with a path when it is served under one.
        host: String,
        /// The project key, e.g. `CORE`.
        project: String,
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
            Self::GitLab { .. } => SourceKind::GitLab,
            Self::Gitea { .. } => SourceKind::Gitea,
            Self::Redmine { .. } => SourceKind::Redmine,
            Self::Jira { .. } => SourceKind::Jira,
        }
    }
}

/// How an item is numbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    /// `#N`: an issue, or on GitHub and Gitea a PR too.
    Issue,
    /// `!N`: a GitLab merge request.
    MergeRequest,
}

/// An issue, pull request or merge request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemRef {
    /// The repository or tracker holding the item.
    pub source: Source,
    /// The item's number.
    pub number: u64,
    /// `#` or `!`.
    pub kind: ItemKind,
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

pub(crate) fn split_scheme(text: &str) -> Result<(&str, &str), RefError> {
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
        match parse(text, "<forge>:<repo>#number")? {
            Reachable::Item(item) => Ok(item),
            Reachable::Commit(_) => Err(RefError::Malformed {
                text: text.to_owned(),
                expected: "<forge>:<repo>#number",
            }),
        }
    }
}

impl FromStr for Reachable {
    type Err = RefError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse(
            text,
            "<forge>:<repo>#number, <forge>:<repo>@sha or git:<url>@sha",
        )
    }
}

/// A commit id: 7 to 40 hex digits.
fn is_sha(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.chars().all(|char| char.is_ascii_hexdigit())
}

/// Schemes an item or commit reference may start with.
const SCHEMES: &[&str] = &[
    "github", "gitlab", "gitea", "forgejo", "codeberg", "redmine", "jira", "git",
];

/// Codeberg's host, which `codeberg:` stands for.
pub const CODEBERG: &str = "codeberg.org";

/// The default GitLab host.
pub const GITLAB_COM: &str = "gitlab.com";

/// Reads the repository part of a reference; `None` when it is malformed.
fn source_of(scheme: &str, path: &str) -> Option<Source> {
    match scheme {
        "github" => owner_repo(path).map(|(owner, repo)| Source::GitHub { owner, repo }),
        "git" => (!path.is_empty()).then(|| Source::Git {
            url: path.to_owned(),
        }),
        "gitlab" => gitlab_project(path),
        "gitea" | "forgejo" => {
            let (host, rest) = path.split_once('/')?;
            gitea_repo(host, rest)
        }
        "codeberg" => gitea_repo(CODEBERG, path),
        "redmine" => bare(path).map(|host| Source::Redmine { host }),
        _ => None,
    }
}

/// An instance: a host, maybe with a path for one served under it
/// (`issues.apache.org/jira`).
fn bare(host: &str) -> Option<String> {
    let valid = !host.is_empty()
        && !host.starts_with('/')
        && !host.ends_with('/')
        && !host.contains(char::is_whitespace);
    valid.then(|| host.to_owned())
}

/// `jira:host#KEY-N`.
fn parse_jira(locator: &str) -> Option<ItemRef> {
    let (host, key) = locator.split_once('#')?;
    let (project, number) = key.rsplit_once('-')?;
    let valid = !project.is_empty()
        && project
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '_');
    Some(ItemRef {
        source: Source::Jira {
            host: bare(host)?,
            project: valid.then(|| project.to_owned())?,
        },
        number: number.parse().ok()?,
        kind: ItemKind::Issue,
    })
}

fn gitea_repo(host: &str, path: &str) -> Option<Source> {
    let (owner, repo) = owner_repo(path)?;
    (!host.is_empty()).then(|| Source::Gitea {
        host: host.to_owned(),
        owner,
        repo,
    })
}

/// `[host/]group/.../project`: a first segment with a dot is the host when
/// at least two segments follow it.
fn gitlab_project(path: &str) -> Option<Source> {
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 || segments.iter().any(|segment| segment.is_empty()) {
        return None;
    }
    let (host, project) = match segments.split_first() {
        Some((first, rest)) if first.contains('.') && rest.len() >= 2 => (*first, rest.join("/")),
        _ => (GITLAB_COM, path.to_owned()),
    };
    Some(Source::GitLab {
        host: host.to_owned(),
        project,
    })
}

/// Parses `<scheme>:<path>#<number>`, `<scheme>:<path>!<number>` (GitLab)
/// or `<scheme>:<path>@<sha>`.
fn parse(text: &str, expected: &'static str) -> Result<Reachable, RefError> {
    let (scheme, locator) = split_scheme(text)?;
    let malformed = || RefError::Malformed {
        text: text.to_owned(),
        expected,
    };
    if !SCHEMES.contains(&scheme) {
        return Err(RefError::UnknownScheme {
            scheme: scheme.to_owned(),
            text: text.to_owned(),
            known: "github, gitlab, gitea, forgejo, codeberg, redmine, jira, git",
        });
    }
    if scheme == "jira" {
        return parse_jira(locator)
            .map(Reachable::Item)
            .ok_or_else(malformed);
    }
    if let Some((path, sha)) = locator.rsplit_once('@') {
        let source = source_of(scheme, path).ok_or_else(malformed)?;
        if !is_sha(sha) {
            return Err(malformed());
        }
        return Ok(Reachable::Commit(CommitRef {
            source,
            sha: sha.to_owned(),
        }));
    }
    let split = locator
        .rfind(['#', '!'])
        .and_then(|at| Some((locator.get(..at)?, locator.get(at..)?)));
    let (path, kind, number) = match split {
        Some((path, suffix)) if scheme != "git" => match suffix.split_at(1) {
            ("#", number) => (path, ItemKind::Issue, number),
            ("!", number) if scheme == "gitlab" => (path, ItemKind::MergeRequest, number),
            _ => return Err(malformed()),
        },
        _ => return Err(malformed()),
    };
    let source = source_of(scheme, path).ok_or_else(malformed)?;
    let number = number.parse().map_err(|_| malformed())?;
    Ok(Reachable::Item(ItemRef {
        source,
        number,
        kind,
    }))
}

/// `url` with its password, or an http(s) user that stands in for a token,
/// shown as `***`, so a message or the state file never carries a secret.
///
/// Only a `scheme://` URL has userinfo. An ssh user alone is no secret, and
/// an `@` past the host belongs to the path.
#[must_use]
pub fn redacted(url: &str) -> std::borrow::Cow<'_, str> {
    let unchanged = std::borrow::Cow::Borrowed(url);
    let Some((scheme, rest)) = url.split_once("://") else {
        return unchanged;
    };
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let Some((userinfo, host)) = authority.rsplit_once('@') else {
        return unchanged;
    };
    let user = match userinfo.split_once(':') {
        Some((user, _)) => format!("{user}:***"),
        None if scheme.starts_with("http") => "***".to_owned(),
        None => return unchanged,
    };
    std::borrow::Cow::Owned(format!("{scheme}://{user}@{host}{path}"))
}

impl fmt::Display for SourceKind {
    /// The scheme a manifest writes for it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::GitHub => "github",
            Self::Git => "git",
            Self::GitLab => "gitlab",
            Self::Gitea => "gitea",
            Self::Redmine => "redmine",
            Self::Jira => "jira",
        })
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub { owner, repo } => write!(formatter, "github:{owner}/{repo}"),
            Self::Git { url } => write!(formatter, "git:{}", redacted(url)),
            Self::GitLab { host, project } if host == GITLAB_COM => {
                write!(formatter, "gitlab:{project}")
            }
            Self::GitLab { host, project } => write!(formatter, "gitlab:{host}/{project}"),
            Self::Gitea { host, owner, repo } if host == CODEBERG => {
                write!(formatter, "codeberg:{owner}/{repo}")
            }
            Self::Gitea { host, owner, repo } => write!(formatter, "gitea:{host}/{owner}/{repo}"),
            Self::Redmine { host } => write!(formatter, "redmine:{host}"),
            Self::Jira { host, project } => write!(formatter, "jira:{host} {project}"),
        }
    }
}

impl fmt::Display for ItemRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let separator = match self.kind {
            ItemKind::Issue => '#',
            ItemKind::MergeRequest => '!',
        };
        if let Source::Jira { host, project } = &self.source {
            return write!(formatter, "jira:{host}#{project}-{}", self.number);
        }
        write!(formatter, "{}{separator}{}", self.source, self.number)
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
