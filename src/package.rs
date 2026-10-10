//! Package references: `repology:<project>` and `deps.dev:<system>/<name>`.

use std::fmt;

use crate::refs::{RefError, split_scheme};

/// Which package index answers for a [`PackageRef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexKind {
    /// repology.org: distro repos, incl. nixpkgs channels.
    Repology,
    /// deps.dev: registry packages.
    DepsDev,
}

/// deps.dev systems.
pub const DEPS_DEV_SYSTEMS: &[&str] = &["cargo", "npm", "pypi", "go", "maven", "nuget", "rubygems"];

/// A package in an index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageRef {
    /// A Repology project in one repo.
    Repology {
        /// The Repology project name.
        project: String,
        /// The repo, e.g. `nix_unstable`.
        repo: String,
        /// Only packages with this source name.
        srcname: Option<String>,
    },
    /// A registry package on deps.dev.
    DepsDev {
        /// `cargo`, `npm`, `pypi`, `go`, `maven`, `nuget` or `rubygems`.
        system: String,
        /// The package name as the registry spells it.
        name: String,
    },
}

impl PackageRef {
    /// Parses `repology:<project>` (with `repo`, optionally `srcname`) or
    /// `deps.dev:<system>/<name>`.
    ///
    /// # Errors
    ///
    /// [`RefError`] naming what is wrong.
    pub fn parse(
        text: &str,
        repo: Option<String>,
        srcname: Option<String>,
    ) -> Result<Self, RefError> {
        let (scheme, locator) = split_scheme(text)?;
        let invalid = |message: &str| RefError::Invalid(format!("`{text}` {message}"));
        match scheme {
            "repology" => {
                if locator.is_empty() {
                    return Err(RefError::Malformed {
                        text: text.to_owned(),
                        expected: "repology:<project>",
                    });
                }
                let repo =
                    repo.ok_or_else(|| invalid("needs `repo`, e.g. repo = \"nix_unstable\""))?;
                Ok(Self::Repology {
                    project: locator.to_owned(),
                    repo,
                    srcname,
                })
            }
            "deps.dev" => {
                if repo.is_some() {
                    return Err(invalid("takes no `repo`"));
                }
                if srcname.is_some() {
                    return Err(invalid("takes no `srcname`"));
                }
                let (system, name) = locator
                    .split_once('/')
                    .filter(|(_, name)| !name.is_empty())
                    .ok_or_else(|| RefError::Malformed {
                        text: text.to_owned(),
                        expected: "deps.dev:<system>/<name>",
                    })?;
                let system = system.to_ascii_lowercase();
                if !DEPS_DEV_SYSTEMS.contains(&system.as_str()) {
                    return Err(invalid(&format!(
                        "names system `{system}`; known: {}",
                        DEPS_DEV_SYSTEMS.join(", ")
                    )));
                }
                Ok(Self::DepsDev {
                    system,
                    name: name.to_owned(),
                })
            }
            other => Err(RefError::UnknownScheme {
                scheme: other.to_owned(),
                text: text.to_owned(),
                known: "repology, deps.dev",
            }),
        }
    }

    /// The index that answers for it.
    #[must_use]
    pub const fn index(&self) -> IndexKind {
        match self {
            Self::Repology { .. } => IndexKind::Repology,
            Self::DepsDev { .. } => IndexKind::DepsDev,
        }
    }
}

impl fmt::Display for PackageRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repology { project, repo, .. } => {
                write!(formatter, "repology:{project} in {repo}")
            }
            Self::DepsDev { system, name } => write!(formatter, "deps.dev:{system}/{name}"),
        }
    }
}

#[cfg(test)]
#[path = "package_tests.rs"]
mod tests;
