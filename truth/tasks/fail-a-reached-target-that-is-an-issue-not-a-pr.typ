#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail a reached target that is an issue, not a PR",
  kind: "fix",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("github", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[an issue in `reached` can no longer read `bumpable` without any branch or tag being checked],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[a unit test pins every issue-only state in `reached`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Write a blocker `reached = "github:o/r#1"` with `in = { tag = "v9.9.9" }`, and let GitHub answer `GET /repos/o/r/issues/1` with `{"state":"closed","state_reason":"completed"}` (an issue fixed upstream, no PR).
+ Run `canibump --refresh`.
+ 🔴 It prints `bumpable  b  github:o/r#1 closed as completed (n)` and exits 9. No tag was read: `eval::resolve_commit` returns `state_outcome` for every state but `Merged`, and `describe` maps `Completed` to `Bumpable`.
+ 🔴 The same holds for any issue-only state: `NotPlanned`, `Duplicate` and `ClosedNoReason` read `dead`, as if upstream rejected a PR, though `reached` has nothing to follow.


== Environment

- `canibump` built from `main` at `f018858`, on Linux, against an `httpmock` GitHub.


== TO-BE

- 🟢 A `reached` item in an issue-only state (`Completed`, `NotPlanned`, `Duplicate`, `ClosedNoReason`) reads `failed` with `github:o/r#1 is an issue, not a PR; wait on it with state`, and the run exits 12.
- 🟢 `failed`, not `dead`: the manifest asks what an issue has no answer to (a merge commit). `bumpable` would say it landed and `dead` that upstream rejected it; both lie. `failed` never counts as landed and is never reused from the cache, so the line nags every run until the manifest is fixed.
- 🟢 An open item still waits, a PR closed unmerged still reads `dead`, a merged PR still follows its merge commit.


== Changes

- `src/eval_tests.rs`: the red test first, each issue-only state in `reached` evaluating to that `failed` outcome, with no merge-commit or history call.
- `src/eval.rs`: `resolve_commit` maps the issue-only states to `failed` before any code-host call.


== Out of scope

- An open issue in `reached` still waits as `is open` until it closes. The GitHub issues API tells a PR from an issue, but `ItemState` drops it; carrying the item kind lives in `src/provider/`, not here.
- Other forges: `reached` accepts only GitHub and git targets once `reject-a-reached-target-with-no-history-provider` lands, and git targets are commits.


== Dogfood

`just canibump` keeps reading `reached = "github:taiki-e/cargo-llvm-cov#522"` in `canibump.toml` as before: it is a PR.

