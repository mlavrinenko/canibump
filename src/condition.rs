//! Conditions: when a blocker, or one step of it, is resolved.
//!
//! A condition table is read by the key present, then deserialized into
//! that condition's own type, so an unknown key or a bad reference is an
//! error naming it.

use std::collections::HashSet;

use schemars::JsonSchema;
use serde::Deserialize;
use toml::{Table, Value};

use crate::package::PackageRef;
use crate::refs::{ItemRef, Reachable, SourceKind};

/// Condition keys a blocker may carry, one per blocker.
pub const CONDITION_KEYS: &[&str] = &["state", "reached", "version", "all", "any"];

/// The sources `reached` can follow: those with a code host in
/// `provider::defaults`.
const HISTORY: &[SourceKind] = &[SourceKind::GitHub, SourceKind::Git];

/// When a blocker is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// An issue is fixed, or a PR merged.
    State(StateCondition),
    /// A commit, or a merged PR's merge commit, is in a branch or tag.
    Reached(ReachedCondition),
    /// A package version is at least a bound.
    Version(VersionCondition),
    /// Every step holds, checked in order.
    All(Vec<Step>),
    /// Some step holds, checked in order.
    Any(Vec<Step>),
}

/// One step of an all-of or any-of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// A name later steps refer to, e.g. `at-least = { step = "released" }`.
    pub id: Option<String>,
    /// What must hold.
    pub condition: Condition,
}

/// The `at-least` of a `version` condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bound {
    /// A literal version.
    Version(String),
    /// The tag an earlier `reached` step with `in = { tags = ... }` found.
    Step(String),
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
    pub at_least: Bound,
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

/// Waits on an issue, PR or MR. Merged, or closed as completed, is bumpable.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "StateCondition")]
pub(crate) struct RawState {
    /// The item: `github:owner/repo#N`, `gitlab:group/project!N`, `codeberg:owner/repo#N`,
    /// `gitea:host/owner/repo#N`, `forgejo:host/owner/repo#N`, `redmine:host#N` or
    /// `jira:host#KEY-N`.
    state: String,
}

/// Waits for a commit, or a PR's merge commit, to land in a branch or tag.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ReachedCondition")]
pub(crate) struct RawReached {
    /// The commit or PR: `github:owner/repo#N`, `github:owner/repo@<sha>` or
    /// `git:<remote url>@<sha>`.
    reached: String,
    /// Where it has to arrive.
    #[serde(rename = "in")]
    within: RawWithin,
}

/// Waits for a package to reach a version.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
#[schemars(rename = "VersionCondition")]
pub(crate) struct RawVersion {
    /// The package: `repology:<project>` or `deps.dev:<system>/<name>`, where
    /// the system is cargo, npm, pypi, go, maven, nuget or rubygems.
    version: String,
    /// The Repology repo to read, e.g. `nix_unstable`.
    repo: Option<String>,
    /// The package name in that repo, when it ships several.
    srcname: Option<String>,
    /// The lowest version that resolves the blocker.
    at_least: RawBound,
}

/// The lowest version that resolves a `version` condition.
#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "Bound")]
pub(crate) enum RawBound {
    /// A literal version.
    Version(String),
    /// The tag an earlier `reached` step with `in = { tags = ... }` found.
    Step {
        /// The `id` of that step.
        step: String,
    },
}

/// Where a `reached` commit has to arrive: exactly one of `branch`, `tag` or
/// `tags`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Within")]
pub(crate) struct RawWithin {
    /// A branch name.
    branch: Option<String>,
    /// An exact tag name.
    tag: Option<String>,
    /// A glob such as `v*`; the first matching tag, in version order, counts.
    tags: Option<String>,
    /// With `tags`, ignore tags below this version.
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

/// Deserializes a condition table into its own type.
fn typed<T: serde::de::DeserializeOwned>(table: Table) -> Result<T, String> {
    Value::Table(table)
        .try_into()
        .map_err(|error: toml::de::Error| error.message().to_owned())
}

/// Reads the one condition a table holds, by the condition key present.
pub(crate) fn parse_condition(table: Table) -> Result<Condition, String> {
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
                at_least: match raw.at_least {
                    RawBound::Version(version) => Bound::Version(version),
                    RawBound::Step { step } => Bound::Step(step),
                },
            }))
        }
        ["reached"] => {
            let raw: RawReached = typed(table)?;
            let target: Reachable = raw.reached.parse().map_err(|error| format!("{error}"))?;
            if !HISTORY.contains(&target.source().kind()) {
                return Err(format!(
                    "`reached` takes github:owner/repo#N, github:owner/repo@<sha> or git:<url>@<sha>, not `{}`",
                    raw.reached
                ));
            }
            let within = raw.within.parse()?;
            Ok(Condition::Reached(ReachedCondition { target, within }))
        }
        ["all"] => Ok(Condition::All(parse_steps(table, "all")?)),
        ["any"] => Ok(Condition::Any(parse_steps(table, "any")?)),
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

