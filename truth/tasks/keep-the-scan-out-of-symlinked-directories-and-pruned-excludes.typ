#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the scan out of symlinked directories and pruned excludes",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("manifest",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: helps[a scan over a Nix repo ends in a blink instead of crawling the store],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ In `~/.nix`, whose `result` and `.direnv/flake-profile-*` link into `/nix/store`, add `[scan] paths = ["**/*.nix"]` with `exclude = ["result/**"]`.
+ Run `canibump`.
+ 🔴 It runs past five minutes: `**` follows the symlinks into the store, and `exclude` only filters after the walk.


== Environment

- canibump at `ef0381a` (main, unreleased `[scan]`), Linux.


== TO-BE

- 🟢 A symlinked directory is never entered; a symlinked file is still read.
- 🟢 `.git` is never entered.
- 🟢 An `exclude` ending in `/**` prunes its directory before the walk enters it.


== Changes

- `src/scan_tests.rs`: the red test first, a link to a directory whose files are not reported.
- `src/scan.rs`: walk the tree by hand with `read_dir`, matching each relative path against `paths` and `exclude`.


== Out of scope

- `.gitignore` awareness: pruning covers the store links, and a new walker crate is not needed for it.


== Dogfood

`~/.nix/canibump.toml` gets a `[scan]` over its Nix files, and `just canibump` there finishes in under a second.

