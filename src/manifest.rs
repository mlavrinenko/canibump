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
use toml::{Spanned, Table, Value};

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
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    /// `[settings] ttl` does not parse.
    #[error("settings.ttl: `{0}` is not a duration like \"6h\" or \"30m\"")]
    Ttl(String),
    /// A blocker without an id.
    #[error("blocker #{index}: missing `id`")]
    MissingId {
        /// 1-based position in the file.
        index: usize,
        /// The line its table starts on.
        line: usize,
    },
    /// A blocker that is wrong.
    #[error("blocker `{id}`: {message}")]
    Blocker {
        /// Its id.
        id: String,
        /// What is wrong.
        message: String,
        /// The line its table starts on.
        line: usize,
    },
    /// Two blockers share an id.
    #[error("duplicate blocker id `{id}`")]
    DuplicateId {
        /// The shared id.
        id: String,
        /// The line the second blocker's table starts on.
        line: usize,
    },
    /// Another error, found in the manifest at `path`. The inner error is a
    /// field, not a `source`, so a chain prints it once.
    #[error("{}: {error}", place(.path, .error.line()))]
    File {
        /// The manifest path, as given.
        path: PathBuf,
        /// What is wrong in it.
        error: Box<ManifestError>,
    },
}

impl ManifestError {
    /// The line the offending blocker starts on, when the error sits in one.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::MissingId { line, .. }
            | Self::Blocker { line, .. }
            | Self::DuplicateId { line, .. } => Some(*line),
            Self::File { error, .. } => error.line(),
            Self::Read { .. } | Self::Toml(_) | Self::Ttl(_) => None,
        }
    }
}

/// `path`, or `path:line` when the error sits in a blocker.
fn place(path: &Path, line: Option<usize>) -> String {
    match line {
        Some(line) => format!("{}:{line}", path.display()),
        None => path.display().to_string(),
    }
}

/// The 1-based line a byte offset falls on.
fn line_of(text: &str, offset: usize) -> usize {
    1 + text
        .bytes()
        .take(offset)
        .filter(|byte| *byte == b'\n')
        .count()
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
    blockers: Vec<Spanned<Table>>,
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
    /// [`Manifest::parse`] returns, as a [`ManifestError::File`] naming `path`.
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let text = std::fs::read_to_string(path).map_err(|reason| ManifestError::Read {
            path: path.to_owned(),
            reason,
        })?;
        Self::parse(&text).map_err(|error| ManifestError::File {
            path: path.to_owned(),
            error: Box::new(error),
        })
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
        for (index, spanned) in raw.blockers.into_iter().enumerate() {
            let line = line_of(text, spanned.span().start);
            let blocker = parse_blocker(index + 1, line, spanned.into_inner())?;
            if !seen.insert(blocker.id.clone()) {
                return Err(ManifestError::DuplicateId {
                    id: blocker.id,
                    line,
                });
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

fn parse_blocker(index: usize, line: usize, mut table: Table) -> Result<Blocker, ManifestError> {
    let id = match take_string(&mut table, "id") {
        None => return Err(ManifestError::MissingId { index, line }),
        Some(Err(message)) => {
            return Err(ManifestError::Blocker {
                id: format!("#{index}"),
                message,
                line,
            });
        }
        Some(Ok(id)) => id,
    };
    let fail = |message: String| ManifestError::Blocker {
        id: id.clone(),
        message,
        line,
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