/// Reads the steps under `key`, which must be the table's only key.
fn parse_steps(mut table: Table, key: &str) -> Result<Vec<Step>, String> {
    let value = table.remove(key).unwrap_or(Value::Array(Vec::new()));
    if let Some(other) = table.keys().next() {
        return Err(format!("unknown key `{other}` beside `{key}`"));
    }
    let Value::Array(items) = value else {
        return Err(format!(
            "`{key}` holds steps; write them as [[...{key}]] tables"
        ));
    };
    if items.is_empty() {
        return Err(format!("`{key}` needs at least one step"));
    }
    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let Value::Table(mut table) = item else {
                return Err(format!("`{key}`: steps are tables"));
            };
            let id = match table.remove("id") {
                None => None,
                Some(Value::String(id)) => Some(id),
                Some(_) => return Err(format!("step {}: `id` must be a string", index + 1)),
            };
            let label = id.as_ref().map_or_else(
                || format!("step {}", index + 1),
                |id| format!("step `{id}`"),
            );
            let condition =
                parse_condition(table).map_err(|message| format!("{label}: {message}"))?;
            Ok(Step { id, condition })
        })
        .collect()
}

/// Checks that step ids are unique across the blocker and every
/// `at-least = { step = ... }` names a step that surely held when the reference
/// runs, and that finds a tag.
///
/// A step surely held when it is an earlier step of the same `all`, or of an
/// enclosing `all` reached through `all`s alone. An `any` may resolve through
/// another step, so what one of its steps registers is visible within that
/// step only.
pub(crate) fn check_steps(condition: &Condition) -> Result<(), String> {
    Walk::default().condition(condition)
}

/// The state of the walk over a blocker's steps.
#[derive(Default)]
struct Walk {
    /// Every step id met so far, wherever it sits.
    seen: HashSet<String>,
    /// The steps that surely held where the walk stands, each with whether it
    /// finds a tag.
    held: Vec<(String, bool)>,
}

impl Walk {
    fn condition(&mut self, condition: &Condition) -> Result<(), String> {
        match condition {
            Condition::All(steps) => steps.iter().try_for_each(|step| self.step(step)),
            Condition::Any(steps) => steps.iter().try_for_each(|step| {
                let mark = self.held.len();
                let result = self.step(step);
                self.held.truncate(mark);
                result
            }),
            Condition::Version(VersionCondition {
                at_least: Bound::Step(name),
                ..
            }) => self.reference(name),
            Condition::State(_) | Condition::Reached(_) | Condition::Version(_) => Ok(()),
        }
    }

    fn step(&mut self, step: &Step) -> Result<(), String> {
        self.condition(&step.condition)?;
        if let Some(id) = &step.id {
            if !self.seen.insert(id.clone()) {
                return Err(format!("duplicate step id `{id}`"));
            }
            let finds_tag = matches!(
                &step.condition,
                Condition::Reached(ReachedCondition {
                    within: Within::Tags { .. },
                    ..
                })
            );
            self.held.push((id.clone(), finds_tag));
        }
        Ok(())
    }

    fn reference(&self, name: &str) -> Result<(), String> {
        match self.held.iter().find(|(held, _)| held == name) {
            Some((_, true)) => Ok(()),
            Some((_, false)) => Err(format!(
                "step `{name}` finds no tag; point `at-least` at a `reached` step with `in = {{ tags = ... }}`"
            )),
            None if self.seen.contains(name) => Err(format!(
                "`at-least` names step `{name}`, which sits in an `any` and may not have held; put both in one `all`"
            )),
            None => Err(format!("`at-least` names no earlier step `{name}`")),
        }
    }
}

#[cfg(test)]
#[path = "condition_tests.rs"]
mod tests;
