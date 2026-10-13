# canibump

## Agent Rules

- See and use [Justfile](Justfile). Add any repeatable and regular operations there.
  Every recipe gets a one-line `[doc()]` and a `[group()]`: `daily` is what bare
  `just` lists, `gate` the arms of `just check`, `extra` the rest. A body over
  ~5 lines goes to `scripts/`. `set shell` gives every line `bash -eo pipefail`,
  each in a fresh shell.
- At the end ensure that `just fix-check` is green.
- Tests: inline `#[cfg(test)]` units; CLI/integration in `tests/` (`assert_cmd` + `predicates`). `just fix-check` auto-ejects inline tests from oversized files via `ejectest`.
- Coverage and CRAP gates run in CI, outside `just check`: `just cover` then
  `just crap`, both silent on a pass. `just cover` uses `cargo-llvm-cov` (not
  tarpaulin) — it merges child-process profraw, so subprocess-based e2e tests
  attribute correctly. If `just crap` flags a function, add tests or reduce its
  branching — don't raise the threshold to dodge it.
- Every arm of `just check` is `just mmz <subgate>`:
  [mmz](https://github.com/mlavrinenko/mmz) skips arms whose declared inputs are
  unchanged since they last passed, so a new or retargeted gate needs its rule in
  [.mmz/config.yaml](.mmz/config.yaml) — an unmatched command is a hard error, not
  a silent skip. Don't wrap `just check` arms in `chronic`: a cache hit is already
  quiet, and a miss must stream its output.
- `just check` runs `check-code` and `check-docs`; CI runs each in its own job
  and dev shell (`.#rust`, `.#docs`). A new arm joins one half, and its tool
  goes in that half's list in [flake.nix](flake.nix), else CI lacks it.
- Be careful with the context. Omit non-necessary command outputs using `chronic` or `grep`.
  A recipe that prints on a green run wraps its command in `scripts/quiet.sh`.
- [outdatty.yaml](outdatty.yaml) couples sources to dependents. When `just check`
  reports drift, update the listed dependents, run `just outdatty-update`, and
  commit the refreshed `outdatty.lock` with the change. Add a group whenever you
  introduce files that must stay in sync.
- `just check-dry` (jscpd) flags duplicated Rust blocks of >=70 tokens. Fix
  a finding by extracting a shared helper — never by shuffling tokens until
  the detector loses the scent.
- `schema/*.schema.json` are generated from the types of the manifest, the
  `--json` report and the state file (`canibump schema [manifest|report|state]`),
  never hand-edited. After changing one of those types or a doc comment on it,
  run `just schema`; a test fails while a file is stale.
  `just install` also writes it to `~/.config/canibump/`.
- `just deny` (cargo-deny: advisories, licenses, bans) is not in `just check` —
  it needs network. A green run prints nothing. Run it after touching
  dependencies. Widen `deny.toml`'s allow-list deliberately; never to silence a
  finding. `just outdated` (cargo-outdated) is network-bound too and reports
  newer upstream releases; it is a dev aid, never a gate.
- `just test` gates the suite on cargo-nextest, whose output level lives in
  `.config/nextest.toml` — a green run prints only the summary line, a red one
  prints each failure's full output — plus a `cargo test --doc` pass, because
  nextest does not run doctests <!-- canibump:nextest-doctests -->. `just cover`
  instruments that same runner via `cargo llvm-cov nextest`, so a test cannot
  pass under the gate and fail under coverage (or the reverse) with nothing
  saying so.
- Release only with `just release X.Y.Z`, after `just release X.Y.Z --dry-run`
  passes; never `cargo publish` by hand, never bump `Cargo.toml` or date the
  CHANGELOG yourself, and don't push main first. The one exception is a repo's
  very first push of main, made once by hand, since the hook needs main on
  origin. Keep the release's entries under `[Unreleased]`:
  [cargo-release](https://github.com/crate-ci/cargo-release) (`release.toml`)
  bumps the version, dates them and commits. Its `pre-release-hook`,
  [scripts/release-hook.sh](scripts/release-hook.sh), refuses a dirty tree, a
  branch but main, an origin with no main, an origin/main that main lacks
  commits of, an existing tag, a version already on crates.io, a CHANGELOG with
  nothing to release, failing gates (`just check`) and a failing
  `cargo publish --dry-run`. Then the recipe pushes main, waits for `ci.yml` on
  that commit (`scripts/release-ci.sh`), and tags only if it is green; a red run
  leaves main pushed and no tag, and a re-run after fixing it changes nothing
  the first run did. A release whose workflow failed is finished on the same
  tag with `gh workflow run <release workflow> -f tag=vX.Y.Z`; a pushed tag
  never moves.
- The docs site lives in `www/`, built by `just docs-build` (silent on success)
  and served by `just docs-serve`. [www/README.md](www/README.md) says how it
  works and how to add a page. Every terminal block on it is a capture of the
  binary against a fixture in `www/examples/`, never typed by hand.
  `just check` builds the site and runs `check-doc-blocks` and
  `check-doc-prose` over it. README.md is rendered from `docs/src/readme.typ`,
  so edit that and run `just docs-md`.
- Eating your own dog food: the tool should use itself if applicable.
  [canibump.toml](canibump.toml) lists the upstream items this repo's own
  workarounds wait on; `just canibump` checks it (network-bound, outside
  `just check`). For each workaround, add a blocker with a real upstream item
  you opened, and mark every spot it touches with a `canibump:<id>` comment.
  The why and the exit plan go in the blocker, not beside the marker.
- Delegated work forks with `just fork <slug>` (a sibling worktree on a new
  branch, `../canibump.<slug>` by default, dev shell allowed) and lands with
  `just land` run in that worktree: it gates, checks no named task is left
  `wip`, fast-forwards main and removes the worktree. Both wrap
  [worktrunk](https://worktrunk.dev); [.config/wt.toml](.config/wt.toml) holds
  the hooks. Never run a bare `wt merge`: it squashes away the `Refs:` footers.
- Work is tracked with [MindTape](https://github.com/mlavrinenko/mindtape) in
  `truth/tasks/` (load the `mindtape` skill; `mt ls @agent` lists open agent work).
  Commits follow Conventional Commits; a work commit names its task in a
  `Refs: <stem>` footer, flips it `wip`/`done` in the same commit, and a
  `feat`/`fix`/`perf` touching `src/` adds a CHANGELOG entry. The
  `commit-msg` hook runs `mt check --commit-msg` to hold all of that.

See [CONTRIBUTING.md](CONTRIBUTING.md) for project conventions and code standards.
