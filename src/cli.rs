//! Command-line interface.

use anstream::AutoStream;
use anyhow::{Context as _, Result, bail};
use clap::{Args, ColorChoice, CommandFactory as _, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use jiff::Timestamp;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::cache::{self, Entry, StateFile};
use crate::manifest::Manifest;
use crate::outcome::Status;
use crate::provider::defaults;
use crate::report::{self, Actionable, Exit, Report, SITES_ON, Sites};
use crate::run;
use crate::scan::{self, Findings};

/// Can I bump yet? Checks the upstream blockers in canibump.toml.
///
/// Exit codes: 0 when nothing counted showed up, else 8 plus 1 for bumpable, 2
/// for dead, 4 for failed, 16 for an unlisted link or marker and 32 for an
/// unmarked blocker, summed (bumpable and dead is 11). 1 is an unusable
/// manifest or cache, 2 a bad argument. `--fail-on` picks what counts. `status <id>`
/// answers for one blocker instead: 0 bumpable, 1 waiting, 2 anything else.
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
    /// What fails the run, comma-separated or repeated. Defaults to
    /// `[settings] fail-on`, else bumpable, dead, failed, unlisted and
    /// unmarked.
    #[arg(
        long,
        global = true,
        value_enum,
        value_delimiter = ',',
        value_name = "STATUS"
    )]
    pub fail_on: Option<Vec<Actionable>>,
    /// Which lines print the sites of their workaround, comma-separated or
    /// repeated. Defaults to `[settings] sites-on`, else bumpable and dead for
    /// a check and every status for `status`.
    #[arg(
        long,
        global = true,
        value_enum,
        value_delimiter = ',',
        value_name = "STATUS"
    )]
    pub sites_on: Option<Vec<Status>>,
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
    ///
    /// With a blocker id, answers for that one alone by exit code: 0 bumpable,
    /// 1 waiting, 2 anything else, errors included.
    Status {
        /// The blocker to answer for.
        id: Option<String>,
        /// Print the state file's path and exit.
        #[arg(long, conflicts_with = "id")]
        path: bool,
    },
    /// Print a JSON Schema and exit: canibump.toml's, the `--json` report's or the state file's.
    Schema {
        /// What it describes.
        #[arg(value_enum, default_value_t)]
        of: SchemaOf,
    },
    /// Print a shell completion script and exit.
    Completions {
        /// The shell to complete for.
        shell: Shell,
    },
}

/// What `canibump schema` describes.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum SchemaOf {
    /// `canibump.toml`.
    #[default]
    Manifest,
    /// The `--json` report.
    Report,
    /// The state file.
    State,
}

impl SchemaOf {
    /// The schema as pretty JSON with a trailing newline.
    fn render(self) -> String {
        match self {
            Self::Manifest => crate::schema::render(),
            Self::Report => crate::schema::render_report(),
            Self::State => crate::schema::render_state(),
        }
    }
}

impl Command {
    /// The word that selects it.
    const fn name(&self) -> &'static str {
        match self {
            Self::Check(_) => "check",
            Self::Status { .. } => "status",
            Self::Schema { .. } => "schema",
            Self::Completions { .. } => "completions",
        }
    }
}

/// Runs the CLI and returns the exit code.
///
/// # Errors
///
/// When the manifest, the state file or stdout cannot be used, or a check
/// flag comes ahead of another subcommand. The error of `status <id>` is an
/// [`Exit`] with code 2: the usual 1 would read as "waiting".
pub fn run(cli: &Cli) -> Result<u8> {
    let result = dispatch(cli);
    match cli.command {
        Some(Command::Status { id: Some(_), .. }) => {
            result.map_err(|error| Exit::new(report::EXIT_UNKNOWN, error).into())
        }
        _ => result,
    }
}

