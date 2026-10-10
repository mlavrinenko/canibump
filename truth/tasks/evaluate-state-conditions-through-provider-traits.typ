#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate state conditions through provider traits",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("manifest",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[closed-unmerged, not-planned and duplicate map to dead, a provider error to failed, never to bumpable],
      offline-readers: neutral,
      typed-not-scripted: helps[a target is a `Tracker` implementation registered by source kind],
    ),
    process: (
      sound-codebase: helps[the evaluator is tested against a fake tracker, no network],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  )
    + depends-on("parse-canibump-toml-into-typed-blockers.typ"),
)

== Problem

+ Parse a manifest whose blocker waits on a PR that was closed without merging.
+ 🔴 Nothing turns that into an outcome, so there is no way to tell it apart from a merge.


== TO-BE

- 🟢 `evaluate` returns `waiting`, `bumpable`, `dead` or `failed` with a one-line detail.
- 🟢 Merged and closed-as-completed are bumpable; closed-unmerged, not-planned, duplicate and closed-without-reason are dead.


== Changes

- `src/eval.rs`: the red tests first, against a fake tracker; then the state mapping.
- `src/outcome.rs`: `Status` and `Outcome`.
- `src/provider/mod.rs`: `Tracker`, `ItemState`, `CheckError` and the `Registry`.


== Alternatives

- One trait with every capability: each target would stub the half it lacks.


== Out of scope

- Real HTTP clients. The GitHub task adds the first.


== Dogfood

Not applicable: no provider talks to the network yet.

