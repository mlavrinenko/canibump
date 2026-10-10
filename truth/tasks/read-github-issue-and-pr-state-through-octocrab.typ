#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read GitHub issue and PR state through octocrab",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("github",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[GitHub issues and PRs, the commonest blockers, check for real],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[octocrab with typed response structs, no `gh` subprocess for the check itself],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: parent(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  )
    + depends-on("evaluate-state-conditions-through-provider-traits.typ"),
)

== Problem

+ Declare `state = "github:ggml-org/llama.cpp#26487"`.
+ 🔴 No provider answers for `github:`, so the check cannot run.


== TO-BE

- 🟢 A PR reads as open, merged or closed-unmerged; an issue as open, completed, not planned or duplicate.
- 🟢 The token comes from `GITHUB_TOKEN`, `GH_TOKEN` or `gh auth token`, else the request goes unauthenticated.
- 🟢 401, 403, 404 and rate limits come back as a `CheckError` saying which.


== Changes

- `src/provider/github.rs`: the red tests first, against an httpmock server; then the client.
- `src/auth.rs`: token discovery.


== Alternatives

- octocrab's own models: they demand every field of the payload, so a test fixture is a full API dump.


== Out of scope

- GitHub Enterprise hosts. `CANIBUMP_GITHUB_API` points at one for now.


== Dogfood

`canibump` reads the real state of llama.cpp PR 26487.

