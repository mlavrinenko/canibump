//! Printing results and choosing the exit code.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use anstyle::{AnsiColor, Style};
use clap::ValueEnum;
use jiff::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cache::Entry;
use crate::outcome::Status;
use crate::scan::{Findings, Site};

/// Exit code: nothing counted showed up, every check ran.
pub const EXIT_OK: u8 = 0;
/// Exit code: the manifest, the cache or the arguments are unusable.
pub const EXIT_ERROR: u8 = 1;
/// Exit code base once something counted showed up; each status adds its bit.
pub const EXIT_ACTIONABLE: u8 = 8;

const STATUS_WIDTH: usize = 8;

/// What can fail the run: every status but `waiting`, an unlisted link or
/// marker, and an unmarked blocker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Actionable {
    /// Something landed: adds 1 to the exit code.
    Bumpable,
    /// An upstream fix will not land as declared: adds 2.
    Dead,
    /// A check broke: adds 4.
    Failed,
    /// `[scan]` found a link or marker no blocker names: adds 16.
    Unlisted,
    /// `[scan]` found no `canibump:<id>` for a blocker: adds 32.
    Unmarked,
}

impl Actionable {
    /// All five: what fails a run unless `--fail-on` or `[settings] fail-on`
    /// says otherwise.
    pub const ALL: [Self; 5] = [
        Self::Bumpable,
        Self::Dead,
        Self::Failed,
        Self::Unlisted,
        Self::Unmarked,
    ];

    /// Whether it showed up among the `entries` or in what `[scan]` found.
    fn showed_up(self, entries: &[Entry], findings: &Findings) -> bool {
        let status = match self {
            Self::Bumpable => Status::Bumpable,
            Self::Dead => Status::Dead,
            Self::Failed => Status::Failed,
            Self::Unlisted => return !findings.hits.is_empty(),
            Self::Unmarked => return !findings.unmarked.is_empty(),
        };
        entries.iter().any(|entry| entry.status == status)
    }

    /// Its bit in the exit code.
    const fn bit(self) -> u8 {
        match self {
            Self::Bumpable => 1,
            Self::Dead => 2,
            Self::Failed => 4,
            Self::Unlisted => 16,
            Self::Unmarked => 32,
        }
    }
}

/// The exit code for a set of results and what `[scan]` found: [`EXIT_OK`]
/// when nothing `counted` showed up, else [`EXIT_ACTIONABLE`] plus the bit of
/// each that did.
#[must_use]
pub fn exit_code(entries: &[Entry], findings: &Findings, counted: &[Actionable]) -> u8 {
    let bits = counted
        .iter()
        .filter(|kind| kind.showed_up(entries, findings))
        .fold(0, |bits, kind| bits | kind.bit());
    if bits == 0 {
        EXIT_OK
    } else {
        EXIT_ACTIONABLE | bits
    }
}

/// `status <id>` exit code: the answer is not yes or no. Dead, failed, an
/// entry that is absent, stale or unreadable, and every error land here, since
/// the usual [`EXIT_ERROR`] would read as "waiting".
pub const EXIT_UNKNOWN: u8 = 2;

/// The `status <id>` exit code for a blocker's status: 0 bumpable, 1 waiting,
/// [`EXIT_UNKNOWN`] for the rest.
#[must_use]
pub const fn blocker_exit_code(status: Status) -> u8 {
    match status {
        Status::Bumpable => 0,
        Status::Waiting => 1,
        Status::Dead | Status::Failed => EXIT_UNKNOWN,
    }
}

/// A failure that exits with its own `code` instead of [`EXIT_ERROR`]. It
/// prints as the error it wraps.
#[derive(Debug, Error)]
#[error("{error:#}")]
pub struct Exit {
    /// The process exit code.
    pub code: u8,
    error: anyhow::Error,
}

impl Exit {
    /// `error`, ending the process with `code`.
    pub fn new(code: u8, error: impl Into<anyhow::Error>) -> Self {
        Self {
            code,
            error: error.into(),
        }
    }

