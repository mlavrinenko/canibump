//! The `canibump.toml` manifest: settings and the blockers a repo waits on.
//!
//! Each `[[blocker]]` has an `id`, a `note` and exactly one condition key.
//! The condition is read by the key present, then deserialized into that
//! condition's own type, so an unknown key or a bad reference names the
//! blocker it sits in.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jiff::{SignedDuration, Span, SpanRelativeTo};
use schemars::JsonSchema;
use serde::Deserialize;
use thiserror::Error;
use toml::{Spanned, Table, Value};

use crate::condition::{Condition, check_steps, parse_condition};
use crate::report::Actionable;
use crate::scan::{RawScan, Scan};
use crate::schema::BlockerSchema;
use crate::sites::{self, Declared};

const DEFAULT_TTL: SignedDuration = SignedDuration::from_hours(6);

/// A friendly duration of weeks down to seconds, a day counting 24 hours. A month
/// or a year has no fixed length, so it is not one.
fn parse_ttl(text: &str) -> Option<SignedDuration> {
    let span: Span = text.parse().ok()?;
    span.to_duration(SpanRelativeTo::days_are_24_hours()).ok()
}

/// A parsed manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// `[settings]`.
    pub settings: Settings,
    /// Every `[[blocker]]`, in file order.
    pub blockers: Vec<Blocker>,
    /// `[scan]`, when the manifest has one.
    pub scan: Option<Scan>,
}

/// `[settings]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// How long a result stays fresh before `canibump` re-checks it.
    pub ttl: SignedDuration,
    /// Where the cache file lives, relative to the manifest; `None` picks the
    /// default under the user cache dir.
    pub cache: Option<PathBuf>,
    /// What fails the run; `None` counts all five, `Some([])` nothing.
    pub fail_on: Option<Vec<Actionable>>,
}

/// One workaround waiting on something upstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    /// Unique name.
    pub id: String,
    /// What workaround it guards.
    pub note: String,
    /// Where it sits in files that take no comment.
    pub sites: Vec<Declared>,
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
    /// `[scan]` is wrong.
    #[error("{0}")]
    Scan(String),
    /// `[settings] ttl` does not parse.
    #[error(
        "settings.ttl: `{0}` is not a duration like \"6h\", \"30m\" or \"1d\" (weeks, days, hours, minutes and seconds)"
    )]
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
            Self::Read { .. } | Self::Toml(_) | Self::Ttl(_) | Self::Scan(_) => None,
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
    /// Reads the repo's files for upstream links no blocker lists.
    scan: Option<RawScan>,
}

/// `[settings]`.
#[derive(Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Settings")]
pub(crate) struct RawSettings {
    /// How long a result stays fresh before `canibump` re-checks it, like
    /// `"6h"`, `"30m"` or `"1d"`. Takes weeks, days, hours, minutes and
    /// seconds, a day being 24 hours. Defaults to 6 hours.
    ttl: Option<String>,
    /// Where the cache file lives, relative to the manifest. Defaults to a file
    /// under the user cache dir.
    cache: Option<PathBuf>,
    /// What fails the run, with an exit code from 9 to 63: any of
    /// `"bumpable"`, `"dead"`, `"failed"`, `"unlisted"` and `"unmarked"`.
    /// Defaults to all five; `[]` never fails. `--fail-on` overrides it.
    #[serde(rename = "fail-on")]
    fail_on: Option<Vec<Actionable>>,
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
            Some(text) => parse_ttl(&text).ok_or(ManifestError::Ttl(text))?,
        };
        let scan = raw
            .scan
            .map(RawScan::parse)
            .transpose()
            .map_err(ManifestError::Scan)?;
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
                fail_on: raw.settings.fail_on,
            },
            blockers,
            scan,
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
    let markable = |char: char| char.is_ascii_alphanumeric() || char == '_' || char == '-';
    if id.is_empty() || !id.chars().all(markable) {
        return Err(fail(
            "an id takes only ASCII letters, digits, `_` and `-`, so `canibump:<id>` can mark it"
                .to_owned(),
        ));
    }
    let note = take_string(&mut table, "note")
        .ok_or_else(|| fail("missing `note`: say what workaround it guards".to_owned()))?
        .map_err(fail)?;
    let sites = match table.remove("sites") {
        Some(value) => sites::parse(value).map_err(fail)?,
        None => Vec::new(),
    };
    let spec = toml::to_string(&table).map_err(|error| fail(error.to_string()))?;
    let condition = parse_condition(table).map_err(fail)?;
    check_steps(&condition).map_err(fail)?;
    Ok(Blocker {
        id,
        note,
        sites,
        condition,
        spec,
    })
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
