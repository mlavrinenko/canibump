#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title, imperative verb first",
    example: "Document the exit codes in the README",
  ),
  kind: "docs",
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  tags: slot("tags", kind: "list", default: ()),
  estimate: slot("estimate", kind: "text", default: "(src-files: 2)"),
  x-goals: (
    product: (
      one-command: slot(
        "one-command",
        kind: "raw",
        default: "neutral",
        example: "helps[a closed-unmerged PR prints one honest line]",
      ),
      many-targets: slot("many-targets", kind: "raw", default: "neutral"),
      whole-chain: slot("whole-chain", kind: "raw", default: "neutral"),
      honest-state: slot("honest-state", kind: "raw", default: "neutral"),
      offline-readers: slot(
        "offline-readers",
        kind: "raw",
        default: "neutral",
      ),
      typed-not-scripted: slot(
        "typed-not-scripted",
        kind: "raw",
        default: "neutral",
      ),
    ),
    process: (
      sound-codebase: slot("sound-codebase", kind: "raw", default: "neutral"),
      docs-match-code: slot(
        "docs-match-code",
        kind: "raw",
        default: "neutral",
      ),
    ),
  ),
)

== Problem

#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
  + Point a `canibump.toml` entry at a PR closed without merging, then run `canibump`.
  + 🔴 It prints `bumpable` and exits 10.
  ```,
)

== TO-BE

#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it should work, behave and feel once this lands",
  example: ```typ
  - 🟢 The entry prints `dead: closed unmerged` and the run exits 0.
  ```,
)

== Source of truth

#slot(
  "source-of-truth",
  kind: "raw",
  prompt: "Bullets: the code, config or live output the prose must match",
  example: ```typ
  - `src/report.rs`: the exit code each outcome maps to.
  ```,
)

== Changes

#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: `path`, then what changes there",
  example: ```typ
  - `src/eval_tests.rs`: the red test first, a closed-unmerged PR evaluating to dead.
  - `src/eval.rs`: map a closed-unmerged PR to `Dead`.
  ```,
)

== Out of scope

#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "Bullets: what not to touch, and why",
  example: ```typ
  - Issue states. Each forge gets its own task.
  ```,
)

== Dogfood

#slot(
  "dogfood",
  kind: "raw",
  prompt: "How this repo will use the change, or Not applicable and why",
  example: ```typ
  `canibump` against `~/.nix/canibump.toml` prints the dead line for a rejected upstream PR.
  ```,
)
