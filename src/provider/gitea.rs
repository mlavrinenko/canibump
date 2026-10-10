//! Gitea and Forgejo (Codeberg included) through their REST API v1.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

use super::http::{base, client, host_url, send_json};
use super::{CheckError, ItemState, Tracker};
use crate::refs::{ItemRef, Source};

/// A Gitea client for any host.
#[derive(Debug, Clone)]
pub struct Gitea {
    client: Client,
    base: Option<Url>,
    tokens: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct IssueDto {
    state: String,
    #[serde(default)]
    pull_request: Option<PullDto>,
}

#[derive(Debug, Deserialize)]
struct PullDto {
    #[serde(default)]
    merged: bool,
}

impl Gitea {
    /// A client that talks to `https://<host>` for each item, or to `base`
    /// for every host when given; `tokens` maps a host to its token.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(
        base_url: Option<&str>,
        tokens: HashMap<String, String>,
    ) -> Result<Self, CheckError> {
        Ok(Self {
            client: client()?,
            base: base_url.map(base).transpose()?,
            tokens,
        })
    }
}

#[async_trait]
impl Tracker for Gitea {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        let Source::Gitea { host, owner, repo } = &item.source else {
            return Err(CheckError::Unsupported(format!("{item} is not on Gitea")));
        };
        let mut url = host_url(self.base.as_ref(), host)?;
        let number = item.number.to_string();
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("Gitea URL for {host}")))?
            .pop_if_empty()
            .extend(["api", "v1", "repos", owner, repo, "issues", &number]);
        let mut request = self.client.get(url);
        if let Some(token) = self.tokens.get(host) {
            request = request.header("Authorization", format!("token {token}"));
        }
        let issue: IssueDto = send_json(request, &item.to_string()).await?;
        let closed = issue.state == "closed";
        Ok(match (issue.pull_request, closed) {
            (Some(PullDto { merged: true }), _) => ItemState::Merged,
            (_, false) => ItemState::Open,
            (Some(_), true) => ItemState::ClosedUnmerged,
            (None, true) => ItemState::ClosedNoReason,
        })
    }
}

#[cfg(test)]
#[path = "gitea_tests.rs"]
mod tests;
