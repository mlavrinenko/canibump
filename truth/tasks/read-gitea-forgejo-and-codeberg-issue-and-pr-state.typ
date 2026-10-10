#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read Gitea, Forgejo and Codeberg issue and PR state",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("forges",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[`gitea:host/owner/repo#N`, `forgejo:` and `codeberg:owner/repo#N` check like GitHub items],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[a thin typed reqwest client behind `Tracker`],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent("add-gitlab-gitea-forgejo-redmine-and-jira-providers.typ")
    + depends-on("read-gitlab-issue-and-mr-state-self-hosted-included.typ"),
)

== Problem

+ Declare `state = "codeberg:forgejo/forgejo#1234"`.
+ 🔴 It fails: `unknown scheme codeberg`.


== TO-BE

- 🟢 A PR reads open, merged or closed unmerged; a closed issue reads closed without a reason, since Gitea keeps none.
- 🟢 tea's config logins give tokens to their own hosts; `GITEA_TOKEN` goes to `GITEA_HOST` only.


== Changes

- `src/provider/gitea_tests.rs`: the red tests first.
- `src/refs.rs`: the `gitea:`, `forgejo:` and `codeberg:` schemes.
- `src/provider/gitea.rs`: the client.
- `src/auth.rs`: Gitea tokens by host.
- `src/provider/defaults.rs`: register it.


== Alternatives

- Reading a closed issue as fixed: a wontfix would read as landed.


== Out of scope

- Reachability on Gitea. A `git:` remote covers it.


== Dogfood

Not applicable: `~/.nix` waits on nothing on Gitea.

