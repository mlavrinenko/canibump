#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Treat missing trailing version components as zero",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("packages", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[a `v3.0.0` tag from a step meets Repology `3.0` as equals, so a chain no longer waits forever],
      honest-state: helps[`at-least = "2.0.0"` resolves when the index reports `2.0`],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[the totality corpus grows the zero-padded spellings],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Compare `1.0` with `1.0.0` through `version::compare`, or set `at-least = "2.0.0"` on a package the index reports as `2.0`.
+ 🔴 `compare("1.0", "1.0.0")` is `Less`, not `Equal`: the end sentinel ranks below numbers, so `1.0` sorts below `1.0.0` (`missing_trailing_zeros_equal_the_written_ones` fails with `left: Less, right: Equal`). `2.0` never reaches `2.0.0`, so the blocker waits forever.
+ 🔴 The same break hits a step: `v3.0.0` from a tag list against Repology's `3.0`.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Missing trailing components equal zero: `1.0` = `1.0.0` = `1.0.0.0` = `1`, `v3.0.0` = `3.0`.
- 🟢 A zero before more numbers still counts (`1.0` < `1.0.1`, `1.2` < `1.2.1`, `1` < `1.0.0.1`), and a pre-release stays below its release however its zeros are written (`1.0.0rc1` < `1.0`, `1.0rc1` < `1.0.0`).
- 🟢 The order stays total: the corpus test grows `1`, `1.0.0.0`, `3.0`, `v3.0.0` and `1.0.0rc1` and stays green.


== Changes

- `src/version_tests.rs`: the red tests first, the equal spellings, the zero in the middle, the pre-release below its release, and the longer corpus.
- `src/version.rs`: build the key with zeros dropped that precede the end or a letter run, and update the module doc.
- `CHANGELOG.md`: refine the version-order bullet under `[Unreleased]`.


== Out of scope

- Epochs and ecosystem-specific qualifiers. The one lenient order stays.


== Dogfood

Not applicable, no blocker of this repo compares a short version.

