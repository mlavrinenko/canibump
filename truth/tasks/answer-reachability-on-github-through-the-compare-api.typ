#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Answer reachability on GitHub through the compare API",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("github",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[GitHub commits and merged PRs reach branches and tags without a clone],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[three typed GitHub endpoints, no `git` subprocess],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent(
    "check-that-a-commit-or-merged-pr-reached-a-branch-tag-or-release.typ",
  )
    + depends-on("evaluate-reached-conditions-through-a-code-host-trait.typ"),
)

== Problem

+ Declare `reached = "github:ggml-org/llama.cpp#26487"` with `in = { tags = "v*" }`.
+ 🔴 No code host answers for `github:`, so the check fails.


== TO-BE

- 🟢 A merged PR resolves to its merge commit; `compare/{commit}...{ref}` says whether a ref contains it.
- 🟢 `git/matching-refs/tags/<prefix>` lists the tags a pattern can match in one call.


== Changes

- `src/provider/github_tests.rs`: the red tests first, against httpmock.
- `src/provider/github.rs`: `CodeHost` for GitHub.
- `src/cli.rs`: register it.


== Alternatives

- A gix clone of every GitHub repo: llama.cpp alone is hundreds of MB, while the API answers in a few calls.


== Out of scope

- GitHub Releases as distinct from tags. A release is a tag, and the pattern picks the stream.


== Dogfood

`canibump` finds which llama.cpp `v*` tag first carries PR 26487 once it merges.

