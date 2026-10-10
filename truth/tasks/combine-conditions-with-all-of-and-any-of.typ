#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Combine conditions with all-of and any-of",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  kind: "milestone",
  tags: ("manifest",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[an entry resolves only when every step of an all-of holds],
      honest-state: helps[a dead or failed step in an all-of never lets the entry read as bumpable],
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
  links: depends-on(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  ),
)

== Problem

+ Wait on a fix that must be merged, released and packaged before a patch can go.
+ 🔴 A single condition per entry cannot say that; three entries resolve one by one and none means "drop it now".


== TO-BE

- 🟢 `[[blocker.all]]` and `[[blocker.any]]` hold steps, nest, and name steps with `id`.
- 🟢 An all-of stops at its first step that does not hold and reports it; an any-of resolves on its first step that does.
- 🟢 `at-least = { step = "released" }` reads the release an earlier step found.


== Changes

- `src/manifest.rs`: composite conditions and step ids.
- `src/eval.rs`: all-of and any-of evaluation with step outputs.


== Out of scope

- A free-form expression language. Two combinators cover every chain filed so far.


== Dogfood

The llama.cpp entry in `~/.nix/canibump.toml` is one all-of of three steps.

