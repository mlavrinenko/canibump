//! The real providers, wired from the environment.
//!
//! `CANIBUMP_<PROVIDER>_API` (GITHUB, GITLAB, GITEA, REDMINE, JIRA, REPOLOGY,
//! DEPSDEV) points a provider elsewhere: a mock in tests, GitHub Enterprise,
//! a mirror.
//!
//! Nothing starts here. Each provider is a factory the registry runs when a
//! blocker first needs it, so a manifest that never mentions GitHub never
//! runs `gh`.

use std::sync::Arc;

use super::depsdev::{self, DepsDev};
use super::git::GitRemotes;
use super::gitea::Gitea;
use super::github::GitHub;
use super::gitlab::GitLab;
use super::jira::Jira;
use super::lazy::Lazy;
use super::redmine::Redmine;
use super::repology::{self, Repology};
use super::{CheckError, CodeHost, PackageIndex, Registry, Tracker};
use crate::auth;
use crate::package::IndexKind;
use crate::refs::SourceKind;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_owned())
}

/// Registers a tracker that `start` builds on demand.
fn tracker<T: Tracker + 'static>(
    registry: Registry,
    kind: SourceKind,
    start: impl Fn() -> Result<T, CheckError> + Send + Sync + 'static,
) -> Registry {
    registry.with_lazy_tracker(kind, move || {
        start().map(|provider| Arc::new(provider) as Arc<dyn Tracker>)
    })
}

/// Registers a package index that `start` builds on demand.
fn index<T: PackageIndex + 'static>(
    registry: Registry,
    kind: IndexKind,
    start: impl Fn() -> Result<T, CheckError> + Send + Sync + 'static,
) -> Registry {
    registry.with_lazy_index(kind, move || {
        start().map(|provider| Arc::new(provider) as Arc<dyn PackageIndex>)
    })
}

/// Registers GitHub as tracker and code host. Both slots share one client,
/// started by whichever is asked first.
fn github(registry: Registry) -> Registry {
    let client = Arc::new(Lazy::new(|| {
        let api = env_or("CANIBUMP_GITHUB_API", super::github::API);
        GitHub::new(Some(&api), auth::github_token()).map(Arc::new)
    }));
    let for_tracker = Arc::clone(&client);
    registry
        .with_lazy_tracker(SourceKind::GitHub, move || {
            for_tracker
                .get()
                .clone()
                .map(|github| github as Arc<dyn Tracker>)
        })
        .with_lazy_code_host(SourceKind::GitHub, move || {
            client
                .get()
                .clone()
                .map(|github| github as Arc<dyn CodeHost>)
        })
}

/// Every provider canibump ships, none of them started.
#[must_use]
pub fn registry() -> Registry {
    let mut registry = Registry::default();
    if let Some(root) = crate::cache::root() {
        registry = registry.with_lazy_code_host(SourceKind::Git, move || {
            Ok(Arc::new(GitRemotes::new(root.join("git"))))
        });
    }
    registry = github(registry);
    registry = tracker(registry, SourceKind::GitLab, || {
        GitLab::new(env("CANIBUMP_GITLAB_API").as_deref(), auth::gitlab_tokens())
    });
    registry = tracker(registry, SourceKind::Gitea, || {
        Gitea::new(env("CANIBUMP_GITEA_API").as_deref(), auth::gitea_tokens())
    });
    registry = tracker(registry, SourceKind::Redmine, || {
        Redmine::new(env("CANIBUMP_REDMINE_API").as_deref(), auth::redmine_keys())
    });
    registry = tracker(registry, SourceKind::Jira, || {
        Jira::new(env("CANIBUMP_JIRA_API").as_deref(), auth::jira_auth())
    });
    registry = index(registry, IndexKind::Repology, || {
        Repology::new(
            &env_or("CANIBUMP_REPOLOGY_API", repology::API),
            repology::INTERVAL,
        )
    });
    index(registry, IndexKind::DepsDev, || {
        DepsDev::new(&env_or("CANIBUMP_DEPSDEV_API", depsdev::API))
    })
}
