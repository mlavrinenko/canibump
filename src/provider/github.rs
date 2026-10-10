//! GitHub through octocrab, with response types that hold only the fields
//! canibump reads.

use std::time::Duration;

use async_trait::async_trait;
use octocrab::Octocrab;
use serde::Deserialize;
use serde::de::{DeserializeOwned, IgnoredAny};

use super::{CheckError, CodeHost, GitRef, ItemState, Tracker};
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

#[derive(Debug, Deserialize)]
struct IssueDto {
    state: ApiState,
    #[serde(default)]
    state_reason: Option<String>,
    #[serde(default)]
    pull_request: Option<IgnoredAny>,
}

#[derive(Debug, Deserialize)]
struct PullDto {
    state: ApiState,
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
        let mut builder = Octocrab::builder()
            .base_uri(base.unwrap_or(API))
            .map_err(|error| CheckError::Unsupported(format!("GitHub API URL: {error}")))?
            .set_connect_timeout(Some(Duration::from_secs(10)))
            .set_read_timeout(Some(Duration::from_secs(30)));
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }
        let client = builder
            .build()
            .map_err(|error| CheckError::Unsupported(format!("GitHub client: {error}")))?;
        Ok(Self { client })
    }

    async fn get<T: DeserializeOwned>(&self, route: String, what: &str) -> Result<T, CheckError> {
        self.client
            .get::<T, _, ()>(route, None)
            .await
            .map_err(|error| classify(&error, what))
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

#[async_trait]
impl Tracker for GitHub {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        let Source::GitHub { owner, repo } = &item.source;
        let what = item.to_string();
        let number = item.number;
        let issue: IssueDto = self
            .get(format!("/repos/{owner}/{repo}/issues/{number}"), &what)
            .await?;
        if issue.pull_request.is_none() {
            return Ok(issue_state(&issue.state, issue.state_reason.as_deref()));
        }
        let pull: PullDto = self
            .get(format!("/repos/{owner}/{repo}/pulls/{number}"), &what)
            .await?;
        Ok(match (pull.state, pull.merged) {
            (_, true) => ItemState::Merged,
            (ApiState::Open, false) => ItemState::Open,
            (ApiState::Closed, false) => ItemState::ClosedUnmerged,
        })
    }
}

#[async_trait]
impl CodeHost for GitHub {
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError> {
        let Source::GitHub { owner, repo } = &item.source;
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
    ) -> Result<bool, CheckError> {
        let Source::GitHub { owner, repo } = source;
        let name = target.name();
        let what = format!("{source}: {commit}...{name}");
        let compare: CompareDto = self
            .get(
                format!("/repos/{owner}/{repo}/compare/{commit}...{name}?per_page=1"),
                &what,
            )
            .await?;
        Ok(matches!(
            compare.status,
            CompareStatus::Ahead | CompareStatus::Identical
        ))
    }

    async fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        let Source::GitHub { owner, repo } = source;
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
