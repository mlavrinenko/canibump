//! The real providers, wired from the environment.
//!
//! `CANIBUMP_<PROVIDER>_API` (GITHUB, GITLAB, GITEA, REDMINE, JIRA, REPOLOGY,
//! DEPSDEV) points a provider elsewhere: a mock in tests, GitHub Enterprise,
//! a mirror.

use std::sync::Arc;

use super::depsdev::{self, DepsDev};
use super::git::GitRemotes;
use super::gitea::Gitea;
use super::github::GitHub;
use super::gitlab::GitLab;
use super::jira::Jira;
use super::redmine::Redmine;
use super::repology::{self, Repology};
use super::{CheckError, Registry, Tracker};
use crate::auth;
use crate::package::IndexKind;
use crate::refs::SourceKind;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_owned())
}

/// Logs a provider that cannot start; only the blockers needing it fail.
fn started<T>(provider: Result<T, CheckError>) -> Option<Arc<T>> {
    provider
        .inspect_err(|error| log::warn!("{error}"))
        .ok()
        .map(Arc::new)
}

fn with_tracker<T: Tracker + 'static>(
    registry: Registry,
    kind: SourceKind,
    provider: Result<T, CheckError>,
) -> Registry {
    match started(provider) {
        Some(tracker) => registry.with_tracker(kind, tracker),
        None => registry,
    }
}

/// Every provider canibump ships.
#[must_use]
pub fn registry() -> Registry {
    let mut registry = Registry::default();
    if let Some(root) = crate::cache::root() {
        let remotes = Arc::new(GitRemotes::new(root.join("git")));
        registry = registry.with_code_host(SourceKind::Git, remotes);
    }
    let github_api = env_or("CANIBUMP_GITHUB_API", super::github::API);
    if let Some(github) = started(GitHub::new(Some(&github_api), auth::github_token())) {
        registry = registry
            .with_tracker(SourceKind::GitHub, github.clone())
            .with_code_host(SourceKind::GitHub, github);
    }
    let gitlab = GitLab::new(env("CANIBUMP_GITLAB_API").as_deref(), auth::gitlab_tokens());
    registry = with_tracker(registry, SourceKind::GitLab, gitlab);
    let gitea = Gitea::new(env("CANIBUMP_GITEA_API").as_deref(), auth::gitea_tokens());
    registry = with_tracker(registry, SourceKind::Gitea, gitea);
    let redmine = Redmine::new(env("CANIBUMP_REDMINE_API").as_deref(), auth::redmine_keys());
    registry = with_tracker(registry, SourceKind::Redmine, redmine);
    let jira = Jira::new(env("CANIBUMP_JIRA_API").as_deref(), auth::jira_auth());
    registry = with_tracker(registry, SourceKind::Jira, jira);
    let repology_api = env_or("CANIBUMP_REPOLOGY_API", repology::API);
    if let Some(index) = started(Repology::new(&repology_api, repology::INTERVAL)) {
        registry = registry.with_index(IndexKind::Repology, index);
    }
    let depsdev_api = env_or("CANIBUMP_DEPSDEV_API", depsdev::API);
    if let Some(index) = started(DepsDev::new(&depsdev_api)) {
        registry = registry.with_index(IndexKind::DepsDev, index);
    }
    registry
}
