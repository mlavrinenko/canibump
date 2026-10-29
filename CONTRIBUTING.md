# Contributing to canibump

## Development

You need [Nix](https://nixos.org/) with flakes. Enter the dev shell with
`direnv allow` or `nix develop`; it also wires the git hooks. Then:

```bash
just          # the recipes you need daily
just --list   # all of them, grouped
```

Before you send a change, run `just fix-check`: clippy's fixes, the formatter,
then `just check`. Recipes live in the [Justfile](Justfile); a body over about
five lines goes in `scripts/`.

`just check` is two halves, `check-code` and `check-docs`, which CI runs as
separate jobs in narrower shells, `nix develop .#rust` and `.#docs`. A tool a
half runs goes in that shell's list in [flake.nix](flake.nix), one no CI job
runs in `devTools`.

## Worktrees

Work in parallel in a sibling worktree. `just fork` makes one and `just land`
merges it back; both wrap [worktrunk](https://worktrunk.dev) (`wt`, from the
dev shell) with the flags this repo needs, since a bare `wt merge` squashes away
each commit's `Refs:` footer and commits a dirty tree.

```bash
just fork <slug>           # ../canibump.<slug> on a new branch
just fork <slug> --base=@  # the same, from the current commit, not main
just land                  # in the worktree: gate, wip check, fast-forward, remove
```

A worktree only gets commits, so commit first. Never `git stash` in one: all
worktrees share one stash list.

The hooks live in [.config/wt.toml](.config/wt.toml). Before the fast-forward
they run `just check` and `mt check --commits main..HEAD`, which refuses a
branch naming a task still `wip`. After it they carry the gate records
(`mmz --import`, once your mmz has it) and a fresh coverage lcov home.

## Code Style

Every clippy lint is `deny`, so a violation does not compile. The key ones:

- No `unwrap()` or `expect()`; propagate with `?`, `anyhow::Result` in the
  binary, `thiserror::Error` for library errors callers match on
- No `todo!()`, `unimplemented!()`, `unreachable!()`; handle every case
- No `unsafe`, no wildcard imports (`use foo::*`)
- Names of at least two characters
- Functions: at most 70 lines, 5 arguments, cognitive complexity 20
- `rust-version` in `Cargo.toml` is the minimum supported Rust; CI builds on it

## Project Structure

`main.rs` only parses arguments and calls the library; logic lives in `lib.rs`
and its modules, since coverage skips `main.rs`.

## Testing

- Unit tests live inline in a `#[cfg(test)] mod tests` next to their code.
- CLI and integration tests live in `tests/` and drive the binary with
  [`assert_cmd`](https://docs.rs/assert_cmd) and
  [`predicates`](https://docs.rs/predicates), see `tests/it/cli.rs`. The files
  in `tests/it/` link into one binary: a new one needs a `mod` line in
  `tests/it/main.rs`.
- `just test` runs `cargo nextest run`, one process per test, plus
  `cargo test --doc`, which [nextest](https://nexte.st) skips
  <!-- canibump:nextest-doctests -->. A green run prints the summary line.
  `just cover` instruments the same runner, so gate and coverage agree. The
  output level lives in [.config/nextest.toml](.config/nextest.toml).
- Near the linecop limit, `just fix-check` ejects an inline `#[cfg(test)]`
  module into a sibling `_tests.rs` with
  [ejectest](https://github.com/mlavrinenko/ejectest).

## Coverage and CRAP

`just cover` enforces 70% line coverage with `cargo-llvm-cov`, which counts
subprocess e2e tests too. It starts with `cargo llvm-cov clean --workspace`,
or a long-lived checkout keeps scoring binaries that no longer exist.

`just crap` fails a function above 30 on the Change Risk Anti-Patterns metric
(complexity weighted by coverage), which a global floor misses. Fix it with
tests or less branching, never a higher threshold. It refuses an lcov older
than `src/` or `tests/`, so run `just cover` first. `tests/**` and
`*_tests.rs` are excluded.

Both are silent on a pass; CI runs them outside `just check`.

## File Size Limits

Rust files stop at 500 lines, Markdown at 200. Split a file past its limit.

## Dependency Drift

[outdatty.yaml](outdatty.yaml) couples `source` files to the `dependents` that
follow them, like the CLI code and its docs. `just check` fails when a source
changed and its dependents were not re-confirmed. Update them, run
`just outdatty-update` and commit `outdatty.lock` with the change. New files
that must move together get a group.

## Memoized Gates

Every arm of `just check` runs as `just mmz <arm>`, which is `mmz just <arm>`.
[mmz](https://github.com/mlavrinenko/mmz) skips an arm while its declared
inputs match the last run that passed, so a no-op re-run costs nothing and a
one-file change re-runs only the arms that read it. Rules and their inputs
live in [.mmz/config.yaml](.mmz/config.yaml).

- A new arm needs a rule there. mmz rejects a command that matches no rule.
- Rules match by prefix on whole argv tokens. `just test` is `match: exact`,
  so a filtered `just test <filter>` never records the full-suite run.
- `mmz --is-fresh --tag gate` asserts that every `gate`-tagged rule passed
  against this worktree, running nothing. A tag that matches no rule passes
  vacuously, so a new arm must carry it. `just cover` and `just crap` are
  untagged, so they never block a commit.
- No `--force`: touch an input or delete the record under `.mmz/cache/` to
  re-run a fresh rule. `just status` prints each rule's freshness.
- A new clone's first `just check` writes `Cargo.lock`, so the second misses
  once more. The cache is fine.

## Copy-Paste Gate

`just check-dry` runs [jscpd](https://github.com/kucherenko/jscpd) over `src/`
and `tests/`, failing on a Rust block of 70+ duplicated tokens. Extract a
shared helper, never shuffle tokens until the detector loses the scent.

## Docs Site

`just docs-build` builds the site under `www/`, `just docs-serve` serves it
live. [www/README.md](www/README.md) says how it works and how to add a page.

## JSON Schema

`schema/*.schema.json` come from the manifest, report and state types
(`canibump schema [manifest|report|state]`), never from a hand edit. A test
fails while a committed file is stale; `just schema` regenerates them, and
`just install` copies the manifest schema to `~/.config/canibump/`.

## Dependency Audits

`just deny` runs `cargo-deny` (advisories, licenses, bans), silent when green.
It needs network, so it runs in its own CI job, not in `just check`. In
[deny.toml](deny.toml), widen `licenses.allow` deliberately, never to silence a
finding, and keep cargo-deny's current schema: a retired key fails the whole
check. Drop `--hide-inclusion-graph` from the recipe to chase one finding.

`just outdated` lists newer dependency versions, network-bound and never a gate.

## Upstream Workarounds

[canibump.toml](canibump.toml) is this repo using itself: one blocker per
workaround that waits on an upstream issue or PR you opened and read. A
`canibump:<id>` comment marks each spot it touches, so a bumpable line prints
where to edit. The why and the exit plan live in the blocker alone.
`just canibump` checks them, off `just check` since it needs network.

## Releasing

`just release X.Y.Z --dry-run` first, then without the flag, with the entries
under `[Unreleased]`. [cargo-release](https://github.com/crate-ci/cargo-release)
(`release.toml`) bumps `Cargo.toml`, dates them, runs `scripts/release-hook.sh`
(every refusal) and commits. The recipe pushes `main`, waits for `ci.yml` on it
(`scripts/release-ci.sh`), which builds every release target, and tags only if
it is green. After a red run, fix `main` and re-run. The tag push is the only
publish path (`release.yml`), never `cargo publish` by hand. A repo's first
release needs `main` pushed once by hand; the hook asks for it.

## Mutation Testing

`cargo-mutants` injects bugs and checks that the tests catch them.
`just mutants-diff` sweeps this branch against `main`, the daily driver;
`just mutants` sweeps every file, slow and never a gate. A `MISSED` mutant is
a line covered but pinned by no assertion.

## Git Hooks

Both hooks live in `.githooks/`; the dev shell points `core.hooksPath` there.

- `pre-commit` runs `mmz --is-fresh --tag gate`, so it runs no gate itself.
- `commit-msg` runs `mt check --commit-msg` (see Task Backlog).

Skip them for one commit with `git commit --no-verify`.

## Task Backlog

Tasks are [MindTape](https://github.com/mlavrinenko/mindtape) files under
`truth/tasks/`, configured in `.mindtape/config.toml`. `just check-tasks`
(`mt check`) is a `just check` arm, and `mt` comes from qahq in the dev shell.
A work commit carries a `Refs: <stem>` footer and the task's `wip` or `done`
flip; the `commit-msg` hook holds both. Tasks rate themselves against the goals
in `truth/goals.typ` and `truth/process-goals.typ`.
