#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Tell an unreleased commit from one on another branch in the tag search",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("cib", "github", "git", "found"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[the first tag named is the true first tag even when one release line has a tag cut from another branch],
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

+ Make a repo whose tags `v1.5.0` and `v1.5.2` hold a fix but `v1.5.1`, cut from another branch, does not.
+ Point a blocker at the fix with `in = { tags = "v*" }` and run `canibump`.
+ 🔴 `first_containing` in `src/eval.rs` binary-searches inside the `1.5` line and assumes a later tag holds what an earlier one does, so it can name `v1.5.2` or miss `v1.5.0` depending on where the search probes.


== Environment

- `canibump` from `main` at `b30affe`, Linux.


== TO-BE

- 🟢 `CodeHost::contains` answers three ways: contained, not yet (an ancestor of a later point), or on another branch. GitHub's compare API (`behind`, `diverged`) and git ancestry both give that.
- 🟢 The search uses it to stay correct inside a line without falling back to a per-tag scan.


== Changes

- `src/eval_tests.rs`: the red test first, the out-of-branch tag case.
- `src/provider/mod.rs`: `contains` returns the three-state answer.
- `src/provider/github.rs`, `src/provider/git.rs`: map compare status and ancestry onto it.
- `src/eval.rs`: `first_containing` steers by it.


== Out of scope

- The per-release-line grouping from `find-the-first-tag-holding-a-commit-per-release-line`, which stays.


== Dogfood

The `released` step in this repo's `canibump.toml` keeps naming cargo-llvm-cov's first `v*` tag.

