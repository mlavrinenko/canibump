//! GitLab, gitlab.com and self-hosted, through its REST API v4.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

use super::http::{base, client, host_url, send_json};
use super::{CheckError, ItemState, Tracker};
use crate::refs::{ItemKind, ItemRef, Source};

/// A GitLab client for any host.
#[derive(Debug, Clone)]
pub struct GitLab {
    client: Client,
    base: Option<Url>,
    tokens: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ItemDto {
    state: String,
}

impl GitLab {
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

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        host: &str,
        project: &str,
        tail: &[&str],
        what: &str,
    ) -> Result<T, CheckError> {
        let mut url = host_url(self.base.as_ref(), host)?;
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("GitLab URL for {host}")))?
            .pop_if_empty()
            .extend(["api", "v4", "projects", project])
            .extend(tail);
        let mut request = self.client.get(url);
        if let Some(token) = super::http::credential(&self.tokens, self.base.as_ref(), host) {
            request = request.header("PRIVATE-TOKEN", token);
        }
        send_json(request, what).await
    }
}

#[async_trait]
impl Tracker for GitLab {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        let Source::GitLab { host, project } = &item.source else {
            return Err(CheckError::Unsupported(format!("{item} is not on GitLab")));
        };
        let what = item.to_string();
        let number = item.number.to_string();
        if item.kind == ItemKind::MergeRequest {
            let request: ItemDto = self
                .get(host, project, &["merge_requests", &number], &what)
                .await?;
            return Ok(match request.state.as_str() {
                "merged" => ItemState::Merged,
                "closed" => ItemState::ClosedUnmerged,
                _ => ItemState::Open,
            });
        }
        let issue: ItemDto = self.get(host, project, &["issues", &number], &what).await?;
        if issue.state != "closed" {
            return Ok(ItemState::Open);
        }
        // `closed_by` lists every MR that would close the issue, in any state.
        let closers: Vec<ItemDto> = self
            .get(host, project, &["issues", &number, "closed_by"], &what)
            .await?;
        Ok(if closers.iter().any(|mr| mr.state == "merged") {
            ItemState::Completed
        } else {
            ItemState::ClosedNoReason
        })
    }
}

#[cfg(test)]
#[path = "gitlab_tests.rs"]
mod tests;
