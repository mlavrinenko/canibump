#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say on its own line why a provider could not start",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "forges", "found"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: helps[a blocker that cannot be checked says why on its own line, with no `RUST_LOG` needed],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a corrupt state file or a bad API override is reported, not silently ignored],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[`log` and `env_logger` leave the tree; nothing prints through a channel nobody reads],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
)

== Problem

+ `CANIBUMP_GITHUB_API=not-a-url canibump` against a GitHub blocker.
+ 🔴 It prints `failed  x  no provider for GitHub items`. The real reason goes to `log::warn!` in `src/provider/defaults.rs`, and `env_logger::init()` shows only errors unless `RUST_LOG` is set, so nobody sees it.
+ Corrupt the state file (`echo junk > "$(canibump status --path)"`) and run `canibump`.
+ 🔴 It silently drops the old results: `ignoring the old state file: ...` in `src/cli.rs` goes to the same unread `log::warn!`.
+ Errors print Rust `Debug` names (`GitHub`, `DepsDev`) where a manifest says `github`, `deps.dev`.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 A provider that cannot start fails only its own blockers, and their line carries the reason: `` failed  x  github: API URL `not-a-url`: relative URL without a base ``.
- 🟢 A corrupt state file prints `canibump: warning: ignoring <path>: <reason>` to stderr, and the run goes on.
- 🟢 `log` and `env_logger` leave `Cargo.toml`; `main` has no logger to init.
- 🟢 Messages name kinds by the scheme a manifest uses (`github`, `gitlab`, `deps.dev`, `repology`).


== Changes

- `tests/cli.rs`: the red e2e tests first. A bad `CANIBUMP_GITHUB_API` puts its reason on the blocker's line; a corrupt state file warns on stderr and the run exits by its results.
- `src/provider/mod.rs`: `Registry` keeps each kind's start-up error and returns it to every blocker that asks, instead of `no provider for ...`. `SourceKind`/`IndexKind` get a scheme-name `Display`.
- `src/provider/defaults.rs`: register a failed start as that error, not as a missing provider.
- `src/cli.rs`, `src/main.rs`: the stderr warning; no logger.
- `Cargo.toml`: drop `log` and `env_logger`.


== Out of scope

- When providers start. The follow-up task makes them lazy.


== Dogfood

`canibump` in `~/.nix` with a typo in a `CANIBUMP_*_API` override says so on the blocker's line.

