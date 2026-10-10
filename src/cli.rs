//! Command-line interface.

use anstream::AutoStream;
use anyhow::{Context as _, Result, bail};
use clap::{Args, ColorChoice, CommandFactory as _, Parser, Subcommand};
use clap_complete::Shell;
use jiff::Timestamp;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::cache::{self, StateFile};
use crate::manifest::Manifest;
use crate::provider::defaults;
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
    /// When to color the status words; `auto` honors the terminal and `NO_COLOR`.
    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,
    /// The check flags, for a bare `canibump` or ahead of `check`.
    #[command(flatten)]
    pub check: CheckArgs,
    /// What to do; `check` when omitted.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// What `check` takes, bare or as a subcommand.
#[derive(Debug, Clone, Copy, Args)]
pub struct CheckArgs {
    /// Re-check every blocker, even fresh ones.
    #[arg(long)]
    pub refresh: bool,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check every blocker, reusing results younger than the TTL.
    Check(CheckArgs),
    /// Print the last results from the state file, without network.
    Status {
        /// Print the state file's path and exit.
        #[arg(long)]
        path: bool,
    },
    /// Print the JSON Schema for canibump.toml and exit.
    Schema,
    /// Print a shell completion script and exit.
    Completions {
        /// The shell to complete for.
        shell: Shell,
    },
}

impl Command {
    /// The word that selects it.
    const fn name(&self) -> &'static str {
        match self {
            Self::Check(_) => "check",
            Self::Status { .. } => "status",
            Self::Schema => "schema",
            Self::Completions { .. } => "completions",
        }
    }
}

/// Runs the CLI and returns the exit code.
///
/// # Errors
///
/// When the manifest, the state file or stdout cannot be used, or a check
/// flag comes ahead of another subcommand.
pub fn run(cli: &Cli) -> Result<u8> {
    match &cli.command {
        None => check(cli, cli.check),
        Some(Command::Check(args)) => check(
            cli,
            CheckArgs {
                refresh: cli.check.refresh || args.refresh,
            },
        ),
        Some(other) if cli.check.refresh => {
            bail!("--refresh goes with check, not {}", other.name())
        }
        Some(Command::Schema) => {
            print(cli, &crate::schema::render())?;
            Ok(report::EXIT_OK)
        }
        Some(Command::Completions { shell }) => {
            print(cli, &completions(*shell)?)?;
            Ok(report::EXIT_OK)
        }
        Some(Command::Status { path }) => {
            let (_, cache_path) = load(cli)?;
            if *path {
                print(cli, &format!("{}\n", cache_path.display()))?;
                return Ok(report::EXIT_OK);
            }
            status(cli, &cache_path)
        }
    }
}

/// The completion script for `shell`, generated from the `Cli` definition.
fn completions(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut command, name, &mut script);
    String::from_utf8(script).context("rendering the completion script")
}

/// The manifest, and the state file its results live in.
fn load(cli: &Cli) -> Result<(Manifest, PathBuf)> {
    let manifest = Manifest::load(&cli.manifest)?;
    let cache_path = cache::resolve(
        &cli.manifest,
        manifest.settings.cache.as_deref(),
        cli.cache.as_deref(),
    )?;
    Ok((manifest, cache_path))
}

fn check(cli: &Cli, args: CheckArgs) -> Result<u8> {
    let (manifest, cache_path) = load(cli)?;
    let previous = match cache::read(&cache_path) {
        Ok(previous) => previous,
        Err(error) => {
            eprintln!("canibump: warning: ignoring {error}");
            None
        }
    };
    let now = Timestamp::now();
    let runtime = tokio::runtime::Runtime::new().context("starting the async runtime")?;
    let registry = defaults::registry();
    let entries = runtime.block_on(run::check(
        &manifest,
        previous.as_ref(),
        &registry,
        now,
        args.refresh,
    ));
    let absolute = std::path::absolute(&cli.manifest).unwrap_or_else(|_| cli.manifest.clone());
    let state = StateFile {
        version: cache::VERSION,
        manifest: absolute,
        entries,
    };
    cache::write(&cache_path, &state)?;
    output(cli, &cache_path, &state, now)
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
    print(cli, &text)?;
    Ok(report::exit_code(&state.entries))
}

/// Writes to stdout, which keeps the styles in `text` when `--color` and the
/// terminal say so and strips them otherwise.
fn print(cli: &Cli, text: &str) -> Result<()> {
    let choice = match cli.color {
        ColorChoice::Auto => anstream::ColorChoice::Auto,
        ColorChoice::Always => anstream::ColorChoice::Always,
        ColorChoice::Never => anstream::ColorChoice::Never,
    };
    AutoStream::new(std::io::stdout().lock(), choice)
        .write_all(text.as_bytes())
        .context("writing to stdout")
}
