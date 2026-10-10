//! The `canibump.toml` manifest: settings and the blockers a repo waits on.
//!
//! Each `[[blocker]]` has an `id`, a `note` and exactly one condition key.
//! The condition is read by the key present, then deserialized into that
//! condition's own type, so an unknown key or a bad reference names the
//! blocker it sits in.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jiff::SignedDuration;
use serde::Deserialize;
use thiserror::Error;
use toml::{Table, Value};

use crate::refs::{ItemRef, PackageRef, Reachable};

/// Condition keys a blocker may carry, one per blocker.
pub const CONDITION_KEYS: &[&str] = &["state", "reached", "version"];

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

/// When a blocker is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// An issue is fixed, or a PR merged.
    State(StateCondition),
    /// A commit, or a merged PR's merge commit, is in a branch or tag.
    Reached(ReachedCondition),
    /// A package version is at least a bound.
    Version(VersionCondition),
}

/// `state = "<item>"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateCondition {
    /// The issue or PR.
    pub item: ItemRef,
}

/// `reached = "<commit or PR>"` with `in = { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachedCondition {
    /// The commit, or the PR whose merge commit counts.
    pub target: Reachable,
    /// Where it has to arrive.
    pub within: Within,
}

/// `version = "<package>"` with `at-least = "<version>"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionCondition {
    /// The package and where to look.
    pub package: PackageRef,
    /// The lowest version that resolves it.
    pub at_least: String,
}

/// The `in` of a `reached` condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Within {
    /// `{ branch = "main" }`.
    Branch(String),
    /// `{ tag = "v1.2" }`.
    Tag(String),
    /// `{ tags = "v*", since = "0.5.0" }`: the first matching tag, in version
    /// order, at or after `since`.
    Tags {
        /// Which tags count.
        pattern: glob::Pattern,
        /// Ignore tags below this version.
        since: Option<String>,
    },
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    #[serde(default)]
    settings: RawSettings,
    #[serde(default, rename = "blocker")]
    blockers: Vec<Table>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSettings {
    ttl: Option<String>,
    cache: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawState {
    state: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReached {
    reached: String,
    #[serde(rename = "in")]
    within: RawWithin,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawVersion {
    version: String,
    repo: Option<String>,
    srcname: Option<String>,
    at_least: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWithin {
    branch: Option<String>,
    tag: Option<String>,
    tags: Option<String>,
    since: Option<String>,
}

impl RawWithin {
    fn parse(self) -> Result<Within, String> {
        let one_of = "`in` needs one of branch, tag, tags";
        match (self.branch, self.tag, self.tags, self.since) {
            (Some(_) | None, Some(_) | None, None, Some(_)) => {
                Err("`since` goes with `tags`".to_owned())
            }
            (Some(branch), None, None, None) => Ok(Within::Branch(branch)),
            (None, Some(tag), None, None) => Ok(Within::Tag(tag)),
            (None, None, Some(tags), since) => {
                let pattern = glob::Pattern::new(&tags)
                    .map_err(|error| format!("`tags = \"{tags}\"`: {error}"))?;
                Ok(Within::Tags { pattern, since })
            }
            _ => Err(one_of.to_owned()),
        }
    }
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
    Ok(Blocker {
        id,
        note,
        condition,
        spec,
    })
}

/// Deserializes a condition table into its own type.
fn typed<T: serde::de::DeserializeOwned>(table: Table) -> Result<T, String> {
    Value::Table(table)
        .try_into()
        .map_err(|error: toml::de::Error| error.message().to_owned())
}

/// Reads the one condition a table holds, by the condition key present.
fn parse_condition(table: Table) -> Result<Condition, String> {
    let present: Vec<&str> = CONDITION_KEYS
        .iter()
        .copied()
        .filter(|key| table.contains_key(*key))
        .collect();
    match present.as_slice() {
        ["state"] => {
            let raw: RawState = typed(table)?;
            let item = raw.state.parse().map_err(|error| format!("{error}"))?;
            Ok(Condition::State(StateCondition { item }))
        }
        ["version"] => {
            let raw: RawVersion = typed(table)?;
            let package = PackageRef::parse(&raw.version, raw.repo, raw.srcname)
                .map_err(|error| format!("{error}"))?;
            Ok(Condition::Version(VersionCondition {
                package,
                at_least: raw.at_least,
            }))
        }
        ["reached"] => {
            let raw: RawReached = typed(table)?;
            let target = raw.reached.parse().map_err(|error| format!("{error}"))?;
            let within = raw.within.parse()?;
            Ok(Condition::Reached(ReachedCondition { target, within }))
        }
        [] => Err(format!(
            "needs one condition: {}",
            CONDITION_KEYS.join(", ")
        )),
        several => Err(format!(
            "has several conditions ({}); keep one",
            several.join(", ")
        )),
    }
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
