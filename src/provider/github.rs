//! GitHub through octocrab, with response types that hold only the fields
//! canibump reads.

use std::time::Duration;

use async_trait::async_trait;
use octocrab::Octocrab;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::{CheckError, CodeHost, GitRef, Item, ItemKind, ItemState, Reach, Tracker};
use crate::refs::{ItemRef, Source};

/// The public API root.
pub const API: &str = "https://api.github.com";

/// A GitHub client.
#[derive(Debug, Clone)]
pub struct GitHub {
    client: Octocrab,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ApiState {
    Open,
    Closed,
}

/// The pull request an issue stands for, as the issues API folds it in.
#[derive(Debug, Deserialize)]
struct PullLink {
    /// Set once the PR merged, null before.
    #[serde(default)]
    merged_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IssueDto {
    state: ApiState,
    #[serde(default)]
    state_reason: Option<String>,
    #[serde(default)]
    pull_request: Option<PullLink>,
}

#[derive(Debug, Deserialize)]
struct PullDto {
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    merge_commit_sha: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CompareStatus {
    Ahead,
    Identical,
    Behind,
    Diverged,
}

#[derive(Debug, Deserialize)]
struct CompareDto {
    status: CompareStatus,
}

#[derive(Debug, Deserialize)]
struct RefDto {
    #[serde(rename = "ref")]
    name: String,
}

impl GitHub {
    /// A client for `base` (default [`API`]), authenticated when `token`
    /// is given.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(base: Option<&str>, token: Option<String>) -> Result<Self, CheckError> {
        let base = base.unwrap_or(API);
        // Octocrab takes any string as a URI, then panics on the first request.
        super::http::base(base)?;
        let mut builder = Octocrab::builder()
            .base_uri(base)
            .map_err(|error| CheckError::Unsupported(format!("API URL `{base}`: {error}")))?
            .set_connect_timeout(Some(Duration::from_secs(10)))
            .set_read_timeout(Some(Duration::from_secs(30)));
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }
        let client = builder
            .build()
            .map_err(|error| CheckError::Unsupported(format!("HTTP client: {error}")))?;
        Ok(Self { client })
    }

    async fn get<T: DeserializeOwned>(&self, route: String, what: &str) -> Result<T, CheckError> {
        self.client
            .get::<T, _, ()>(route, None)
            .await
            .map_err(|error| classify(&error, what))
    }
}

/// The owner and repo of a GitHub source.
fn repo_path(source: &Source) -> Result<(&str, &str), CheckError> {
    match source {
        Source::GitHub { owner, repo } => Ok((owner, repo)),
        other => Err(CheckError::Unsupported(format!("{other} is not on GitHub"))),
    }
}

/// Maps an octocrab error onto what the user can act on.
fn classify(error: &octocrab::Error, what: &str) -> CheckError {
    let what = what.to_owned();
    let octocrab::Error::GitHub { source, .. } = error else {
        return CheckError::Network(format!("{what}: {error}"));
    };
    let rate_limited = source.message.to_lowercase().contains("rate limit");
    match source.status_code.as_u16() {
        404 => CheckError::NotFound(what),
        429 => CheckError::RateLimited(what),
        403 if rate_limited => CheckError::RateLimited(what),
        401 | 403 => CheckError::Auth(what),
        status => CheckError::Network(format!("{what}: HTTP {status} {}", source.message)),
    }
}

fn issue_state(state: &ApiState, reason: Option<&str>) -> ItemState {
    match (state, reason) {
        (ApiState::Open, _) => ItemState::Open,
        (ApiState::Closed, Some("completed")) => ItemState::Completed,
        (ApiState::Closed, Some("not_planned")) => ItemState::NotPlanned,
        (ApiState::Closed, Some("duplicate")) => ItemState::Duplicate,
        (ApiState::Closed, _) => ItemState::ClosedNoReason,
    }
}

/// A PR's state from its issue alone: `merged_at` tells merged from closed.
fn pull_state(state: &ApiState, link: &PullLink) -> ItemState {
    match (state, &link.merged_at) {
        (_, Some(_)) => ItemState::Merged,
        (ApiState::Open, None) => ItemState::Open,
        (ApiState::Closed, None) => ItemState::ClosedUnmerged,
    }
}

#[async_trait]
impl Tracker for GitHub {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        Ok(self.item(item).await?.state)
    }

    /// One `/issues/N` call: a PR rides along there as `pull_request`.
    async fn item(&self, item: &ItemRef) -> Result<Item, CheckError> {
        let (owner, repo) = repo_path(&item.source)?;
        let number = item.number;
        let issue: IssueDto = self
            .get(
                format!("/repos/{owner}/{repo}/issues/{number}"),
                &item.to_string(),
            )
            .await?;
        Ok(match &issue.pull_request {
            Some(link) => Item {
                state: pull_state(&issue.state, link),
                kind: Some(ItemKind::Pull),
            },
            None => Item {
                state: issue_state(&issue.state, issue.state_reason.as_deref()),
                kind: Some(ItemKind::Issue),
            },
        })
    }
}

#[async_trait]
impl CodeHost for GitHub {
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError> {
        let (owner, repo) = repo_path(&item.source)?;
        let number = item.number;
        let pull: PullDto = self
            .get(
                format!("/repos/{owner}/{repo}/pulls/{number}"),
                &item.to_string(),
            )
            .await?;
        Ok(pull.merge_commit_sha.filter(|_| pull.merged))
    }

    async fn contains(
        &self,
        source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<Reach, CheckError> {
        let (owner, repo) = repo_path(source)?;
        let name = target.name();
        let what = format!("{source}: {commit}...{name}");
        let compare: CompareDto = self
            .get(
                format!("/repos/{owner}/{repo}/compare/{commit}...{name}?per_page=1"),
                &what,
            )
            .await?;
        // `base...head` with the commit as base: `ahead` means the ref has
        // commits beyond it, `behind` that the ref is one of its ancestors.
        Ok(match compare.status {
            CompareStatus::Ahead | CompareStatus::Identical => Reach::Holds,
            CompareStatus::Behind => Reach::Before,
            CompareStatus::Diverged => Reach::Apart,
        })
    }

    async fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        let (owner, repo) = repo_path(source)?;
        let refs: Vec<RefDto> = self
            .get(
                format!("/repos/{owner}/{repo}/git/matching-refs/tags/{prefix}"),
                &format!("{source} tags"),
            )
            .await?;
        Ok(refs
            .into_iter()
            .filter_map(|reference| reference.name.strip_prefix("refs/tags/").map(str::to_owned))
            .collect())
    }
}

#[cfg(test)]
#[path = "github_tests.rs"]
mod tests;
