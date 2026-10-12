//! `sites`: where a blocker's workaround sits in a file that takes no
//! comment, like JSON, so no `canibump:<id>` marker can say so.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::manifest::Manifest;
use crate::scan::{Findings, Site};

/// A site a blocker declares: a file, and a regex run over each of its lines.
#[derive(Debug, Clone)]
pub struct Declared {
    file: PathBuf,
    pattern: Regex,
}

impl PartialEq for Declared {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && self.pattern.as_str() == other.pattern.as_str()
    }
}

impl Eq for Declared {}

/// Where the workaround sits in a file that takes no comment, so no
/// `canibump:<id>` marker can.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "DeclaredSite")]
pub(crate) struct RawSite {
    /// The file, relative to the manifest, like `"package.json"`.
    file: PathBuf,
    /// A regex run over each line; each line it matches is a site, like
    /// `'"left-pad": "=1\.3\.0"'`. Matching none prints `unmarked`.
    pattern: String,
}

/// A blocker [`crate::scan::find`] lists as unmarked, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Unmarked {
    /// The blocker id.
    pub id: String,
    /// Why, like `no canibump:<id> in the scanned files`.
    pub detail: String,
}

/// A blocker's `sites`, checked.
pub(crate) fn parse(value: Value) -> Result<Vec<Declared>, String> {
    let raws: Vec<RawSite> = value
        .try_into()
        .map_err(|error| format!("sites: {error}"))?;
    raws.into_iter()
        .map(|raw| {
            let pattern = Regex::new(&raw.pattern)
                .map_err(|error| format!("sites: bad `pattern` `{}`: {error}", raw.pattern))?;
            Ok(Declared {
                file: raw.file,
                pattern,
            })
        })
        .collect()
}

/// Adds each blocker's declared sites under `root` to `findings`, then lists
/// as unmarked each declared site that matches nothing and, with `[scan]`,
/// each blocker that declares none and has no marker. Each blocker's sites
/// end sorted by file and line.
pub(crate) fn settle(manifest: &Manifest, root: &Path, findings: &mut Findings) {
    for blocker in &manifest.blockers {
        let marked = findings.sites.contains_key(&blocker.id);
        for declared in &blocker.sites {
            match declared.lines(root) {
                Ok(lines) => findings
                    .sites
                    .entry(blocker.id.clone())
                    .or_default()
                    .extend(lines),
                Err(detail) => findings.unmarked.push(Unmarked {
                    id: blocker.id.clone(),
                    detail,
                }),
            }
        }
        if blocker.sites.is_empty() && manifest.scan.is_some() && !marked {
            findings.unmarked.push(Unmarked {
                id: blocker.id.clone(),
                detail: format!("no canibump:{} in the scanned files", blocker.id),
            });
        }
    }
    for sites in findings.sites.values_mut() {
        sites.sort();
        sites.dedup();
    }
}

impl Declared {
    /// The lines of the file under `root` the pattern matches; why not, when
    /// none does or the file cannot be read.
    fn lines(&self, root: &Path) -> Result<Vec<Site>, String> {
        let file = self.file.display();
        let text = std::fs::read_to_string(root.join(&self.file)).map_err(|error| {
            if error.kind() == ErrorKind::NotFound {
                format!("{file} is missing")
            } else {
                format!("cannot read {file}: {error}")
            }
        })?;
        let lines: Vec<Site> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| self.pattern.is_match(line))
            .map(|(index, _)| Site {
                file: self.file.clone(),
                line: index + 1,
            })
            .collect();
        if lines.is_empty() {
            return Err(format!("no line of {file} matches `{}`", self.pattern));
        }
        Ok(lines)
    }
}

#[cfg(test)]
#[path = "sites_tests.rs"]
mod tests;
