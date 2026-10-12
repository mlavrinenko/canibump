//! `[scan]`: upstream links in the repo's files that no blocker lists, and
//! the `canibump:<id>` markers that place each blocker's workaround.
//!
//! A catcher is a regex whose named groups fill a `ref` template, so a link
//! turns into the manifest ref a blocker would name. The built-ins catch
//! GitHub, GitLab and Codeberg links; a `[[scan.catcher]]` adds one, replaces
//! a built-in by its id, or turns one off.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use glob::{MatchOptions, Pattern};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::condition::{Condition, ReachedCondition, StateCondition};
use crate::manifest::Manifest;
use crate::refs::Reachable;

/// A line holding this is never reported, and marks nothing.
const IGNORE: &str = "canibump-ignore";

/// The catcher an unlisted marker is reported under.
const MARKER: &str = "marker";

/// `canibump:<id>` as a word of its own marks a site of blocker `<id>`.
const MARKER_PATTERN: &str = r"\bcanibump:(?<id>[A-Za-z0-9_-]+)";

/// `*` stops at `/`, so `*.nix` is the top directory and `**/*.nix` all of them.
const MATCH: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// The catchers on unless a `[[scan.catcher]]` of the same id says otherwise:
/// id, pattern, ref.
const BUILT_IN: &[(&str, &str, &str)] = &[
    (
        "github",
        r"https?://github\.com/(?<owner>[\w.-]+)/(?<repo>[\w.-]+)/(?:issues|pull)/(?<number>\d+)",
        "github:${owner}/${repo}#${number}",
    ),
    (
        "gitlab-issue",
        r"https?://gitlab\.com/(?<project>[\w.-]+(?:/[\w.-]+)+)/-/issues/(?<number>\d+)",
        "gitlab:${project}#${number}",
    ),
    (
        "gitlab-mr",
        r"https?://gitlab\.com/(?<project>[\w.-]+(?:/[\w.-]+)+)/-/merge_requests/(?<number>\d+)",
        "gitlab:${project}!${number}",
    ),
    (
        "codeberg",
        r"https?://codeberg\.org/(?<owner>[\w.-]+)/(?<repo>[\w.-]+)/(?:issues|pulls)/(?<number>\d+)",
        "codeberg:${owner}/${repo}#${number}",
    ),
];

/// `[scan]`, checked.
#[derive(Debug, Clone)]
pub struct Scan {
    /// Which files to read, relative to the manifest.
    paths: Vec<Pattern>,
    /// Which of those to skip.
    exclude: Vec<Pattern>,
    /// Directories to skip whole: each `exclude` ending in `/**`, minus it.
    pruned: Vec<Pattern>,
    /// `canibump:<id>`.
    marker: Regex,
    /// The catchers that are on, built-ins first.
    catchers: Vec<Catcher>,
}

impl PartialEq for Scan {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths
            && self.exclude == other.exclude
            && self.catchers == other.catchers
    }
}

impl Eq for Scan {}

impl Scan {
    /// Whether an `exclude` ending in `/**` skips the whole directory `dir`.
    fn prunes(&self, dir: &Path) -> bool {
        self.pruned
            .iter()
            .any(|skip| skip.matches_path_with(dir, MATCH))
    }

    /// Whether `paths` takes the file and `exclude` leaves it.
    fn wants(&self, file: &Path) -> bool {
        let matches = |pattern: &Pattern| pattern.matches_path_with(file, MATCH);
        self.paths.iter().any(matches) && !self.exclude.iter().any(matches)
    }
}

/// A regex that turns a link into a manifest ref.
#[derive(Debug, Clone)]
struct Catcher {
    id: String,
    pattern: Regex,
    template: String,
}

impl PartialEq for Catcher {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.pattern.as_str() == other.pattern.as_str()
            && self.template == other.template
    }
}

