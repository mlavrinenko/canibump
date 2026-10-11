#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Send a token only to the host it was issued for",
  kind: "fix",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("github", "forges", "trackers", "found"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a GitHub Enterprise or mirror override authenticates with its own token, or none],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[unit and end-to-end tests pin which host each token goes to],
      docs-match-code: helps[`docs/manifest.md` says what each `CANIBUMP_<PROVIDER>_API` override does to tokens],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `canibump` with `GH_TOKEN=t0ken` and `CANIBUMP_GITHUB_API` on an httpmock server, as `src/provider/defaults.rs` suggests for GitHub Enterprise. See `a_github_com_token_never_reaches_another_api_host` in `tests/it/cli.rs`.
+ 🔴 The mock receives `Authorization: Bearer t0ken`, printing `assertion left == right failed: GH_TOKEN is github.com's` with `left: 1`. `src/provider/defaults.rs:62` passes `auth::github_token()`, github.com's token, whatever host the override names.
+ Point `GitLab::new` (or `Redmine::new`) at a server, with a token keyed by `gitlab.com` (or `redmine.example.org`), and ask for an item there. See `gitlab_override_gets_its_own_token_not_the_items_host` and `redmine_override_gets_its_own_key_not_the_items_host` in `src/provider/http_tests.rs`.
+ 🔴 The override's server receives the item host's credential: `assertion left == right failed: Ok(Open)` with `left: 1`. `src/provider/gitlab.rs:59` and `src/provider/redmine.rs:89` look the token up by the item's host but send it to the override. `src/provider/gitea.rs:66` and `src/provider/jira.rs:135` do the same.


== Environment

- `canibump` built from `main` at f018858, on Linux.


== TO-BE

- 🟢 With no `CANIBUMP_GITHUB_API`, or one on `api.github.com`, GitHub's token is still `GITHUB_TOKEN`, `GH_TOKEN`, then `gh auth token`.
- 🟢 With an override on another host, the token is that host's, the way gh picks one: `GH_ENTERPRISE_TOKEN`, `GITHUB_ENTERPRISE_TOKEN`, then `gh auth token --hostname <host>`. Else none.
- 🟢 A GitLab, Gitea, Redmine or Jira override gets the credential keyed by the override's own host (with its port and path, like `REDMINE_URL`), never the item's.
- 🟢 `docs/manifest.md` documents the overrides and their token rule.


== Changes

- `tests/it/cli.rs`: the red test first, then one where `GH_ENTERPRISE_TOKEN` reaches the override.
- `src/provider/http_tests.rs`: the red override tests first, then a token keyed by the override's host reaching it.
- `src/auth.rs`, `src/auth_tests.rs`: the GitHub token for an API URL, github.com's or the Enterprise host's.
- `src/provider/defaults.rs`: pass the API to the token lookup; the module doc states the rule.
- `src/provider/http.rs`: one lookup that keys a credential by where the request goes, used by `gitlab.rs`, `redmine.rs`, `gitea.rs` and `jira.rs` on their token lines; their tests key tokens by the mock's host.
- `docs/manifest.md`: the overrides and their token rule under Auth.


== Out of scope

- Redirect handling. Its own task covers it.
- GitHub Enterprise Cloud with data residency (`*.ghe.com`). gh hands it `GH_TOKEN`; canibump treats it as any other host until someone asks.


== Dogfood

Not applicable. This repo's blockers live on github.com with no override.
