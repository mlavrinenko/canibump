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

[.config/wt.toml](.config/wt.toml) holds the hooks, so a clone needs no
worktrunk config of its own. Before the fast-forward they run `just check` and
`mt check --commits main..HEAD`, which refuses a branch naming a task still
`wip`. After it they carry the gate records (`mmz --import`, once your mmz has
it) and a fresh coverage lcov to the primary checkout.

## Code Style

All clippy lints are set to `deny`, so a violation does not compile. Key
restrictions:

- No `unwrap()` or `expect()`; propagate with `?`, `anyhow::Result` in the
  binary, `thiserror::Error` for library errors callers match on
- No `todo!()`, `unimplemented!()`, `unreachable!()`; handle every case
- No `unsafe`, no wildcard imports (`use foo::*`)
- Names of at least two characters
- Functions: at most 70 lines, 5 arguments, cognitive complexity 20
- `rust-version` in `Cargo.toml` is the minimum supported Rust; CI builds on it

## Project Structure

`main.rs` stays a thin entry point: argument parsing and a call into the
library. Logic belongs in `lib.rs` and its modules. `main.rs` is excluded from
coverage, so anything there goes untested.

## Testing

- Unit tests live inline in a `#[cfg(test)] mod tests` next to their code.
- CLI and integration tests live in `tests/` and drive the binary with
  [`assert_cmd`](https://docs.rs/assert_cmd) and
  [`predicates`](https://docs.rs/predicates), see `tests/it/cli.rs`. The files
  in `tests/it/` link into one binary: a new one needs a `mod` line in
  `tests/it/main.rs`.
- `just test` runs `cargo nextest run`, one process per test, plus
  `cargo test --doc`, which [nextest](https://nexte.st) skips
  <!-- canibump:nextest-doctests -->. A green run prints the summary line. `just cover` instruments the same runner, so the
  gate and the coverage cannot disagree. The output level lives in
  [.config/nextest.toml](.config/nextest.toml).
- Near the linecop limit, `just fix-check` ejects an inline `#[cfg(test)]`
  module into a sibling `_tests.rs` with
  [ejectest](https://github.com/mlavrinenko/ejectest).

## Coverage and CRAP

`just cover` enforces 70% line coverage with `cargo-llvm-cov`, which counts
subprocess e2e tests too. It starts with `cargo llvm-cov clean --workspace`,
or a long-lived checkout keeps scoring binaries that no longer exist.

`just crap` scores each function by the Change Risk Anti-Patterns metric
(complexity weighted by coverage) and fails above 30, which a global coverage
floor misses. Fix a flagged function with tests or less branching, never a
higher threshold. It refuses a `target/coverage/lcov.info` older than a file
in `src/` or `tests/`, since an uncovered function grades 0%, so run `just
cover` first. `tests/**` and `*_tests.rs` are excluded.

Both are silent on a pass; CI runs them outside `just check`.

## File Size Limits

Rust files stop at 500 lines, Markdown at 200. Split a file past its limit.

## Dependency Drift

[outdatty.yaml](outdatty.yaml) couples `source` files to the `dependents` that
must follow them, like the CLI code and the docs that describe it. `just check`
fails when a source changed and its dependents were not re-confirmed. Review
them, update what needs it, run `just outdatty-update` and commit
`outdatty.lock` with the change. Add a group for any new files that must move
together.

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
- A new clone has no `Cargo.lock` until the first `just check` writes it, so the
  second run misses once more. That is no broken cache.

## Copy-Paste Gate

`just check-dry` runs [jscpd](https://github.com/kucherenko/jscpd) over `src/`
and `tests/`, failing on a Rust block of 70+ duplicated tokens. Extract a
shared helper, never shuffle tokens until the detector loses the scent.

## Docs Site

The site under `www/` builds with `just docs-build` and serves with live
reload from `just docs-serve`. [www/README.md](www/README.md) says how it
works and how to add a page.

## JSON Schema

`schema/*.schema.json` come from the manifest, report and state types
(`canibump schema [manifest|report|state]`), never from a hand edit. A test
fails while a committed file is stale; `just schema` regenerates them, and
`just install` copies the manifest schema to `~/.config/canibump/`.

## Dependency Audits

`just deny` runs `cargo-deny` (advisories, licenses, bans). It needs network,
so it runs in its own CI job, not in `just check`. The policy is
[deny.toml](deny.toml): add a license to `licenses.allow` deliberately, never
to silence a finding, and keep the file in cargo-deny's current schema, since a
retired key fails the whole check. A green run is silent. Drop
`--hide-inclusion-graph` from the recipe when chasing one finding.

`just outdated` lists newer dependency versions, network-bound and never a gate.

## Upstream Workarounds

[canibump.toml](canibump.toml) is this repo using itself: one blocker per
workaround in `flake.nix`, the `Justfile` or `.config/nextest.toml` that waits
on an upstream issue or PR. A `canibump:<id>` comment marks each spot a
workaround touches, so a bumpable line prints where to edit. `just canibump`
checks them, off `just check` since it needs network. Add a blocker for each
workaround, for an upstream item you opened and read, and mark its spots. The
why and the exit plan live in the blocker alone.

## Releasing

`just release X.Y.Z --dry-run` first, then without the flag. Keep the entries
under `[Unreleased]`. [cargo-release](https://github.com/crate-ci/cargo-release)
(`release.toml`) bumps `Cargo.toml`, dates them, runs `scripts/release-hook.sh`,
which holds every refusal, and commits. The recipe pushes `main`, waits for
`ci.yml` on it (`scripts/release-ci.sh`), which builds every release target
(`build.yml`), and tags only if it is green. After a red run, fix `main` and
re-run: the bump stays done. The tag push is the only publish path
(`release.yml`), so never `cargo publish` by hand. A repo's first release needs
`main` pushed once by hand beforehand; the hook asks for it.

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
(`mt check`) is a `just check` arm, skipped in CI, whose dev shell has no
MindTape yet. A work commit carries a `Refs: <stem>` footer, and the task's
`wip` or `done` flip rides with the work. The `commit-msg` hook holds both. The
goals every open task rates itself against live in `truth/goals.typ` and
`truth/process-goals.typ`.
