//! Command-line interface.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use clap::{Parser, Subcommand};
use jiff::Timestamp;

use crate::cache::{self, StateFile};
use crate::manifest::Manifest;
use crate::provider::Registry;
use crate::provider::git::GitRemotes;
use crate::provider::github::GitHub;
use crate::refs::SourceKind;
use crate::report::{self, Report};
use crate::run;

/// Can I bump yet? Checks the upstream blockers in canibump.toml.
///
/// Exit codes: 0 nothing bumpable, 10 something bumpable, 11 a check failed,
/// 12 both, 1 unusable manifest, cache or arguments.
#[derive(Debug, Parser)]
#[command(version, about, long_about)]
pub struct Cli {
    /// The manifest to read.
    #[arg(short, long, global = true, default_value = "canibump.toml")]
    pub manifest: PathBuf,
    /// The state file [default: `$XDG_CACHE_HOME/canibump/state/<manifest path>.json`].
    #[arg(long, global = true)]
    pub cache: Option<PathBuf>,
    /// Print JSON instead of lines.
    #[arg(long, global = true)]
    pub json: bool,
    /// Re-check every blocker, even fresh ones.
    #[arg(long)]
    pub refresh: bool,
    /// What to do; `check` when omitted.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check every blocker, reusing results younger than the TTL.
    Check {
        /// Re-check every blocker, even fresh ones.
        #[arg(long)]
        refresh: bool,
    },
    /// Print the last results from the state file, without network.
    Status {
        /// Print the state file's path and exit.
        #[arg(long)]
        path: bool,
    },
}

/// Runs the CLI and returns the exit code.
///
/// # Errors
///
/// When the manifest, the state file or stdout cannot be used.
pub fn run(cli: &Cli) -> Result<u8> {
    let manifest = Manifest::load(&cli.manifest)?;
    let cache_path = cache::resolve(
        &cli.manifest,
        manifest.settings.cache.as_deref(),
        cli.cache.as_deref(),
    )?;
    match cli.command {
        Some(Command::Status { path: true }) => {
            print(&format!("{}\n", cache_path.display()))?;
            Ok(report::EXIT_OK)
        }
        Some(Command::Status { path: false }) => status(cli, &cache_path),
        Some(Command::Check { refresh }) => {
            check(cli, &manifest, &cache_path, refresh || cli.refresh)
        }
        None => check(cli, &manifest, &cache_path, cli.refresh),
    }
}

fn check(cli: &Cli, manifest: &Manifest, cache_path: &Path, refresh: bool) -> Result<u8> {
    let previous = match cache::read(cache_path) {
        Ok(previous) => previous,
        Err(error) => {
            log::warn!("ignoring the old state file: {error}");
            None
        }
    };
    let now = Timestamp::now();
    let runtime = tokio::runtime::Runtime::new().context("starting the async runtime")?;
    let entries = runtime.block_on(run::check(
        manifest,
        previous.as_ref(),
        default_registry,
        now,
        refresh,
    ));
    let absolute = std::path::absolute(&cli.manifest).unwrap_or_else(|_| cli.manifest.clone());
    let state = StateFile {
        version: cache::VERSION,
        manifest: absolute,
        entries,
    };
    cache::write(cache_path, &state)?;
    output(cli, cache_path, &state, now)
}

fn status(cli: &Cli, cache_path: &Path) -> Result<u8> {
    let Some(state) = cache::read(cache_path)? else {
        bail!(
            "no results at {}: run `canibump` first",
            cache_path.display()
        );
    };
    output(cli, cache_path, &state, Timestamp::now())
}

fn output(cli: &Cli, cache_path: &Path, state: &StateFile, now: Timestamp) -> Result<u8> {
    let text = if cli.json {
        let report = Report::new(&state.manifest, cache_path, &state.entries, now);
        serde_json::to_string_pretty(&report)? + "\n"
    } else {
        report::lines(&state.entries, now)
    };
    print(&text)?;
    Ok(report::exit_code(&state.entries))
}

fn print(text: &str) -> Result<()> {
    std::io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .context("writing to stdout")
}

/// The real providers. `CANIBUMP_GITHUB_API` points GitHub elsewhere (tests,
/// GitHub Enterprise).
fn default_registry() -> Registry {
    let mut registry = Registry::default();
    let base = std::env::var("CANIBUMP_GITHUB_API").ok();
    if let Some(root) = cache::root() {
        let remotes = Arc::new(GitRemotes::new(root.join("git")));
        registry = registry.with_code_host(SourceKind::Git, remotes);
    }
    match GitHub::new(base.as_deref(), crate::auth::github_token()) {
        Ok(github) => {
            let github = Arc::new(github);
            registry = registry
                .with_tracker(SourceKind::GitHub, github.clone())
                .with_code_host(SourceKind::GitHub, github);
        }
        Err(error) => log::warn!("{error}"),
    }
    registry
}
