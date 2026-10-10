//! The JSON Schema for `canibump.toml`, generated from the manifest types.
//!
//! The manifest is read by hand: a blocker is a table whose condition is
//! picked by the key present. [`BlockerSchema`] and [`StepSchema`] describe
//! that shape for schemars, and compose the same condition types the parser
//! deserializes into, so a key added there reaches the schema with no second
//! edit.

#![expect(
    dead_code,
    reason = "fields exist only to describe the TOML shape to schemars"
)]

use schemars::{JsonSchema, schema_for};

use crate::condition::{RawReached, RawState, RawVersion};
use crate::manifest::RawManifest;

/// One workaround waiting on something upstream: an `id`, a `note` and
/// exactly one condition.
///
/// One variant per condition key. The condition's own type forbids unknown
/// keys, so `id` and `note` join it per variant instead of beside an `anyOf`
/// that would reject them.
#[derive(JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "Blocker")]
pub(crate) enum BlockerSchema {
    /// Waits on an issue, PR or MR.
    State(BlockerOf<RawState>),
    /// Waits for a commit to land in a branch or tag.
    Reached(BlockerOf<RawReached>),
    /// Waits for a package version.
    Version(BlockerOf<RawVersion>),
    /// Every step holds, checked in order.
    All(BlockerOf<AllSchema>),
    /// Some step holds, checked in order.
    Any(BlockerOf<AnySchema>),
}

/// A blocker with its condition `C`.
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Blocker_{C}")]
pub(crate) struct BlockerOf<C> {
    /// Unique name.
    id: String,
    /// What workaround it guards, and where.
    note: String,
    #[serde(flatten)]
    condition: C,
}

/// One step of an `all` or `any`: an optional `id` and exactly one condition.
#[derive(JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "Step")]
pub(crate) enum StepSchema {
    /// Waits on an issue, PR or MR.
    State(StepOf<RawState>),
    /// Waits for a commit to land in a branch or tag.
    Reached(StepOf<RawReached>),
    /// Waits for a package version.
    Version(StepOf<RawVersion>),
    /// Every step holds, checked in order.
    All(StepOf<AllSchema>),
    /// Some step holds, checked in order.
    Any(StepOf<AnySchema>),
}

/// A step with its condition `C`.
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Step_{C}")]
pub(crate) struct StepOf<C> {
    /// A name later steps refer to, e.g. `at-least = { step = "released" }`.
    id: Option<String>,
    #[serde(flatten)]
    condition: C,
}

/// `[[blocker.all]]`: every step holds.
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AllCondition")]
pub(crate) struct AllSchema {
    /// The steps, checked in order.
    all: Vec<StepSchema>,
}

/// `[[blocker.any]]`: some step holds.
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AnyCondition")]
pub(crate) struct AnySchema {
    /// The steps, checked in order.
    any: Vec<StepSchema>,
}

/// The schema as pretty JSON with a trailing newline.
#[must_use]
pub fn render() -> String {
    format!("{:#}\n", schema_for!(RawManifest).as_value())
}
