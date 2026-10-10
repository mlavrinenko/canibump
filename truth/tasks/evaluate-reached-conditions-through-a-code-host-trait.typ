#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate reached conditions through a code host trait",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("manifest", "git"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[a commit or a merged PR can wait on a branch, a tag, or the first tag matching a pattern],
      whole-chain: neutral,
      honest-state: helps[a PR closed unmerged is dead here too, never `waiting` for a merge commit that will not come],
      offline-readers: neutral,
      typed-not-scripted: helps[reachability is a `CodeHost` trait any target implements],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: parent(
    "check-that-a-commit-or-merged-pr-reached-a-branch-tag-or-release.typ",
  ),
)

== Problem

+ Wait on a fix that merged on GitHub but is in no release yet.
+ 🔴 `state = "github:o/r#N"` turns `bumpable` at the merge, while no tag carries the fix.


== TO-BE

- 🟢 `reached = "github:o/r#N"` or `"github:o/r@sha"` with `in = { branch = "main" }`, `{ tag = "v1.2" }` or `{ tags = "v*", since = "0.5.0" }` parses.
- 🟢 It is bumpable once the commit is in that ref; for a pattern, the detail names the first matching tag that has it.
- 🟢 Tags sort by version, semver when both parse and leniently otherwise, so `b7012` sorts as `7012`.


== Changes

- `src/eval_tests.rs`: the red tests first, against a fake code host.
- `src/refs.rs`: commit references, `@sha`.
- `src/manifest.rs`: the `reached` condition and its `in` target.
- `src/provider/mod.rs`: the `CodeHost` trait.
- `src/eval.rs`: reachability, finding the first containing tag by binary search.
- `src/version.rs`: the version order.


== Alternatives

- Check every tag: thousands of API calls for a repo like llama.cpp; tags are ordered, so a binary search needs a dozen.


== Out of scope

- Real hosts. The next two tasks add GitHub and gix.


== Dogfood

Not applicable until a host answers; the GitHub task wires it.