/// Reads the repo's files for `canibump:<id>` markers, each a site of blocker
/// `<id>`, and for upstream links. A link or marker no blocker names prints as
/// an `unlisted` line; a line holding `canibump-ignore` is skipped, and so is
/// the manifest.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Scan")]
pub(crate) struct RawScan {
    /// Globs of the files to read, relative to the manifest, like
    /// `"**/*.nix"`. `*` stops at `/`, `**` crosses directories.
    paths: Vec<String>,
    /// Globs of files to skip among those.
    #[serde(default)]
    exclude: Vec<String>,
    /// Catchers to add, replace or turn off. The built-ins, `github`,
    /// `gitlab-issue`, `gitlab-mr` and `codeberg`, are on by default.
    #[serde(default, rename = "catcher")]
    catchers: Vec<RawCatcher>,
}

/// A catcher: a regex over each line, and the ref its match stands for.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Catcher")]
struct RawCatcher {
    /// Its name. A built-in's id replaces or turns off that built-in.
    id: String,
    /// A regex with named groups, like
    /// `'https://jira\.example\.com/browse/(?<key>[A-Z]+-\d+)'`.
    pattern: Option<String>,
    /// The ref a match stands for, with `${group}` for each named group, like
    /// `"jira:jira.example.com#${key}"`.
    #[serde(rename = "ref")]
    reference: Option<String>,
    /// `false` turns it off. Defaults to `true`.
    #[serde(default = "on")]
    enabled: bool,
}

const fn on() -> bool {
    true
}

impl RawScan {
    /// Checks the globs and catchers.
    pub(crate) fn parse(self) -> Result<Scan, String> {
        if self.paths.is_empty() {
            return Err("scan.paths: list at least one glob of files to read".to_owned());
        }
        let pruned: Vec<String> = self
            .exclude
            .iter()
            .filter_map(|text| text.strip_suffix("/**"))
            .map(str::to_owned)
            .collect();
        Ok(Scan {
            paths: globs("paths", &self.paths)?,
            exclude: globs("exclude", &self.exclude)?,
            pruned: globs("exclude", &pruned)?,
            marker: Regex::new(MARKER_PATTERN).map_err(|error| format!("scan: {error}"))?,
            catchers: catchers(self.catchers)?,
        })
    }
}

fn globs(key: &str, texts: &[String]) -> Result<Vec<Pattern>, String> {
    texts
        .iter()
        .map(|text| {
            Pattern::new(text)
                .map_err(|error| format!("scan.{key}: `{text}` is not a glob: {error}"))
        })
        .collect()
}

/// The built-ins, then each `[[scan.catcher]]` applied in turn.
fn catchers(raws: Vec<RawCatcher>) -> Result<Vec<Catcher>, String> {
    let mut list: Vec<Option<Catcher>> = BUILT_IN
        .iter()
        .map(|(id, pattern, template)| catcher(id, pattern, template).map(Some))
        .collect::<Result<_, _>>()?;
    let mut seen = HashSet::new();
    for raw in raws {
        if !seen.insert(raw.id.clone()) {
            return Err(format!("scan.catcher `{}`: listed twice", raw.id));
        }
        let built_in = BUILT_IN.iter().position(|(id, ..)| *id == raw.id);
        let made = custom(&raw, built_in.is_some())?;
        let made = made.filter(|_| raw.enabled);
        match built_in.and_then(|index| list.get_mut(index)) {
            Some(slot) if made.is_some() || !raw.enabled => *slot = made,
            Some(_) => {}
            None => list.push(made),
        }
    }
    Ok(list.into_iter().flatten().collect())
}

/// A `[[scan.catcher]]`'s own catcher: `None` when it only toggles a built-in.
fn custom(raw: &RawCatcher, built_in: bool) -> Result<Option<Catcher>, String> {
    let fail = |message: &str| format!("scan.catcher `{}`: {message}", raw.id);
    match (&raw.pattern, &raw.reference) {
        (Some(pattern), Some(template)) => catcher(&raw.id, pattern, template).map(Some),
        (Some(_), None) => Err(fail("a `pattern` needs a `ref`")),
        (None, Some(_)) => Err(fail("a `ref` needs a `pattern`")),
        (None, None) if built_in => Ok(None),
        (None, None) => Err(fail("not a built-in, so give it a `pattern` and a `ref`")),
    }
}

