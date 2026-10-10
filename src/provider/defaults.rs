//! The real providers, wired from the environment.
//!
//! `CANIBUMP_GITHUB_API`, `CANIBUMP_REPOLOGY_API` and `CANIBUMP_DEPSDEV_API`
//! point a provider elsewhere: a mock in tests, GitHub Enterprise, a mirror.

use std::sync::Arc;

use super::Registry;
use super::depsdev::{self, DepsDev};
use super::git::GitRemotes;
use super::github::GitHub;
use super::repology::{self, Repology};
use crate::package::IndexKind;
use crate::refs::SourceKind;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Every provider canibump ships. One that cannot start is logged and left
/// out, so only the blockers needing it fail.
#[must_use]
pub fn registry() -> Registry {
    let mut registry = Registry::default();
    if let Some(root) = crate::cache::root() {
        let remotes = Arc::new(GitRemotes::new(root.join("git")));
        registry = registry.with_code_host(SourceKind::Git, remotes);
    }
    let github_api = env_or("CANIBUMP_GITHUB_API", super::github::API);
    match GitHub::new(Some(&github_api), crate::auth::github_token()) {
        Ok(github) => {
            let github = Arc::new(github);
            registry = registry
                .with_tracker(SourceKind::GitHub, github.clone())
                .with_code_host(SourceKind::GitHub, github);
        }
        Err(error) => log::warn!("{error}"),
    }
    let repology_api = env_or("CANIBUMP_REPOLOGY_API", repology::API);
    match Repology::new(&repology_api, repology::INTERVAL) {
        Ok(index) => registry = registry.with_index(IndexKind::Repology, Arc::new(index)),
        Err(error) => log::warn!("{error}"),
    }
    match DepsDev::new(&env_or("CANIBUMP_DEPSDEV_API", depsdev::API)) {
        Ok(index) => registry = registry.with_index(IndexKind::DepsDev, Arc::new(index)),
        Err(error) => log::warn!("{error}"),
    }
    registry
}
