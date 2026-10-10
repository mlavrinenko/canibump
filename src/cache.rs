//! The state file: every blocker's last result with its check time, so other
//! tools read it offline.
//!
//! It lives at `$XDG_CACHE_HOME/canibump/state/<absolute manifest path>.json`
//! unless `--cache` or `[settings] cache` says otherwise.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::outcome::Status;

/// The state file format version.
pub const VERSION: u32 = 1;

/// The whole state file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateFile {
    /// [`VERSION`] when written.
    pub version: u32,
    /// The manifest these entries come from.
    pub manifest: PathBuf,
    /// One entry per blocker, in manifest order.
    pub entries: Vec<Entry>,
}

/// One blocker's last result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The blocker id.
    pub id: String,
    /// The blocker note.
    pub note: String,
    /// What the check found.
    pub status: Status,
    /// Why, in one line.
    pub detail: String,
    /// When the check ran.
    pub checked_at: Timestamp,
    /// Until when the result is reused without a new check.
    pub fresh_until: Timestamp,
    /// The condition it was checked against.
    pub spec: String,
}

impl Entry {
    /// Whether a new run may reuse this result. A failure never is.
    #[must_use]
    pub fn is_fresh(&self, now: Timestamp) -> bool {
        self.status != Status::Failed && now < self.fresh_until
    }

    /// The end of the freshness window for a result checked at `checked_at`.
    /// A failure gets none, so the next run re-checks it.
    #[must_use]
    pub fn freshness(status: Status, checked_at: Timestamp, ttl: SignedDuration) -> Timestamp {
        if status == Status::Failed {
            return checked_at;
        }
        checked_at.checked_add(ttl).unwrap_or(Timestamp::MAX)
    }
}

/// A state file that cannot be read or written.
#[derive(Debug, Error)]
pub enum CacheError {
    /// No cache dir: neither `XDG_CACHE_HOME` nor `HOME` is set.
    #[error("no cache dir: set XDG_CACHE_HOME or HOME, or pass --cache")]
    NoHome,
    /// File system trouble.
    #[error("{path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// Not a state file.
    #[error("{path}: not a canibump state file: {source}")]
    Format {
        /// The file.
        path: PathBuf,
        /// Why.
        source: serde_json::Error,
    },
}

fn io_error(path: &Path) -> impl FnOnce(io::Error) -> CacheError + '_ {
    move |source| CacheError::Io {
        path: path.to_owned(),
        source,
    }
}

/// Picks the state file: the `--cache` flag, else the manifest's `cache`
/// setting (relative to the manifest), else the default under the cache dir.
///
/// # Errors
///
/// [`CacheError`] when the manifest path cannot be made absolute or no cache
/// dir is known.
pub fn resolve(
    manifest: &Path,
    setting: Option<&Path>,
    flag: Option<&Path>,
) -> Result<PathBuf, CacheError> {
    if let Some(flag) = flag {
        return Ok(flag.to_owned());
    }
    let absolute = std::path::absolute(manifest).map_err(io_error(manifest))?;
    let canonical = absolute.canonicalize().unwrap_or(absolute);
    if let Some(setting) = setting {
        let dir = canonical.parent().unwrap_or_else(|| Path::new("/"));
        return Ok(dir.join(setting));
    }
    let home = cache_home(|key| std::env::var_os(key)).ok_or(CacheError::NoHome)?;
    Ok(default_path(&home, &canonical))
}

fn cache_home(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let set = |key| env(key).filter(|value: &OsString| !value.is_empty());
    set("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| set("HOME").map(|home| PathBuf::from(home).join(".cache")))
}

/// `<cache home>/canibump/state/<manifest path without the root>.json`.
fn default_path(cache_home: &Path, manifest: &Path) -> PathBuf {
    let relative: PathBuf = manifest
        .components()
        .filter(|part| matches!(part, std::path::Component::Normal(_)))
        .collect();
    let mut name = relative.into_os_string();
    name.push(".json");
    cache_home.join("canibump").join("state").join(name)
}

/// Reads the state file; `None` when there is none yet.
///
/// # Errors
///
/// [`CacheError`] when it exists but cannot be read or parsed.
pub fn read(path: &Path) -> Result<Option<StateFile>, CacheError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(path)(error)),
    };
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|source| CacheError::Format {
            path: path.to_owned(),
            source,
        })
}

/// Writes the state file atomically (temp file, then rename).
///
/// # Errors
///
/// [`CacheError::Io`] when the directory or file cannot be written.
pub fn write(path: &Path, state: &StateFile) -> Result<(), CacheError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_error(dir))?;
    }
    let mut text = serde_json::to_string_pretty(state).map_err(|source| CacheError::Format {
        path: path.to_owned(),
        source,
    })?;
    text.push('\n');
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    std::fs::write(&temp, text).map_err(io_error(&temp))?;
    std::fs::rename(&temp, path).map_err(io_error(path))
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
