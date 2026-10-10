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
use super::{CheckError, PackageIndex, Registry, Tracker};
use crate::auth;
use crate::package::IndexKind;
use crate::refs::SourceKind;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_owned())
}

/// Registers a tracker, or why it could not start; only the blockers
/// needing it fail.
fn tracker<T: Tracker + 'static>(
    registry: Registry,
    kind: SourceKind,
    provider: Result<T, CheckError>,
) -> Registry {
    match provider {
        Ok(tracker) => registry.with_tracker(kind, Arc::new(tracker)),
        Err(error) => registry.with_tracker_error(kind, error),
    }
}

/// Registers a package index, or why it could not start.
fn index<T: PackageIndex + 'static>(
    registry: Registry,
    kind: IndexKind,
    provider: Result<T, CheckError>,
) -> Registry {
    match provider {
        Ok(index) => registry.with_index(kind, Arc::new(index)),
        Err(error) => registry.with_index_error(kind, error),
    }
}

/// Registers GitHub as tracker and code host, from one client.
fn github(registry: Registry, provider: Result<GitHub, CheckError>) -> Registry {
    match provider {
        Ok(github) => {
            let github = Arc::new(github);
            registry
                .with_tracker(SourceKind::GitHub, github.clone())
                .with_code_host(SourceKind::GitHub, github)
        }
        Err(error) => registry
            .with_tracker_error(SourceKind::GitHub, error.clone())
            .with_code_host_error(SourceKind::GitHub, error),
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
    registry = github(
        registry,
        GitHub::new(Some(&github_api), auth::github_token()),
    );
    let gitlab = GitLab::new(env("CANIBUMP_GITLAB_API").as_deref(), auth::gitlab_tokens());
    registry = tracker(registry, SourceKind::GitLab, gitlab);
    let gitea = Gitea::new(env("CANIBUMP_GITEA_API").as_deref(), auth::gitea_tokens());
    registry = tracker(registry, SourceKind::Gitea, gitea);
    let redmine = Redmine::new(env("CANIBUMP_REDMINE_API").as_deref(), auth::redmine_keys());
    registry = tracker(registry, SourceKind::Redmine, redmine);
    let jira = Jira::new(env("CANIBUMP_JIRA_API").as_deref(), auth::jira_auth());
    registry = tracker(registry, SourceKind::Jira, jira);
    let repology_api = env_or("CANIBUMP_REPOLOGY_API", repology::API);
    let repology = Repology::new(&repology_api, repology::INTERVAL);
    registry = index(registry, IndexKind::Repology, repology);
    let depsdev_api = env_or("CANIBUMP_DEPSDEV_API", depsdev::API);
    index(registry, IndexKind::DepsDev, DepsDev::new(&depsdev_api))
}
