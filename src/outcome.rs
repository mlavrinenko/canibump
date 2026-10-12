//! What a check found.

use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The state of a blocker, or of one step of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Upstream has not landed it yet.
    Waiting,
    /// Landed: the workaround can go.
    Bumpable,
    /// Will not land as declared (closed unmerged, wontfix, duplicate).
    Dead,
    /// The check itself failed (network, auth, rate limit).
    Failed,
}

impl Status {
    /// Every status, in the order a run prints them.
    pub const ALL: [Self; 4] = [Self::Waiting, Self::Bumpable, Self::Dead, Self::Failed];

    /// The word printed for this status.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Bumpable => "bumpable",
            Self::Dead => "dead",
            Self::Failed => "failed",
        }
    }
}

/// A status with a one-line reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// What was found.
    pub status: Status,
    /// Why, in one line.
    pub detail: String,
    /// What a later step may use, e.g. the first tag that has a commit.
    pub value: Option<String>,
}

impl Outcome {
    /// An outcome from a status and its reason.
    #[must_use]
    pub fn new(status: Status, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
            value: None,
        }
    }

    /// The same outcome, carrying a value for later steps.
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}
