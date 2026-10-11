//! Redmine through its REST API. Status names are per instance, so `is_closed`
//! says whether an issue is over and the name only says why (rejected,
//! duplicate, else fixed). Instances too old to send it get the name's guess.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

use super::http::{base, client, host_url, send_json};
use super::{CheckError, ItemState, Tracker, names_any};
use crate::refs::{ItemRef, Source};

/// A Redmine client for any host.
#[derive(Debug, Clone)]
pub struct Redmine {
    client: Client,
    base: Option<Url>,
    keys: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    issue: IssueDto,
}

#[derive(Debug, Deserialize)]
struct IssueDto {
    status: StatusDto,
}

#[derive(Debug, Deserialize)]
struct StatusDto {
    name: String,
    #[serde(default)]
    is_closed: Option<bool>,
}

/// Names that mean it will not land.
const NOT_PLANNED: &[&str] = &[
    "rejected",
    "reject",
    "won't fix",
    "wont fix",
    "invalid",
    "declined",
    "abandoned",
];

/// Names that mean it is finished, for instances that send no `is_closed`.
const FINISHED: &[&str] = &["closed", "resolved", "done", "fixed"];

/// `is_closed` says whether the issue is over, and the name only says why.
/// An instance too old to send it gets the name's guess at both.
fn state_of(status: &StatusDto) -> ItemState {
    let has = |phrases: &[&str]| names_any(&status.name, phrases);
    let closed = status
        .is_closed
        .unwrap_or_else(|| has(NOT_PLANNED) || has(&["duplicate"]) || has(FINISHED));
    if !closed {
        ItemState::Open
    } else if has(NOT_PLANNED) {
        ItemState::NotPlanned
    } else if has(&["duplicate"]) {
        ItemState::Duplicate
    } else {
        ItemState::Completed
    }
}

impl Redmine {
    /// A client that talks to `https://<host>` for each item, or to `base`
    /// for every host when given; `keys` maps a host to its API key.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(base_url: Option<&str>, keys: HashMap<String, String>) -> Result<Self, CheckError> {
        Ok(Self {
            client: client()?,
            base: base_url.map(base).transpose()?,
            keys,
        })
    }
}

#[async_trait]
impl Tracker for Redmine {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        let Source::Redmine { host } = &item.source else {
            return Err(CheckError::Unsupported(format!("{item} is not on Redmine")));
        };
        let mut url = host_url(self.base.as_ref(), host)?;
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("Redmine URL for {host}")))?
            .pop_if_empty()
            .extend(["issues", &format!("{}.json", item.number)]);
        let mut request = self.client.get(url);
        if let Some(key) = self.keys.get(host) {
            request = request.header("X-Redmine-API-Key", key);
        }
        let envelope: Envelope = send_json(request, &item.to_string()).await?;
        Ok(state_of(&envelope.issue.status))
    }
}

#[cfg(test)]
#[path = "redmine_tests.rs"]
mod tests;
