//! deps.dev: published versions for crates.io, npm, PyPI, Go, Maven, NuGet
//! and RubyGems through one API.

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

use super::http::{base, client, get_json};
use super::{CheckError, PackageIndex};
use crate::refs::PackageRef;

/// The public API root.
pub const API: &str = "https://api.deps.dev";

/// A deps.dev client.
#[derive(Debug, Clone)]
pub struct DepsDev {
    client: Client,
    base: Url,
}

#[derive(Debug, Deserialize)]
struct PackageDto {
    #[serde(default)]
    versions: Vec<VersionDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionDto {
    version_key: VersionKeyDto,
}

#[derive(Debug, Deserialize)]
struct VersionKeyDto {
    version: String,
}

impl DepsDev {
    /// A client for `base`.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unsupported`] when `base` is not a URL.
    pub fn new(base_url: &str) -> Result<Self, CheckError> {
        Ok(Self {
            client: client()?,
            base: base(base_url)?,
        })
    }
}

#[async_trait]
impl PackageIndex for DepsDev {
    async fn versions(&self, package: &PackageRef) -> Result<Vec<String>, CheckError> {
        let PackageRef::DepsDev { system, name } = package else {
            return Err(CheckError::Unsupported(format!(
                "{package} is not on deps.dev"
            )));
        };
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|()| CheckError::Unsupported(format!("API URL `{}`", self.base)))?
            .pop_if_empty()
            .extend(["v3", "systems", system, "packages", name]);
        let found: PackageDto = get_json(&self.client, url, &package.to_string()).await?;
        Ok(found
            .versions
            .into_iter()
            .map(|version| version.version_key.version)
            .collect())
    }
}

#[cfg(test)]
#[path = "depsdev_tests.rs"]
mod tests;
