#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail an open issue in reached before it closes",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cib", "github", "found"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a `reached` on an issue says it is misconfigured on the first run, not only once the issue closes],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Point a `canibump.toml` blocker at an open GitHub issue: `reached = "github:o/r#1"`, `in = { tags = "v*" }`.
+ Run `canibump` against a GitHub answering `{"state":"open"}` for `#1`.
+ 🔴 It prints `waiting`, though the blocker can never read bumpable. Only after the issue closes does it print `github:o/r#1 is an issue, not a PR; wait on it with state` and exit 12.


== Environment

- `canibump` from `main` at `b30affe`, Linux.


== TO-BE

- 🟢 An open issue in `reached` reads `failed` with the same `is an issue, not a PR` message from the first run.
- 🟢 An open PR still reads `waiting`.


== Changes

- `src/eval_tests.rs`: the red test first, an open issue in `reached` reading failed.
- `src/provider/mod.rs`: the tracker reports whether a GitHub `#N` is an issue or a PR next to its state.
- `src/provider/github.rs`: fill that from the issue's `pull_request` field.
- `src/eval.rs`: `resolve_commit` fails on an issue whatever its state.


== Out of scope

- GitLab, Redmine and Jira targets. The manifest already rejects them in `reached`.


== Dogfood

Not applicable. This repo's `reached` blockers point at PRs.

