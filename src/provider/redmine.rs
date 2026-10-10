//! Redmine through its REST API. Status names are per instance, so the
//! name decides a rejection or a duplicate, and `is_closed` (or a closing
//! name on instances too old to send it) a fix.

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

fn state_of(status: &StatusDto) -> ItemState {
    let has = |phrases: &[&str]| names_any(&status.name, phrases);
    if has(&[
        "rejected",
        "reject",
        "won't fix",
        "wont fix",
        "invalid",
        "declined",
        "abandoned",
    ]) {
        ItemState::NotPlanned
    } else if has(&["duplicate"]) {
        ItemState::Duplicate
    } else if status.is_closed == Some(true) || has(&["closed", "resolved", "done", "fixed"]) {
        ItemState::Completed
    } else {
        ItemState::Open
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
