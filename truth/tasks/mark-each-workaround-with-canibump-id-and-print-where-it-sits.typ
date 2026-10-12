#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Mark each workaround with canibump:<id> and print where it sits",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "manifest", "dogfood"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: helps[a bumpable line says which files to edit],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: helps[`--json` carries each blocker's sites],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[notes stop carrying a `file:line` that rots on every edit],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Open `canibump.toml`: each note starts with a `file:line`, like `flake.nix:97-98`.
+ Add a line above `flake.nix:97`.
+ 🔴 The note now points at the wrong line, and nothing says so. AGENTS.md asks agents to keep it true by hand.


== TO-BE

- 🟢 A line holding `canibump:<id>` in a scanned file marks a site of blocker `<id>`; a blocker may have many.
- 🟢 A bumpable or dead line prints its sites under it: `at flake.nix:29, Justfile:26`. `--json` gives each row `sites: [{file, line}]`.
- 🟢 A marker no blocker has prints `unlisted  f:3  canibump:old has no blocker`.
- 🟢 `canibump-ignore` skips a line, replacing the unreleased `canibump:ignore`.
- 🟢 The manifest rejects a blocker id outside `[A-Za-z0-9_-]+`.


== Changes

- `src/scan_tests.rs`: the red tests first: a marker makes a site, an unknown one a hit, `canibump-ignore` skips.
- `src/scan.rs`: collect markers beside links; return hits and sites.
- `src/manifest.rs`: the id grammar.
- `src/report.rs`, `src/cli.rs`: sites in lines and `--json`.
- `docs/scan.md`, `CHANGELOG.md`, `schema/*`: markers and the rename.
- `canibump.toml`, `flake.nix`, `Justfile`, `AGENTS.md`, `CONTRIBUTING.md`: markers in, `file:line` out of notes, the AGENTS.md rule replaced.


== Alternatives

- Keep `file:line` in notes: it rots silently on every edit above it.
- An upstream URL beside each workaround: duplicates the manifest's ref, and a version blocker has none.
- Reserve `ignore` as an id instead of renaming the skip marker: a rule users trip over for nothing.


== Out of scope

- A blocker with no site: the `unmarked` task.
- Files that take no comments, like JSON: the manifest-declared site task.


== Dogfood

`just canibump` prints `at flake.nix:…` under `llvm-cov-path-fallback` once it turns bumpable.

