//! Printing results and choosing the exit code.

use std::fmt::Write as _;
use std::path::Path;

use jiff::Timestamp;
use serde::Serialize;

use crate::cache::Entry;
use crate::outcome::Status;

/// Exit code: nothing bumpable, every check ran.
pub const EXIT_OK: u8 = 0;
/// Exit code: the manifest, the cache or the arguments are unusable.
pub const EXIT_ERROR: u8 = 1;
/// Exit code: at least one blocker is bumpable.
pub const EXIT_BUMPABLE: u8 = 10;
/// Exit code: at least one check failed, nothing bumpable.
pub const EXIT_FAILED: u8 = 11;
/// Exit code: something is bumpable and some check failed.
pub const EXIT_BUMPABLE_AND_FAILED: u8 = 12;

const STATUS_WIDTH: usize = 8;

/// The exit code for a set of results.
#[must_use]
pub fn exit_code(entries: &[Entry]) -> u8 {
    let has = |status| entries.iter().any(|entry| entry.status == status);
    match (has(Status::Bumpable), has(Status::Failed)) {
        (true, true) => EXIT_BUMPABLE_AND_FAILED,
        (true, false) => EXIT_BUMPABLE,
        (false, true) => EXIT_FAILED,
        (false, false) => EXIT_OK,
    }
}

/// One line per entry: status, id, detail. A bumpable entry adds its note; a
/// stale one adds when it was checked.
#[must_use]
pub fn lines(entries: &[Entry], now: Timestamp) -> String {
    let id_width = entries
        .iter()
        .map(|entry| entry.id.len())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for entry in entries {
        let mut line = format!(
            "{:STATUS_WIDTH$}  {:id_width$}  {}",
            entry.status.label(),
            entry.id,
            entry.detail
        );
        if entry.status == Status::Bumpable {
            let _ = write!(line, " ({})", entry.note);
        }
        if is_stale(entry, now) {
            let _ = write!(line, " (stale, checked {})", entry.checked_at);
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
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
    /// The document for these results.
    #[must_use]
    pub fn new(manifest: &'a Path, cache: &'a Path, entries: &'a [Entry], now: Timestamp) -> Self {
        Self {
            manifest,
            cache,
            exit_code: exit_code(entries),
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
