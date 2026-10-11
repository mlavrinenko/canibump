//! Any git remote through gix: fetch only the refs asked about into a bare
//! repo under the cache dir, then answer ancestry in-process: one prefix
//! lookup and a walk that stops where the two histories meet, never one down
//! to the root.
//!
//! HTTP(S) remotes speak the git protocol natively. A local path or
//! `file://` remote makes gix spawn `git-upload-pack`, and `ssh://` spawns
//! `ssh`, as git itself would.

use std::collections::{HashMap, HashSet};
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
    /// One lock per remote URL, held while its repo is opened, created or
    /// fetched into.
    turns: Mutex<HashMap<String, Arc<Mutex<()>>>>,
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

/// Deletes the local refs `spec` writes into that are not in `kept`, the ones
/// the remote mapped this fetch.
///
/// The bare clone outlives the run, so a branch deleted or a tag retracted
/// upstream would otherwise keep answering from its stale local copy.
fn prune(
    repo: &gix::Repository,
    spec: &str,
    kept: &HashSet<gix::bstr::BString>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local = spec.rsplit(':').next().unwrap_or(spec);
    let owned: Vec<gix::Reference<'_>> = match local.strip_suffix('*') {
        Some(prefix) => repo
            .references()?
            .prefixed(prefix)?
            .collect::<Result<_, _>>()?,
        None => repo.try_find_reference(local)?.into_iter().collect(),
    };
    for reference in owned {
        if !kept.contains(reference.name().as_bstr()) {
            reference.delete()?;
        }
    }
    Ok(())
}

impl GitRemotes {
    /// Remotes cloned under `cache`.
    #[must_use]
    pub fn new(cache: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                cache,
                fetched: Mutex::new(HashSet::new()),
                turns: Mutex::new(HashMap::new()),
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
    /// The bare repo of `source` with `spec` fetched into it.
    ///
    /// One worker at a time opens or creates a remote's repo and fetches into
    /// it, so two blockers on one remote never race `init_bare` or a fetch.
    /// Reading the repo afterwards is not under the lock.
    fn synced(&self, source: &Source, spec: &str) -> Result<gix::Repository, CheckError> {
        let url = remote_url(source)?;
        let turn = Arc::clone(
            self.turns
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .entry(url.to_owned())
                .or_default(),
        );
        let _turn = turn.lock().unwrap_or_else(PoisonError::into_inner);
        let repo = self.repo(source)?;
        self.fetch(&repo, source, spec)?;
        Ok(repo)
    }

    fn repo(&self, source: &Source) -> Result<gix::Repository, CheckError> {
        let dir = clone_dir(&self.cache, remote_url(source)?);
        let fail = network(source);
        if dir.exists() {
            return gix::open(dir).map_err(|error| fail(&error));
        }
        std::fs::create_dir_all(&self.cache).map_err(|error| fail(&error))?;
        gix::init_bare(dir).map_err(|error| fail(&error))
    }

    /// Whether `spec` came this run, itself or inside a fetch of every tag.
    fn was_fetched(&self, url: &str, spec: &str) -> bool {
        let seen = self.fetched.lock().unwrap_or_else(PoisonError::into_inner);
        let has = |spec: &str| seen.contains(&(url.to_owned(), spec.to_owned()));
        has(spec) || (spec.starts_with(TAG_REFS) && has(ALL_TAGS))
    }

    /// Fetches `spec` once per run.
    fn fetch(&self, repo: &gix::Repository, source: &Source, spec: &str) -> Result<(), CheckError> {
        let url = remote_url(source)?;
        if self.was_fetched(url, spec) {
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
        let kept = match outcome {
            Ok(outcome) => outcome
                .ref_map
                .mappings
                .into_iter()
                .filter_map(|mapping| mapping.local)
                .collect(),
            Err(gix::remote::fetch::Error::NoMapping { .. }) => HashSet::new(),
            Err(error) => return Err(fail(&error)),
        };
        prune(repo, spec, &kept).map_err(|error| fail(&error))?;
        let key = (url.to_owned(), spec.to_owned());
        self.fetched
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key);
        Ok(())
    }

    fn contains(&self, source: &Source, sha: &str, target: &GitRef) -> Result<bool, CheckError> {
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
        let repo = self.synced(source, &spec)?;
        let missing = || CheckError::NotFound(format!("{source}: {}", target.name()));
        let mut reference = repo.find_reference(local.as_str()).map_err(|_| missing())?;
        let tip = reference.peel_to_id().map_err(|_| missing())?.detach();
        let Some(commit) = resolve(&repo, source, sha)? else {
            return Ok(false);
        };
        is_ancestor(&repo, source, commit, tip)
    }

    fn tags(&self, source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        let repo = self.synced(source, ALL_TAGS)?;
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

/// The refspecs of tags start like this.
const TAG_REFS: &str = "+refs/tags/";

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
