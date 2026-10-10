#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check that a commit or merged PR reached a branch, tag or release",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  kind: "milestone",
  tags: ("git", "github"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[a commit on any git remote, or a merged PR on GitHub, can wait on a branch, a tag or the first release that contains it],
      whole-chain: helps[the first containing release becomes a value a later step compares a package version against],
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[ancestry runs through gix and the GitHub API, never a `git` subprocess],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: depends-on(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  ),
)

== Problem

+ Wait on a fix that is merged but not released yet.
+ 🔴 A `state` check says `bumpable` the moment the PR merges, while no release carries it.


== TO-BE

- 🟢 `reached = "github:o/r#N"` or `"github:o/r@sha"` with `in = { branch = "main" }`, `{ tag = "v1.2" }` or `{ tags = "v*" }` resolves once the commit is there.
- 🟢 `{ tags = "v*" }` names the first matching tag that contains the commit, so a later step can use it.
- 🟢 `reached = "git:<url>@sha"` does the same for any git remote through gix, fetching into a cache dir.


== Changes

- `src/manifest.rs`: the `reached` condition and its `in` target.
- `src/provider/github.rs`: merge commit, compare and tag listing.
- `src/provider/git.rs`: gix ref listing, fetch and ancestry.
- `src/eval.rs`: step outputs.


== Out of scope

- GitLab and Gitea reachability. Their provider milestone adds it.


== Dogfood

`~/.nix/canibump.toml` waits on the first `v*` llama.cpp tag that contains PR 26487.

