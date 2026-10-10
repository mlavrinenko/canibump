//! The `canibump.toml` manifest: settings and the blockers a repo waits on.
//!
//! Each `[[blocker]]` has an `id`, a `note` and exactly one condition key.
//! The condition is read by the key present, then deserialized into that
//! condition's own type, so an unknown key or a bad reference names the
//! blocker it sits in.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jiff::SignedDuration;
use schemars::JsonSchema;
use serde::Deserialize;
use thiserror::Error;
use toml::{Table, Value};

use crate::condition::{Condition, check_steps, parse_condition};
use crate::schema::BlockerSchema;

const DEFAULT_TTL: SignedDuration = SignedDuration::from_hours(6);

/// A parsed manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// `[settings]`.
    pub settings: Settings,
    /// Every `[[blocker]]`, in file order.
    pub blockers: Vec<Blocker>,
}

/// `[settings]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// How long a result stays fresh before `canibump` re-checks it.
    pub ttl: SignedDuration,
    /// Where the cache file lives, relative to the manifest; `None` picks the
    /// default under the user cache dir.
    pub cache: Option<PathBuf>,
}

/// One workaround waiting on something upstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    /// Unique name.
    pub id: String,
    /// What workaround it guards, and where.
    pub note: String,
    /// When the workaround can go.
    pub condition: Condition,
    /// The condition's TOML, so a cached result is dropped once it changes.
    pub spec: String,
}

/// A manifest that cannot be used.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// The file cannot be read.
    #[error("cannot read {path}: {reason}")]
    Read {
        /// The manifest path.
        path: PathBuf,
        /// Why.
        reason: std::io::Error,
    },
    /// The file is not valid TOML, or a top-level key is wrong.
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    /// `[settings] ttl` does not parse.
    #[error("settings.ttl: `{0}` is not a duration like \"6h\" or \"30m\"")]
    Ttl(String),
    /// A blocker without an id.
    #[error("blocker #{index}: missing `id`")]
    MissingId {
        /// 1-based position in the file.
        index: usize,
    },
    /// A blocker that is wrong.
    #[error("blocker `{id}`: {message}")]
    Blocker {
        /// Its id.
        id: String,
        /// What is wrong.
        message: String,
    },
    /// Two blockers share an id.
    #[error("duplicate blocker id `{0}`")]
    DuplicateId(String),
}

/// A `canibump.toml`: settings, and the blockers a repo waits on.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "canibump.toml")]
pub(crate) struct RawManifest {
    /// Tuning for the whole run.
    #[serde(default)]
    settings: RawSettings,
    /// One `[[blocker]]` per workaround waiting on something upstream.
    #[serde(default, rename = "blocker")]
    #[schemars(with = "Vec<BlockerSchema>")]
    blockers: Vec<Table>,
}

/// `[settings]`.
#[derive(Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Settings")]
pub(crate) struct RawSettings {
    /// How long a result stays fresh before `canibump` re-checks it, like
    /// `"6h"` or `"30m"`. Defaults to 6 hours.
    ttl: Option<String>,
    /// Where the cache file lives, relative to the manifest. Defaults to a file
    /// under the user cache dir.
    cache: Option<PathBuf>,
}

impl Manifest {
    /// Reads and parses a manifest file.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Read`] when the file cannot be read, else whatever
    /// [`Manifest::parse`] returns.
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let text = std::fs::read_to_string(path).map_err(|reason| ManifestError::Read {
            path: path.to_owned(),
            reason,
        })?;
        Self::parse(&text)
    }

    /// Parses manifest text.
    ///
    /// # Errors
    ///
    /// A [`ManifestError`] naming the setting or blocker that is wrong.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let raw: RawManifest = toml::from_str(text)?;
        let ttl = match raw.settings.ttl {
            None => DEFAULT_TTL,
            Some(text) => text.parse().map_err(|_| ManifestError::Ttl(text))?,
        };
        let mut seen = HashSet::new();
        let mut blockers = Vec::with_capacity(raw.blockers.len());
        for (index, table) in raw.blockers.into_iter().enumerate() {
            let blocker = parse_blocker(index + 1, table)?;
            if !seen.insert(blocker.id.clone()) {
                return Err(ManifestError::DuplicateId(blocker.id));
            }
            blockers.push(blocker);
        }
        Ok(Self {
            settings: Settings {
                ttl,
                cache: raw.settings.cache,
            },
            blockers,
        })
    }
}

fn take_string(table: &mut Table, key: &str) -> Option<Result<String, String>> {
    table.remove(key).map(|value| match value {
        Value::String(text) => Ok(text),
        other => Err(format!(
            "`{key}` must be a string, not {}",
            other.type_str()
        )),
    })
}

fn parse_blocker(index: usize, mut table: Table) -> Result<Blocker, ManifestError> {
    let id = match take_string(&mut table, "id") {
        None => return Err(ManifestError::MissingId { index }),
        Some(Err(message)) => {
            return Err(ManifestError::Blocker {
                id: format!("#{index}"),
                message,
            });
        }
        Some(Ok(id)) => id,
    };
    let fail = |message: String| ManifestError::Blocker {
        id: id.clone(),
        message,
    };
    let note = take_string(&mut table, "note")
        .ok_or_else(|| fail("missing `note`: say what workaround it guards".to_owned()))?
        .map_err(fail)?;
    let spec = toml::to_string(&table).map_err(|error| fail(error.to_string()))?;
    let condition = parse_condition(table).map_err(fail)?;
    check_steps(&condition).map_err(fail)?;
    Ok(Blocker {
        id,
        note,
        condition,
        spec,
    })
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
