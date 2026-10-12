#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report upstream links that no blocker lists",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("manifest", "cli"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: helps[a forgotten workaround prints as an `unlisted` line in the same run],
      many-targets: helps[catchers are regexes, so any tracker's URL maps to a manifest ref],
      whole-chain: neutral,
      honest-state: helps[a pin whose comment links an untracked issue no longer passes as checked],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 12),
)

== Problem

+ Pin a package in `hosts/gpu.nix` with a comment `# pin until https://github.com/NixOS/nixpkgs/issues/412345 is fixed`, and add no `[[blocker]]` for it.
+ Close that issue upstream, then run `canibump`.
+ 🔴 It prints nothing about the pin and exits 0. The workaround outlives its reason, the one thing canibump exists to prevent.


== TO-BE

- 🟢 `[scan] paths = ["**/*.nix"]` makes `canibump` read those files and print `unlisted  hosts/gpu.nix:41  github:NixOS/nixpkgs#412345 has no blocker`.
- 🟢 Built-in catchers turn GitHub, GitLab and Codeberg issue, PR and MR URLs into manifest refs, on by default.
- 🟢 `[[scan.catcher]]` adds a regex catcher whose named groups fill a `ref` template, replaces a built-in by its `id`, or turns one off with `enabled = false`.
- 🟢 A ref any blocker's condition names, at any depth of `all`/`any`, is listed; a line holding `canibump:ignore` is skipped; the manifest itself is never scanned.
- 🟢 No `[scan]`, no scan: a manifest without one behaves as before.


== Changes

- `src/scan_tests.rs`: the red tests first, a GitHub URL with no blocker reported with its file and line, a listed one not.
- `src/scan.rs`: the `[scan]` config, the built-in catchers, the file walk and the `unlisted` lines.
- `src/manifest.rs`: read `[scan]` into the manifest and validate each catcher.
- `src/cli.rs`: run the scan in `check` and print its lines after the blockers'.
- `src/lib.rs`, `Cargo.toml`: the module and the `regex` crate.


== Alternatives

- Fixed URL parsers per forge, no regex: a self-hosted Jira or a nixpkgs PR tracker link would need a release to catch.
- A separate `canibump scan` subcommand: CI runs `canibump`, so a forgotten workaround would never fail it.
- Status word `untracked`: nine characters break the status column every existing line aligns on.


== Out of scope

- The exit code and `--json`: a follow-up task counts `unlisted` there.
- `.gitignore` awareness: `paths` and `exclude` say what to read.
- Commit links (`/commit/<sha>`): workaround comments cite issues and PRs.


== Dogfood

`canibump.toml` gains `[scan]` over `flake.nix`, the `Justfile` and `scripts/`, so a workaround comment added there without a blocker fails `just canibump`.

