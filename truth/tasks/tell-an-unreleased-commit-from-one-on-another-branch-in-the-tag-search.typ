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
  status: done(2026, 10, 11),
)

== Problem

+ Make a repo whose tags `v1.5.0` and `v1.5.2` hold a fix but `v1.5.1`, cut from another branch, does not.
+ Point a blocker at the fix with `in = { tags = "v*" }` and run `canibump`.
+ 🔴 `first_containing` in `src/eval.rs` binary-searches inside the `1.5` line and assumes a later tag holds what an earlier one does. It probes `v1.5.1`, reads "does not hold", searches right and names `v1.5.2`, missing `v1.5.0`.


== Environment

- `canibump` from `main` at `b30affe`, Linux.


== TO-BE

- 🟢 `CodeHost::contains` answers `Holds` (the ref has the commit), `Before` (the ref is an ancestor of the commit) or `Apart` (neither, or the commit is unseen). GitHub compare `{commit}...{ref}` maps `ahead`/`identical`, `behind`, `diverged`; git walks ancestry both ways.
- 🟢 In the line, `Holds` sends the search left, `Before` right, `Apart` both, left first. The case above names `v1.5.0`.
- 🟢 GitHub calls: one per line before the holding one, then log2 of that line, plus a search of its sides per `Apart` probe. No tag is asked twice, so the worst case is one call per tag. A 64-tag line costs 7 with none apart, 14 with five probes apart, 64 with all.

== Changes

- `src/eval_tests.rs`: the red test first, the out-of-branch tag case.
- `src/provider/mod.rs`: `contains` returns the three-state answer.
- `src/provider/github.rs`, `src/provider/git.rs`: map compare status and ancestry onto it.
- `src/provider/memo.rs`: pass the three-state answer through.
- `src/eval.rs`: `first_containing` steers by it.


== Out of scope

- The per-release-line grouping from `find-the-first-tag-holding-a-commit-per-release-line`, which stays.
- A line whose newest tag is `Apart` still reads as not holding the commit, so an out-of-branch newest tag hides an older holder in its line. Every older line of a project with release branches reads `Apart` against a later fix, so asking past the newest would cost a call per tag of every such line.

== Dogfood

The `released` step in this repo's `canibump.toml` keeps naming cargo-llvm-cov's first `v*` tag.