fn catcher(id: &str, pattern: &str, template: &str) -> Result<Catcher, String> {
    let fail = |message: String| format!("scan.catcher `{id}`: {message}");
    let pattern = Regex::new(pattern).map_err(|error| fail(format!("bad `pattern`: {error}")))?;
    let names: HashSet<&str> = pattern.capture_names().flatten().collect();
    let groups = template
        .split("${")
        .skip(1)
        .filter_map(|rest| rest.split_once('}'));
    for (name, _) in groups {
        let numbered = name
            .parse::<usize>()
            .is_ok_and(|n| n < pattern.captures_len());
        if !names.contains(name) && !numbered {
            return Err(fail(format!(
                "`ref` names `${{{name}}}`, which `pattern` has no group for"
            )));
        }
    }
    Ok(Catcher {
        id: id.to_owned(),
        pattern,
        template: template.to_owned(),
    })
}

/// An upstream link no blocker lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The file, relative to the manifest's directory.
    pub file: PathBuf,
    /// The 1-based line.
    pub line: usize,
    /// The link as a manifest ref.
    pub reference: String,
    /// The catcher that found it.
    pub catcher: String,
}

/// A line of a scanned file holding a blocker's marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Site {
    /// The file, relative to the manifest's directory.
    pub file: PathBuf,
    /// The 1-based line.
    pub line: usize,
}

/// What `[scan]` found: links and markers no blocker lists, and each
/// blocker's sites.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Findings {
    /// Links and markers no blocker lists, by file and line.
    pub hits: Vec<Hit>,
    /// Each marked blocker's sites, by file and line.
    pub sites: BTreeMap<String, Vec<Site>>,
}

/// What `manifest`'s `[scan]` finds. A file that is not UTF-8 is skipped;
/// one that cannot be read, or a directory that cannot be listed, earns a
/// warning on stderr.
#[must_use]
pub fn find(manifest: &Manifest, manifest_path: &Path) -> Findings {
    let mut findings = Findings::default();
    let Some(scan) = &manifest.scan else {
        return findings;
    };
    let lookout = Lookout {
        scan,
        listed: listed(manifest),
        ids: manifest
            .blockers
            .iter()
            .map(|one| one.id.as_str())
            .collect(),
    };
    let root = manifest_path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    for file in files(scan, root, manifest_path) {
        let text = read(&root.join(&file));
        lookout.read(&file, &text, &mut findings);
    }
    findings
}

/// What a scan looks for, and what the manifest already names.
struct Lookout<'a> {
    scan: &'a Scan,
    /// Every ref the blockers name, in the form a hit is compared in.
    listed: HashSet<String>,
    /// Every blocker id.
    ids: HashSet<&'a str>,
}

/// Every ref the blockers name, in the form a hit is compared in.
fn listed(manifest: &Manifest) -> HashSet<String> {
    let mut refs = HashSet::new();
    for blocker in &manifest.blockers {
        collect(&blocker.condition, &mut refs);
    }
    refs
}

fn collect(condition: &Condition, refs: &mut HashSet<String>) {
    match condition {
        Condition::State(StateCondition { item }) => {
            refs.insert(item.to_string().to_lowercase());
        }
        Condition::Reached(ReachedCondition { target, .. }) => {
            refs.insert(target.to_string().to_lowercase());
        }
        Condition::Version(_) => {}
        Condition::All(steps) | Condition::Any(steps) => {
            for step in steps {
                collect(&step.condition, refs);
            }
        }
    }
}