fn dispatch(cli: &Cli) -> Result<u8> {
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
        Some(Command::Schema { of }) => {
            print(cli, &of.render())?;
            Ok(report::EXIT_OK)
        }
        Some(Command::Completions { shell }) => {
            print(cli, &completions(*shell)?)?;
            Ok(report::EXIT_OK)
        }
        Some(Command::Status { id: Some(id), .. }) => blocker_status(cli, id),
        Some(Command::Status { path, id: None }) => {
            let (manifest, cache_path) = load(cli)?;
            if *path {
                print(cli, &format!("{}\n", cache_path.display()))?;
                return Ok(report::EXIT_OK);
            }
            status(cli, &manifest, &cache_path)
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
    let findings = scan::find(&manifest, &cli.manifest);
    let code = report::exit_code(&state.entries, &findings, counted(cli, &manifest));
    output(
        cli,
        &cache_path,
        &Found {
            state: &state,
            findings: &findings,
            sites_on: sites_on(cli, &manifest),
        },
        now,
        code,
    )
}

fn results(cache_path: &Path) -> Result<StateFile> {
    cache::read(cache_path)?.with_context(|| {
        format!(
            "no results at {}: run `canibump` first",
            cache_path.display()
        )
    })
}

fn status(cli: &Cli, manifest: &Manifest, cache_path: &Path) -> Result<u8> {
    let state = results(cache_path)?;
    let state = StateFile {
        entries: current_entries(manifest, state.entries),
        ..state
    };
    let findings = scan::find(manifest, &cli.manifest);
    let code = report::exit_code(&state.entries, &findings, counted(cli, manifest));
    output(
        cli,
        cache_path,
        &Found {
            state: &state,
            findings: &findings,
            sites_on: sites_on(cli, manifest),
        },
        Timestamp::now(),
        code,
    )
}

/// The entries the manifest still holds under the condition they were checked
/// against. One of a blocker the manifest dropped is skipped; one checked
/// against another condition is no answer, so it is warned about and left out.
fn current_entries(manifest: &Manifest, entries: Vec<Entry>) -> Vec<Entry> {
    entries
        .into_iter()
        .filter(|entry| {
            let Some(blocker) = manifest.blockers.iter().find(|one| one.id == entry.id) else {
                return false;
            };
            let current = entry.spec == blocker.spec;
            if !current {
                eprintln!(
                    "canibump: warning: `{}` was checked against an older condition: run `canibump`",
                    entry.id
                );
            }
            current
        })
        .collect()
}

/// `status <id>`: one blocker's cached result, answered by exit code. A result
/// checked against another condition than the manifest's now is not an answer.
fn blocker_status(cli: &Cli, id: &str) -> Result<u8> {
    let (manifest, cache_path) = load(cli)?;
    let blocker = manifest
        .blockers
        .iter()
        .find(|blocker| blocker.id == id)
        .with_context(|| format!("no blocker `{id}` in {}", cli.manifest.display()))?;
    let state = results(&cache_path)?;
    let entry = state
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .cloned()
        .with_context(|| {
            format!(
                "`{id}` has no result in {}: run `canibump`",
                cache_path.display()
            )
        })?;
    if entry.spec != blocker.spec {
        bail!("`{id}` was checked against an older condition: run `canibump`");
    }
    let code = report::blocker_exit_code(entry.status);
    let findings = Findings {
        sites: scan::find(&manifest, &cli.manifest).sites,
        ..Findings::default()
    };
    let state = StateFile {
        entries: vec![entry],
        ..state
    };
    output(
        cli,
        &cache_path,
        &Found {
            state: &state,
            findings: &findings,
            sites_on: sites_on(cli, &manifest),
        },
        Timestamp::now(),
        code,
    )
}

/// What fails the run: the flag, else the manifest, else all five.
fn counted<'a>(cli: &'a Cli, manifest: &'a Manifest) -> &'a [Actionable] {
    cli.fail_on
        .as_deref()
        .or(manifest.settings.fail_on.as_deref())
        .unwrap_or(&Actionable::ALL)
}

/// Which lines print their sites: the flag, else the manifest, else every
/// status for `status` and bumpable and dead for a check.
fn sites_on<'a>(cli: &'a Cli, manifest: &'a Manifest) -> &'a [Status] {
    let listing = matches!(cli.command, Some(Command::Status { .. }));
    cli.sites_on
        .as_deref()
        .or(manifest.settings.sites_on.as_deref())
        .unwrap_or(if listing { &Status::ALL } else { &SITES_ON })
}

/// What a run found: the blockers' results, and what `[scan]` found.
struct Found<'a> {
    state: &'a StateFile,
    findings: &'a Findings,
    /// The statuses whose lines print their sites.
    sites_on: &'a [Status],
}

/// Prints what was `found` as lines or JSON and returns `code`, the run's exit
/// code.
fn output(cli: &Cli, cache_path: &Path, found: &Found<'_>, now: Timestamp, code: u8) -> Result<u8> {
    let Found {
        state,
        findings,
        sites_on,
    } = *found;
    let text = if cli.json {
        let report =
            Report::new(&state.manifest, cache_path, &state.entries, now, code).with_scan(findings);
        serde_json::to_string_pretty(&report)? + "\n"
    } else {
        let sites = Sites {
            of: &findings.sites,
            on: sites_on,
        };
        report::lines(&state.entries, &sites, now) + &report::scan_lines(findings)
    };
    print(cli, &text)?;
    Ok(code)
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