    /// The exit code `error` asks for: its own when it is an [`Exit`], else
    /// [`EXIT_ERROR`].
    #[must_use]
    pub fn code_of(error: &anyhow::Error) -> u8 {
        error
            .downcast_ref::<Self>()
            .map_or(EXIT_ERROR, |exit| exit.code)
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
pub fn lines(entries: &[Entry], sites: &BTreeMap<String, Vec<Site>>, now: Timestamp) -> String {
    let id_width = widest(entries.iter().map(|entry| &entry.id));
    let mut out = String::new();
    for entry in entries {
        let style = style(entry.status);
        let label = entry.status.label();
        let pad = STATUS_WIDTH.saturating_sub(label.chars().count());
        let mut line = format!(
            "{style}{label}{style:#}{:pad$}  {:id_width$}  {}",
            "", entry.id, entry.detail
        );
        let acts = matches!(entry.status, Status::Bumpable | Status::Dead);
        if acts {
            let _ = write!(line, " ({})", entry.note);
        }
        if is_stale(entry, now) {
            let _ = write!(line, " (stale, checked {})", age(now, entry.checked_at));
        }
        out.push_str(line.trim_end());
        out.push('\n');
        if let Some(known) = sites.get(&entry.id).filter(|_| acts) {
            let _ = writeln!(out, "{:STATUS_WIDTH$}  at {}", "", places(known));
        }
    }
    out
}

/// What `[scan]` found: one `unlisted` line per hit, where and the ref no
/// blocker names, then one `unmarked` line per blocker no marker names.
///
/// The status word carries ANSI styles, like [`lines`].
#[must_use]
pub fn scan_lines(findings: &Findings) -> String {
    let places: Vec<String> = findings
        .hits
        .iter()
        .map(|hit| format!("{}:{}", hit.file.display(), hit.line))
        .collect();
    let width = widest(places.iter());
    let style = AnsiColor::Yellow.on_default();
    let mut out = String::new();
    for (hit, place) in findings.hits.iter().zip(&places) {
        let _ = writeln!(
            out,
            "{style}unlisted{style:#}  {place:width$}  {} has no blocker",
            hit.reference
        );
    }
    let width = widest(findings.unmarked.iter());
    for id in &findings.unmarked {
        let _ = writeln!(
            out,
            "{style}unmarked{style:#}  {id:width$}  no canibump:{id} in the scanned files"
        );
    }
    out
}

/// The width of the widest of `texts`, in chars.
fn widest<'a>(texts: impl Iterator<Item = &'a String>) -> usize {
    texts.map(|text| text.chars().count()).max().unwrap_or(0)
}

/// `flake.nix:29, Justfile:26`.
fn places(sites: &[Site]) -> String {
    let places: Vec<String> = sites
        .iter()
        .map(|site| format!("{}:{}", site.file.display(), site.line))
        .collect();
    places.join(", ")
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
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report<'a> {
    /// The manifest checked.
    pub manifest: &'a Path,
    /// The state file holding these results.
    pub cache: &'a Path,
    /// The process exit code these results map to.
    pub exit_code: u8,
    /// One row per blocker.
    pub entries: Vec<Row<'a>>,
    /// One row per upstream link or marker `[scan]` found that no blocker
    /// names.
    pub unlisted: Vec<Unlisted<'a>>,
    /// The blockers `[scan]` found no `canibump:<id>` for.
    pub unmarked: Vec<&'a str>,
}

/// An upstream link no blocker lists, in the `--json` document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Unlisted<'a> {
    /// The file, relative to the manifest's directory.
    pub file: &'a Path,
    /// The 1-based line.
    pub line: usize,
    /// The link as a manifest ref.
    #[serde(rename = "ref")]
    pub reference: &'a str,
    /// The catcher that found it.
    pub catcher: &'a str,
}

/// One blocker in the `--json` document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Row<'a> {
    /// The blocker id.
    pub id: &'a str,
    /// The blocker note.
    pub note: &'a str,
    /// What the check found.
    pub status: Status,
    /// Why.
    pub detail: &'a str,
    /// When it was checked, RFC 3339.
    #[schemars(with = "String")]
    pub checked_at: Timestamp,
    /// Until when the result is reused, RFC 3339.
    #[schemars(with = "String")]
    pub fresh_until: Timestamp,
    /// Where its workaround sits: each `canibump:<id>` `[scan]` found.
    pub sites: &'a [Site],
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
                    sites: &[],
                    stale: is_stale(entry, now),
                })
                .collect(),
            unlisted: Vec::new(),
            unmarked: Vec::new(),
        }
    }

    /// The same document with what `[scan]` found: each row's sites, and the
    /// links and markers no blocker names.
    #[must_use]
    pub fn with_scan(self, findings: &'a Findings) -> Self {
        let mut entries = self.entries;
        for row in &mut entries {
            row.sites = findings.sites.get(row.id).map_or(&[], Vec::as_slice);
        }
        let unlisted = &findings.hits;
        Self {
            unlisted: unlisted
                .iter()
                .map(|hit| Unlisted {
                    file: &hit.file,
                    line: hit.line,
                    reference: &hit.reference,
                    catcher: &hit.catcher,
                })
                .collect(),
            unmarked: findings.unmarked.iter().map(String::as_str).collect(),
            entries,
            ..self
        }
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
