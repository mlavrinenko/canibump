#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Find the first tag holding a commit per release line",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("git", "github", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[a fix released on a maintenance line names its tag, so `at-least = { step = ... }` compares against it],
      honest-state: helps[a commit in v1.5.0 no longer reads `waiting` because no 2.x tag holds it],
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

+ Follow a merged PR with `in = { tags = "v*" }` whose merge commit landed on the 1.x branch only: tags `v1.5.0` (has it), `v2.0.0` and `v2.0.1` (lack it).
+ 🔴 It reads `waiting` with `github:o/r#1 (abc1234) is in no v* tag yet`, after 2 checks. `eval::first_containing` binary-searches the version-sorted tags, which is right only when every tag after the first holding tag holds it too. A maintenance line breaks that: its tags sort below the next major's.
+ 🔴 With `v1.5.0` (has it), `v2.0.0` (lacks it) and `v2.1.0` (has it), the search names `v2.1.0`, not `v1.5.0`.


== Environment

- `canibump` built from `main` at `f018858`, on Linux, against a fake code host in `src/eval_tests.rs`.


== TO-BE

- 🟢 Version-sorted tags split into release lines: tags sharing every number run but the last (`v1.4.2`, `v1.4.10`, `v1.4.0-rc1` share `1.4`; all `b`-builds share one). From the lowest line up, canibump checks each line's newest tag, then binary-searches the first line whose newest tag holds the commit.
- 🟢 Both cases above name `v1.5.0`. A commit in no tag still reads `waiting`.
- 🟢 Trade-off: one call per line plus log2 of the line, not log2 of all tags. Same for `b`-builds and `1.0.x`; one call per minor for 0.x projects. A linear scan costs a call per tag on every `waiting` run, which unauthenticated GitHub (60 an hour) cannot pay.
- 🟢 Still assumed: within a line, later tags hold what an earlier one holds.


== Changes

- `src/eval_tests.rs`: the red tests first, the two cases above, then call counts for one long line and for many short ones.
- `src/eval.rs`: `first_containing` groups tags by release line and searches as above.
- `docs/manifest.md`: say how the first tag is found and what it assumes.


== Out of scope

- Asking the host for every tag that holds a commit in one call (`git tag --contains`, GitHub compare's `diverged`): it changes the `CodeHost` trait in `src/provider/`.


== Dogfood

`just canibump` runs the `released` step of `llvm-cov-path-fallback` in `canibump.toml` over cargo-llvm-cov's `v0.x.y` tags, a few lines with many patches each.

