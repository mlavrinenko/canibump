#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read GitLab issue and MR state, self-hosted included",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("forges",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[`gitlab:group/proj!N` and `gitlab:host/group/proj#N` check like GitHub items],
      whole-chain: neutral,
      honest-state: helps[a closed issue counts as completed only when an MR closed it; otherwise it is closed without a reason],
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
  links: parent("add-gitlab-gitea-forgejo-redmine-and-jira-providers.typ"),
)

== Problem

+ Declare `state = "gitlab:gitlab.example.com/group/proj!12"`.
+ 🔴 It fails: `unknown scheme gitlab`.


== TO-BE

- 🟢 `#N` is an issue and `!N` an MR; a first path segment with a dot is the host, else gitlab.com.
- 🟢 An MR reads open, merged or closed unmerged; an issue reads completed when `closed_by` lists an MR, else closed without a reason.
- 🟢 `GITLAB_TOKEN` goes to `GITLAB_HOST` (default gitlab.com) and glab's config tokens to their own hosts, never to another host.


== Changes

- `src/refs_tests.rs`, `src/provider/gitlab_tests.rs`: the red tests first.
- `src/refs.rs`: `ItemKind` and the `gitlab:` scheme.
- `src/provider/gitlab.rs`: the client.
- `src/auth.rs`: GitLab tokens by host.
- `src/provider/defaults.rs`: register it.


== Alternatives

- The `gitlab` crate: its own blocking HTTP stack and endpoint builders for two GETs.


== Out of scope

- Reachability on GitLab. A `git:` remote covers it.


== Dogfood

Not applicable: `~/.nix` waits on nothing on GitLab.

