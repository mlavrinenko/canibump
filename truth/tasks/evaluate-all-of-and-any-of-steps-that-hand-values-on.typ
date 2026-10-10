#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate all-of and any-of steps that hand values on",
  kind: "feat",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("manifest",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[merged AND released AND packaged is one entry that resolves only when all three hold],
      honest-state: helps[a dead or failed step stops an all-of before it can read bumpable],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: parent("combine-conditions-with-all-of-and-any-of.typ"),
)

== Problem

+ Declare the llama.cpp patch's blocker: PR merged, then in a `v*` tag, then that tag in `nix_unstable`.
+ 🔴 A blocker holds one condition, so the three resolve separately and none says "drop it now".


== TO-BE

- 🟢 `[[blocker.all]]` and `[[blocker.any]]` hold steps and nest; a step may carry an `id`.
- 🟢 An all-of stops at its first step that does not hold and labels it; an any-of resolves on its first that does, and is dead only when every step is.
- 🟢 `at-least = { step = "released" }` uses the tag an earlier `reached` step found; a bad reference fails when the manifest loads.


== Changes

- `src/manifest_tests.rs`, `src/eval_tests.rs`: the red tests first.
- `src/condition.rs`: conditions move out of `src/manifest.rs`; steps, `Bound` and the step checks.
- `src/outcome.rs`: an outcome carries a value for later steps.
- `src/eval.rs`: all-of, any-of and step outputs.


== Alternatives

- A boolean expression string: a parser to write and errors that point at a column instead of a step.


== Out of scope

- Values other than a found tag. Nothing needs one yet.


== Dogfood

The llama.cpp entry in `~/.nix/canibump.toml` is one all-of of three steps.

