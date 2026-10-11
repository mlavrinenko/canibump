# Contributing to canibump

## Development

You need [Nix](https://nixos.org/) with flakes. Enter the dev shell with
`direnv allow` or `nix develop`; it also wires the git hooks. Then:

```bash
just          # the recipes you need daily
just --list   # all of them, grouped
```

Before you send a change, run `just fix-check`: it applies clippy's fixes and
the formatter, then runs `just check`.

Recipes live in the [Justfile](Justfile). A body longer than about five lines
lives in `scripts/`.

## Code Style

All clippy lints are set to `deny`, so a violation does not compile. Key
restrictions:

- No `unwrap()`; use `?` with `anyhow` or `thiserror`
- No `todo!()`, `unimplemented!()`, `unreachable!()`; handle every case
- No `unsafe`, no wildcard imports (`use foo::*`)
- Names of at least two characters
- Functions: at most 70 lines, 5 arguments, cognitive complexity 20

## Error Handling

- `anyhow::Result` in the binary, `thiserror::Error` for library errors callers
  match on.
- Propagate with `?`, never `unwrap()` or `expect()`.

## Project Structure

`main.rs` stays a thin entry point: argument parsing and a call into the
library. Logic belongs in `lib.rs` and its modules. `main.rs` is excluded from
coverage, so anything there goes untested.

## Testing

- Unit tests live inline in a `#[cfg(test)] mod tests` next to their code.
- CLI and integration tests live in `tests/` and drive the binary with
  [`assert_cmd`](https://docs.rs/assert_cmd) and
  [`predicates`](https://docs.rs/predicates), see `tests/cli.rs`.
- `just test` runs `cargo nextest run`, one process per test, plus
  `cargo test --doc`, which [nextest](https://nexte.st) skips. A green run
  prints the summary line. `just cover` instruments the same runner, so the
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
(complexity weighted by coverage) and fails above 30. A global floor can stay
green while one branchy, untested function rots; CRAP catches that. Fix a
flagged function with tests or less branching, never a higher threshold. It
refuses a `target/coverage/lcov.info` older than a file in `src/` or `tests/`,
since a function with no coverage record grades as 0%, so run `just cover`
first. `tests/**` and `*_tests.rs` are excluded.

Both are silent on a pass. CI runs them as their own steps, outside
`just check`.

## File Size Limits

Rust files stop at 500 lines, Markdown at 200. Split a file that grows past
its limit into modules or separate documents.

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
- There is no `--force`. Touch an input or delete the record under
  `.mmz/cache/` to re-run a fresh rule.
- `just status` prints each rule's freshness.
- A brand-new clone has no `Cargo.lock` until the first `just check` writes
  it, so the second run misses once more. That's not a broken cache.

## Copy-Paste Gate

`just check-dry` runs [jscpd](https://github.com/kucherenko/jscpd) over `src/`
and `tests/` and fails on any duplicated Rust block of 70+ tokens. Fix it by
extracting a shared helper, never by shuffling tokens until the detector
loses the scent.

## JSON Schema

`schema/canibump.schema.json` is generated from the manifest types by
`canibump schema`, never edited by hand. `tests/schema.rs` fails while the
committed file differs from the generated one; `just schema` regenerates it.
`just install` also copies it to `~/.config/canibump/`.

## Dependency Audits

`just deny` runs `cargo-deny` (advisories, licenses, bans). It needs network,
so it runs in its own CI job, not in `just check`. The policy is
[deny.toml](deny.toml): add a license to `licenses.allow` deliberately, never
to silence a finding. Keep the file in cargo-deny's current schema, since a
retired key fails the whole check. A green run is silent; a red one prints
everything. Drop `--hide-inclusion-graph` from the recipe when chasing one
finding.

`just outdated` lists newer dependency versions, also network-bound and never
a gate. It has nothing to do with `just outdatty-check`, which is about doc
drift.

## Upstream Workarounds

[canibump.toml](canibump.toml) is this repo using itself: one blocker per
workaround in `flake.nix`, the `Justfile` or `.config/nextest.toml` that waits
on an upstream issue or PR, each note naming the file and line to change.
`just canibump` checks them. It needs network, so it stays out of
`just check`. When you comment a workaround, add its blocker, but only for an
upstream item you opened and read. Keep the line numbers true; outdatty flags
the manifest when a file it cites changes.

## MSRV

`rust-version` in `Cargo.toml` is the minimum supported Rust. A CI job builds
on it.

## Mutation Testing

`cargo-mutants` injects bugs and checks that the tests catch them.
`just mutants-diff` sweeps what this branch changed against `main`, the daily
driver. `just mutants` sweeps every file, slow and never a gate. A surviving
`MISSED` mutant is a line covered but pinned by no assertion.

## Git Hooks

Both hooks live in `.githooks/`. The dev shell runs `just install-hooks` on
entry, which points `core.hooksPath` there.

- `pre-commit` runs `mmz --is-fresh --tag gate`, so it runs no gate itself.
- `commit-msg` runs `mt check --commit-msg` (see Task Backlog).

Skip them for one commit with `git commit --no-verify`.

## Task Backlog

Tasks are [MindTape](https://github.com/mlavrinenko/mindtape) files under
`truth/tasks/`, configured in `.mindtape/config.toml`. `just check-tasks`
(`mt check`) is a `just check` arm. A work commit carries a `Refs: <stem>`
footer, and the task's `wip` or `done` flip rides with the work. The
`commit-msg` hook holds both. The goals every open task rates itself against
live in `truth/goals.typ` and `truth/process-goals.typ`.
