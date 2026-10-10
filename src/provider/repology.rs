//! Repology: which version each distro repo carries, nixpkgs channels
//! included. Its API rules ask for at most one request per second and a
//! User-Agent naming the client; both hold here, and each project is asked
//! once per run.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;
use tokio::sync::Mutex;

use super::http::{base, client, get_json};
use super::{CheckError, PackageIndex};
use crate::package::PackageRef;

/// The public API root.
pub const API: &str = "https://repology.org/api/v1";

/// The spacing Repology's API rules ask for.
pub const INTERVAL: Duration = Duration::from_secs(1);

/// A Repology client.
#[derive(Debug)]
pub struct Repology {
    client: Client,
    base: Url,
    interval: Duration,
    last: Mutex<Option<Instant>>,
    projects: Mutex<HashMap<String, Vec<PackageDto>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct PackageDto {
    repo: String,
    #[serde(default)]
    srcname: Option<String>,
    version: String,
}

impl Repology {
    /// A client for `base` that spaces requests by `interval`.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(base_url: &str, interval: Duration) -> Result<Self, CheckError> {
        Ok(Self {
            client: client()?,
            base: base(base_url)?,
            interval,
            last: Mutex::new(None),
            projects: Mutex::new(HashMap::new()),
        })
    }

    async fn project(&self, project: &str) -> Result<Vec<PackageDto>, CheckError> {
        if let Some(cached) = self.projects.lock().await.get(project) {
            return Ok(cached.clone());
        }
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("API URL `{}`", self.base)))?
            .pop_if_empty()
            .extend(["project", project]);
        let mut last = self.last.lock().await;
        if let Some(previous) = *last {
            tokio::time::sleep(self.interval.saturating_sub(previous.elapsed())).await;
        }
        let result = get_json(&self.client, url, &format!("repology:{project}")).await;
        *last = Some(Instant::now());
        drop(last);
        let packages: Vec<PackageDto> = result?;
        self.projects
            .lock()
            .await
            .insert(project.to_owned(), packages.clone());
        Ok(packages)
    }
}

#[async_trait]
impl PackageIndex for Repology {
    async fn versions(&self, package: &PackageRef) -> Result<Vec<String>, CheckError> {
        let PackageRef::Repology {
            project,
            repo,
            srcname,
        } = package
        else {
            return Err(CheckError::Unsupported(format!(
                "{package} is not on Repology"
            )));
        };
        let packages = self.project(project).await?;
        if packages.is_empty() {
            return Err(CheckError::NotFound(format!("repology:{project}")));
        }
        Ok(packages
            .into_iter()
            .filter(|entry| &entry.repo == repo)
            .filter(|entry| srcname.is_none() || entry.srcname == *srcname)
            .map(|entry| entry.version)
            .collect())
    }
}

#[cfg(test)]
#[path = "repology_tests.rs"]
mod tests;
