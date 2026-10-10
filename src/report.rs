//! Printing results and choosing the exit code.

use std::fmt::Write as _;
use std::path::Path;

use anstyle::{AnsiColor, Style};
use clap::ValueEnum;
use jiff::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cache::Entry;
use crate::outcome::Status;

/// Exit code: nothing counted showed up, every check ran.
pub const EXIT_OK: u8 = 0;
/// Exit code: the manifest, the cache or the arguments are unusable.
pub const EXIT_ERROR: u8 = 1;
/// Exit code base once something counted showed up; each status adds its bit.
pub const EXIT_ACTIONABLE: u8 = 8;

const STATUS_WIDTH: usize = 8;

/// A status that can fail the run: every one but `waiting`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Actionable {
    /// Something landed: adds 1 to the exit code.
    Bumpable,
    /// An upstream fix will not land as declared: adds 2.
    Dead,
    /// A check broke: adds 4.
    Failed,
}

impl Actionable {
    /// All three: what fails a run unless `--fail-on` or `[settings] fail-on`
    /// says otherwise.
    pub const ALL: [Self; 3] = [Self::Bumpable, Self::Dead, Self::Failed];

    /// The status it counts.
    const fn status(self) -> Status {
        match self {
            Self::Bumpable => Status::Bumpable,
            Self::Dead => Status::Dead,
            Self::Failed => Status::Failed,
        }
    }

    /// Its bit in the exit code.
    const fn bit(self) -> u8 {
        match self {
            Self::Bumpable => 1,
            Self::Dead => 2,
            Self::Failed => 4,
        }
    }
}

/// The exit code for a set of results: [`EXIT_OK`] when no `counted` status
/// showed up, else [`EXIT_ACTIONABLE`] plus the bit of each that did.
#[must_use]
pub fn exit_code(entries: &[Entry], counted: &[Actionable]) -> u8 {
    let bits = counted
        .iter()
        .filter(|kind| entries.iter().any(|entry| entry.status == kind.status()))
        .fold(0, |bits, kind| bits | kind.bit());
    if bits == 0 {
        EXIT_OK
    } else {
        EXIT_ACTIONABLE | bits
    }
}

/// How long ago, as the largest whole unit among days, hours and minutes.
const AGE_UNITS: [(i64, &str); 3] = [(86_400, "d"), (3_600, "h"), (60, "m")];

/// One line per entry: status, id, detail. A bumpable or dead entry adds its
/// note, the workaround to drop or rethink; a stale one adds how long ago it
/// was checked.
///
/// The status word carries ANSI styles. Write the text through an
/// [`anstream::AutoStream`], which keeps them on a terminal and strips them
/// everywhere else.
#[must_use]
pub fn lines(entries: &[Entry], now: Timestamp) -> String {
    let id_width = entries
        .iter()
        .map(|entry| entry.id.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for entry in entries {
        let style = style(entry.status);
        let label = entry.status.label();
        let pad = STATUS_WIDTH.saturating_sub(label.chars().count());
        let mut line = format!(
            "{style}{label}{style:#}{:pad$}  {:id_width$}  {}",
            "", entry.id, entry.detail
        );
        if matches!(entry.status, Status::Bumpable | Status::Dead) {
            let _ = write!(line, " ({})", entry.note);
        }
        if is_stale(entry, now) {
            let _ = write!(line, " (stale, checked {})", age(now, entry.checked_at));
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// The style of a status word: what needs you stands out, waiting recedes.
const fn style(status: Status) -> Style {
    match status {
        Status::Waiting => Style::new().dimmed(),
        Status::Bumpable => AnsiColor::Green.on_default(),
        Status::Dead => AnsiColor::Magenta.on_default(),
        Status::Failed => AnsiColor::Red.on_default(),
    }
}

/// `7h ago`, or `just now` under a minute.
fn age(now: Timestamp, then: Timestamp) -> String {
    let seconds = now.duration_since(then).as_secs();
    AGE_UNITS
        .iter()
        .find(|(size, _)| seconds >= *size)
        .map_or_else(
            || "just now".to_owned(),
            |(size, unit)| format!("{}{unit} ago", seconds / size),
        )
}

fn is_stale(entry: &Entry, now: Timestamp) -> bool {
    entry.status != Status::Failed && now >= entry.fresh_until
}

/// The `--json` document.
#[derive(Debug, Serialize)]
pub struct Report<'a> {
    /// The manifest checked.
    pub manifest: &'a Path,
    /// The state file holding these results.
    pub cache: &'a Path,
    /// The process exit code these results map to.
    pub exit_code: u8,
    /// One row per blocker.
    pub entries: Vec<Row<'a>>,
}

/// One blocker in the `--json` document.
#[derive(Debug, Serialize)]
pub struct Row<'a> {
    /// The blocker id.
    pub id: &'a str,
    /// The blocker note.
    pub note: &'a str,
    /// What the check found.
    pub status: Status,
    /// Why.
    pub detail: &'a str,
    /// When it was checked.
    pub checked_at: Timestamp,
    /// Until when the result is reused.
    pub fresh_until: Timestamp,
    /// Past `fresh_until`.
    pub stale: bool,
}

impl<'a> Report<'a> {
    /// The document for these results and the `exit_code` the run ends with.
    #[must_use]
    pub fn new(
        manifest: &'a Path,
        cache: &'a Path,
        entries: &'a [Entry],
        now: Timestamp,
        exit_code: u8,
    ) -> Self {
        Self {
            manifest,
            cache,
            exit_code,
            entries: entries
                .iter()
                .map(|entry| Row {
                    id: &entry.id,
                    note: &entry.note,
                    status: entry.status,
                    detail: &entry.detail,
                    checked_at: entry.checked_at,
                    fresh_until: entry.fresh_until,
                    stale: is_stale(entry, now),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
