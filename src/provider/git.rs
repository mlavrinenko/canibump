//! Any git remote through gix: fetch only the refs asked about into a bare
//! repo under the cache dir, then answer ancestry in-process: one prefix
//! lookup and a walk that stops where the two histories meet, never one down
//! to the root.
//!
//! HTTP(S) remotes speak the git protocol natively. A local path or
//! `file://` remote makes gix spawn `git-upload-pack`, and `ssh://` spawns
//! `ssh`, as git itself would.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use gix::remote::Direction;

use super::{CheckError, CodeHost, GitRef};
use crate::refs::{ItemRef, Source};

/// Git remotes, cloned bare under one cache dir.
#[derive(Debug, Clone)]
pub struct GitRemotes {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    cache: PathBuf,
    /// Refspecs already fetched this run, as `(url, spec)`.
    fetched: Mutex<HashSet<(String, String)>>,
}

/// The bare repo a remote is fetched into.
fn clone_dir(cache: &Path, url: &str) -> PathBuf {
    let name: String = url
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || matches!(char, '.' | '-') {
                char
            } else {
                '_'
            }
        })
        .collect();
    cache.join(format!("{name}.git"))
}

fn remote_url(source: &Source) -> Result<&str, CheckError> {
    match source {
        Source::Git { url } => Ok(url),
        other => Err(CheckError::Unsupported(format!(
            "{other} is not a git remote"
        ))),
    }
}

fn network(source: &Source) -> impl Fn(&dyn std::fmt::Display) -> CheckError + '_ {
    move |error| CheckError::Network(format!("{source}: {error}"))
}

/// The one commit the written `sha` names among the objects fetched, `None`
/// when there is none. Trees and blobs sharing the digits do not count.
///
/// The fetch brought the whole history of the ref asked about, so a commit it
/// lacks is not reached rather than an error.
fn resolve(
    repo: &gix::Repository,
    source: &Source,
    sha: &str,
) -> Result<Option<gix::ObjectId>, CheckError> {
    let fail = network(source);
    let prefix = gix::hash::Prefix::from_hex(sha)
        .map_err(|error| CheckError::Unsupported(format!("{source}: {sha}: {error}")))?;
    let mut matches = HashSet::new();
    repo.objects
        .lookup_prefix(prefix, Some(&mut matches))
        .map_err(|error| fail(&error))?;
    let mut commits = Vec::new();
    for id in matches {
        let header = repo.find_header(id).map_err(|error| fail(&error))?;
        if header.kind() == gix::object::Kind::Commit {
            commits.push(id);
        }
    }
    match commits.as_slice() {
        [] => Ok(None),
        [only] => Ok(Some(*only)),
        several => Err(CheckError::Unsupported(format!(
            "{source}: {sha} starts {} commits, write more digits",
            several.len()
        ))),
    }
}

/// Whether `tip` reaches `commit`.
///
/// Walks from `commit` with the history of `tip` hidden: gix paints both
/// sides outward in generation order and stops where they meet, so a commit
/// the tip reaches leaves nothing visible, and a miss ends at the fork point,
/// never at the root of a long history.
fn is_ancestor(
    repo: &gix::Repository,
    source: &Source,
    commit: gix::ObjectId,
    tip: gix::ObjectId,
) -> Result<bool, CheckError> {
    let fail = network(source);
    let mut walk = repo
        .rev_walk([commit])
        .with_hidden([tip])
        .all()
        .map_err(|error| fail(&error))?;
    match walk.next() {
        None => Ok(true),
        Some(Ok(_)) => Ok(false),
        Some(Err(error)) => Err(fail(&error)),
    }
}

impl GitRemotes {
    /// Remotes cloned under `cache`.
    #[must_use]
    pub fn new(cache: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                cache,
                fetched: Mutex::new(HashSet::new()),
            }),
        }
    }

    async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Inner) -> Result<T, CheckError> + Send + 'static,
    ) -> Result<T, CheckError> {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || work(&inner))
            .await
            .map_err(|error| CheckError::Network(format!("git worker: {error}")))?
    }
}