/// The files `paths` matches minus `exclude` and the manifest, relative to
/// `root` and sorted.
fn files(scan: &Scan, root: &Path, manifest_path: &Path) -> BTreeSet<PathBuf> {
    let manifest = manifest_path.canonicalize().ok();
    let mut found = BTreeSet::new();
    let mut dirs = vec![PathBuf::new()];
    while let Some(dir) = dirs.pop() {
        let entries = match std::fs::read_dir(root.join(&dir)) {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!(
                    "canibump: warning: scan: cannot list {}: {error}",
                    root.join(&dir).display()
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let relative = dir.join(entry.file_name());
            // A symlink's own type: a link to a directory is not one, so the
            // walk never leaves the tree, say into /nix/store.
            let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
            if is_dir {
                if relative != Path::new(".git") && !scan.prunes(&relative) {
                    dirs.push(relative);
                }
            } else if scan.wants(&relative)
                && root.join(&relative).is_file()
                && root.join(&relative).canonicalize().ok() != manifest
            {
                found.insert(relative);
            }
        }
    }
    found
}

/// The file's text, or nothing when it is not UTF-8 or cannot be read.
fn read(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => String::new(),
        Err(error) => {
            eprintln!(
                "canibump: warning: scan: cannot read {}: {error}",
                path.display()
            );
            String::new()
        }
    }
}

/// An unlisted link or marker on a line: its column, ref and catcher.
type Spot<'a> = (usize, String, &'a str);

impl Lookout<'_> {
    /// Adds the unlisted links and markers in `text` to `findings`, in line
    /// order, then column order, each ref once per line; and the sites of
    /// the markers some blocker has.
    fn read(&self, file: &Path, text: &str, findings: &mut Findings) {
        for (index, line) in text.lines().enumerate() {
            if line.contains(IGNORE) {
                continue;
            }
            let site = Site {
                file: file.to_owned(),
                line: index + 1,
            };
            let mut on_line = self.links(line);
            on_line.extend(self.markers(line, &site, &mut findings.sites));
            on_line.sort_by_key(|(start, ..)| *start);
            let mut seen = HashSet::new();
            for (_, reference, catcher) in on_line {
                if seen.insert(reference.clone()) {
                    findings.hits.push(Hit {
                        file: site.file.clone(),
                        line: site.line,
                        reference,
                        catcher: catcher.to_owned(),
                    });
                }
            }
        }
    }

    /// The links on `line` no blocker lists.
    fn links(&self, line: &str) -> Vec<Spot<'_>> {
        self.scan
            .catchers
            .iter()
            .flat_map(|catcher| {
                catcher.pattern.captures_iter(line).map(|captures| {
                    let mut raw = String::new();
                    captures.expand(&catcher.template, &mut raw);
                    let start = captures.get(0).map_or(0, |whole| whole.start());
                    (start, canonical(raw), catcher.id.as_str())
                })
            })
            .filter(|(_, reference, _)| !self.listed.contains(&reference.to_lowercase()))
            .collect()
    }

    /// The markers on `line` no blocker has. Each one some blocker has adds
    /// `site` to that blocker's `sites`, once.
    fn markers(
        &self,
        line: &str,
        site: &Site,
        sites: &mut BTreeMap<String, Vec<Site>>,
    ) -> Vec<Spot<'static>> {
        let mut unknown = Vec::new();
        for marker in self.scan.marker.captures_iter(line) {
            let (Some(whole), Some(id)) = (marker.get(0), marker.name("id")) else {
                continue;
            };
            if !self.ids.contains(id.as_str()) {
                unknown.push((whole.start(), whole.as_str().to_owned(), MARKER));
                continue;
            }
            let known = sites.entry(id.as_str().to_owned()).or_default();
            if known.last() != Some(site) {
                known.push(site.clone());
            }
        }
        unknown
    }
}

/// The ref as the manifest prints it, so `gitlab:gitlab.com/g/p#1` and
/// `gitlab:g/p#1` compare equal; as written when it is no ref at all.
fn canonical(raw: String) -> String {
    raw.parse::<Reachable>()
        .map_or(raw, |reference| reference.to_string())
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
