//! Jira through its REST API v2: the status category says whether an issue
//! is done, the resolution why.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

use super::http::{base, client, host_url, send_json};
use super::{CheckError, ItemState, Tracker, names_any, negated};
pub use crate::auth::JiraAuth;
use crate::refs::{ItemRef, Source};

/// A Jira client for any host.
#[derive(Debug, Clone)]
pub struct Jira {
    client: Client,
    base: Option<Url>,
    auth: HashMap<String, JiraAuth>,
}

#[derive(Debug, Deserialize)]
struct IssueDto {
    fields: FieldsDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldsDto {
    status: StatusDto,
    #[serde(default)]
    resolution: Option<NameDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusDto {
    status_category: CategoryDto,
}

#[derive(Debug, Deserialize)]
struct CategoryDto {
    key: String,
}

#[derive(Debug, Deserialize)]
struct NameDto {
    name: String,
}

/// Resolutions that mean it will not land.
const NOT_PLANNED: &[&str] = &[
    "won't fix",
    "won't do",
    "wont fix",
    "rejected",
    "declined",
    "cannot reproduce",
    "can't reproduce",
    "incomplete",
    "obsolete",
    "invalid",
    "abandoned",
    "not a bug",
    "not a problem",
    "works as designed",
];

/// Resolutions that mean it landed.
const COMPLETED: &[&str] = &[
    "fixed",
    "done",
    "resolved",
    "complete",
    "completed",
    "implemented",
    "delivered",
];

fn state_of(fields: &FieldsDto) -> ItemState {
    if fields.status.status_category.key != "done" {
        return ItemState::Open;
    }
    let Some(resolution) = &fields.resolution else {
        return ItemState::ClosedNoReason;
    };
    let has = |phrases: &[&str]| names_any(&resolution.name, phrases);
    if has(&["duplicate"]) {
        ItemState::Duplicate
    } else if has(NOT_PLANNED) {
        ItemState::NotPlanned
    } else if !negated(&resolution.name) && has(COMPLETED) {
        ItemState::Completed
    } else {
        ItemState::ClosedNoReason
    }
}

impl Jira {
    /// A client that talks to `https://<host>` for each item, or to `base`
    /// for every host when given; `auth` maps a host to its credentials.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(
        base_url: Option<&str>,
        auth: HashMap<String, JiraAuth>,
    ) -> Result<Self, CheckError> {
        Ok(Self {
            client: client()?,
            base: base_url.map(base).transpose()?,
            auth,
        })
    }
}

#[async_trait]
impl Tracker for Jira {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        let Source::Jira { host, project } = &item.source else {
            return Err(CheckError::Unsupported(format!("{item} is not on Jira")));
        };
        let mut url = host_url(self.base.as_ref(), host)?;
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("Jira URL for {host}")))?
            .pop_if_empty()
            .extend([
                "rest",
                "api",
                "2",
                "issue",
                &format!("{project}-{}", item.number),
            ]);
        url.query_pairs_mut()
            .append_pair("fields", "status,resolution");
        let request = match self.auth.get(host) {
            Some(JiraAuth::Bearer(token)) => self.client.get(url).bearer_auth(token),
            Some(JiraAuth::Basic(user, token)) => {
                self.client.get(url).basic_auth(user, Some(token))
            }
            None => self.client.get(url),
        };
        let issue: IssueDto = send_json(request, &item.to_string()).await?;
        Ok(state_of(&issue.fields))
    }
}

#[cfg(test)]
#[path = "jira_tests.rs"]
mod tests;
