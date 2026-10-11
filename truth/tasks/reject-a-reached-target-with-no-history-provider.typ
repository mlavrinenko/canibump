#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reject a reached target with no history provider",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("manifest", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a target canibump can never follow is a manifest error, not a `failed` line every run],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the manifest accepts exactly README's Targets column for `reached`],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Write a blocker `reached = "gitlab:g/p!12"` with `in = { tag = "v1" }`, and let GitLab answer the MR with `{"state":"merged"}`.
+ Run `canibump --refresh` twice.
+ 🔴 Both runs print `failed    b  no provider for gitlab history` and exit 12. `condition::parse_condition` accepts any `Reachable`, but the registry has code hosts for GitHub and git only, so the check can never pass. `failed` is never reused from the cache, so every run asks GitLab again.
+ 🔴 The same for `gitlab:g/p@<sha>`, `codeberg:`, `gitea:`, `forgejo:`, `redmine:` and `jira:` targets.


== Environment

- `canibump` built from `main` at `f018858`, on Linux, against an `httpmock` GitLab.


== TO-BE

- 🟢 `reached = "gitlab:g/p!12"` is a manifest error naming the file and line, and the run exits 1. It says what `reached` takes, GitHub PRs and commits or git commits, and names the refused target, its key and the target in backticks as other manifest errors do.
- 🟢 It matches README's Targets column for `reached`, GitHub and any git remote, and the `RawReached` doc comment the schema prints.
- 🟢 GitHub items, GitHub commits and git commits parse as before.


== Changes

- `src/condition_tests.rs`: the red test first, every non-GitHub, non-git target refused with that message, and the accepted three still parsing.
- `tests/it/reached.rs`: a GitLab MR in `reached` exits 1 naming `canibump.toml:1`.
- `src/condition.rs`: refuse a `reached` target whose source kind has no shipped code host.


== Out of scope

- Code hosts for GitLab or Gitea. They would widen the accepted list with the provider.


== Dogfood

`just canibump` still loads `canibump.toml`: its one `reached` is a GitHub PR.

