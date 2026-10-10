#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Answer reachability on any git remote through gix",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("git",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[any git remote, not only a forge with an API, can say whether a branch or tag holds a commit],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[gix lists refs, fetches and walks ancestry in-process; no `git` binary],
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

+ Wait on a commit in a repo on a plain git server.
+ 🔴 There is no API to ask, so nothing checks it.


== TO-BE

- 🟢 `reached = "git:https://host/repo.git@sha"` with any `in` target works.
- 🟢 The remote is fetched into `$XDG_CACHE_HOME/canibump/git/`, only the refs asked about, and reused across runs.


== Changes

- `src/provider/git_tests.rs`: the red tests first, against a repo the test builds with gix.
- `src/provider/git.rs`: ref listing, fetch and ancestry.
- `src/refs.rs`: the `git:` scheme.
- `src/cli.rs`: register it.


== Alternatives

- Shelling out to `git`: ruled out, the binary stays self-contained.


== Out of scope

- Items on a git remote. A remote has no issues.


== Dogfood

Not applicable: every `~/.nix` blocker lives on GitHub.

