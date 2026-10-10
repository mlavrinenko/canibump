#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add GitLab, Gitea/Forgejo, Redmine and Jira providers",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  kind: "milestone",
  tags: ("forges", "trackers"),
  estimate: (src-files: 6),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[issues and MRs on GitLab incl. self-hosted, Gitea, Forgejo and Codeberg, Redmine and Jira issues fit the same manifest],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[each target is one typed provider behind the same trait],
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

+ Wait on a GitLab MR or a Codeberg issue.
+ 🔴 `canibump` knows only GitHub and rejects the reference.


== TO-BE

- 🟢 `gitlab:group/proj!N`, `gitlab:host/group/proj#N`, `gitea:host/owner/repo#N`, `codeberg:owner/repo#N`, `redmine:host#N` and `jira:host#KEY-N` check state like GitHub does.
- 🟢 Tokens come from `GITLAB_TOKEN`, glab's config, `GITEA_TOKEN`, `REDMINE_API_KEY` and `JIRA_TOKEN`, and public items work without one.


== Changes

- `src/refs.rs`: the new reference forms.
- `src/provider/gitlab.rs`, `src/provider/gitea.rs`, `src/provider/redmine.rs`, `src/provider/jira.rs`: one typed client each.
- `src/provider/mod.rs`: register them.


== Out of scope

- Reachability on these forges. A `git:` remote covers it.


== Dogfood

Not applicable: `~/.nix` waits on nothing outside GitHub and nixpkgs today.