impl Inner {
    fn repo(&self, source: &Source) -> Result<gix::Repository, CheckError> {
        let dir = clone_dir(&self.cache, remote_url(source)?);
        let fail = network(source);
        if dir.exists() {
            return gix::open(dir).map_err(|error| fail(&error));
        }
        std::fs::create_dir_all(&self.cache).map_err(|error| fail(&error))?;
        gix::init_bare(dir).map_err(|error| fail(&error))
    }

    /// Fetches `spec` once per run.
    fn fetch(&self, repo: &gix::Repository, source: &Source, spec: &str) -> Result<(), CheckError> {
        let url = remote_url(source)?;
        let key = (url.to_owned(), spec.to_owned());
        let seen = || self.fetched.lock().unwrap_or_else(PoisonError::into_inner);
        if seen().contains(&key) {
            return Ok(());
        }
        let fail = network(source);
        let remote = repo
            .remote_at(url)
            .map_err(|error| fail(&error))?
            .with_refspecs([spec], Direction::Fetch)
            .map_err(|error| fail(&error))?;
        let outcome = remote
            .connect(Direction::Fetch)
            .map_err(|error| fail(&error))?
            .prepare_fetch(
                gix::progress::Discard,
                gix::remote::ref_map::Options::default(),
            )
            .map_err(|error| fail(&error))?
            .receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED);
        match outcome {
            Ok(_) | Err(gix::remote::fetch::Error::NoMapping { .. }) => {}
            Err(error) => return Err(fail(&error)),
        }
        seen().insert(key);
        Ok(())
    }

    fn contains(&self, source: &Source, sha: &str, target: &GitRef) -> Result<bool, CheckError> {
        let repo = self.repo(source)?;
        let (spec, local) = match target {
            GitRef::Tag(name) => (
                format!("+refs/tags/{name}:refs/tags/{name}"),
                format!("refs/tags/{name}"),
            ),
            GitRef::Branch(name) => (
                format!("+refs/heads/{name}:refs/remotes/origin/{name}"),
                format!("refs/remotes/origin/{name}"),
            ),
        };
        let tag_fetched = matches!(target, GitRef::Tag(_)) && self.has_tags(source)?;
        if !tag_fetched {
            self.fetch(&repo, source, &spec)?;
        }
        let missing = || CheckError::NotFound(format!("{source}: {}", target.name()));
        let mut reference = repo.find_reference(local.as_str()).map_err(|_| missing())?;
        let tip = reference.peel_to_id().map_err(|_| missing())?.detach();
        let Some(commit) = resolve(&repo, source, sha)? else {
            return Ok(false);
        };
        is_ancestor(&repo, source, commit, tip)
    }

    /// Whether every tag was fetched this run, so a tag needs no fetch of its own.
    fn has_tags(&self, source: &Source) -> Result<bool, CheckError> {
        let key = (remote_url(source)?.to_owned(), ALL_TAGS.to_owned());
        let fetched = self.fetched.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(fetched.contains(&key))
    }

    fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        let repo = self.repo(source)?;
        self.fetch(&repo, source, ALL_TAGS)?;
        let fail = network(source);
        let platform = repo.references().map_err(|error| fail(&error))?;
        let mut tags = Vec::new();
        for reference in platform.tags().map_err(|error| fail(&error))? {
            let reference = reference.map_err(|error| fail(&error))?;
            let name = reference.name().shorten().to_string();
            if name.starts_with(prefix) {
                tags.push(name);
            }
        }
        Ok(tags)
    }
}

const ALL_TAGS: &str = "+refs/tags/*:refs/tags/*";

#[async_trait]
impl CodeHost for GitRemotes {
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError> {
        Err(CheckError::Unsupported(format!(
            "{item}: a git remote has no pull requests"
        )))
    }

    async fn contains(
        &self,
        source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<bool, CheckError> {
        let (source, commit, target) = (source.clone(), commit.to_owned(), target.clone());
        self.blocking(move |inner| inner.contains(&source, &commit, &target))
            .await
    }

    async fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        let (source, prefix) = (source.clone(), prefix.to_owned());
        self.blocking(move |inner| inner.tags(&source, &prefix))
            .await
    }
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
