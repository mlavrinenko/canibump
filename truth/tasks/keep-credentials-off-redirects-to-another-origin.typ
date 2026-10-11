#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep credentials off redirects to another origin",
  kind: "fix",
  priority: ice(
    impact: 9,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("forges", "trackers", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a redirect off the instance fails loudly instead of answering for another host],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[two-server httpmock tests pin that no credential header crosses an origin],
      docs-match-code: helps["A token only ever goes to its own host" holds for redirects too],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Point `GitLab::new` with a token at an httpmock server that answers `302` to a second server, and ask for `gitlab:group/proj#1`. Same for Redmine with a key. See `src/provider/http_tests.rs`.
+ 🔴 The second server gets the credential: `assertion left == right failed: Ok(Open)`, `left: 1`, in `gitlab_token_stays_off_a_redirect_to_another_origin` and `redmine_key_stays_off_a_redirect_to_another_origin`.
+ The client in `src/provider/http.rs` follows 10 redirects. reqwest 0.13.5 `remove_sensitive_headers` (`redirect.rs:239`) strips only `Authorization`, `Cookie`, `Proxy-Authorization` and `WWW-Authenticate` across hosts, ports or schemes. `PRIVATE-TOKEN` (`src/provider/gitlab.rs:61`) and `X-Redmine-API-Key` (`src/provider/redmine.rs:91`) ride along.
+ Gitea and Jira send `Authorization`: the Gitea test passes. octocrab's `FollowRedirectLayer::new()` drops `Authorization` across origins too.


== Environment

- `canibump` built from `main` at f018858, on Linux, reqwest 0.13.5.


== TO-BE

- 🟢 The shared client follows a redirect only within the origin (scheme, host, port) the request started on, up to 10 hops. A hop elsewhere fails with a network error naming the refused URL, so no credential header of any name reaches another origin.
- 🟢 This holds for every request, tokened or not: a reqwest policy sees URLs, not headers. No API canibump calls redirects across origins in normal use, and a loud error beats a quiet leak. `Policy::none()` would also stop harmless same-origin hops, so it loses.
- 🟢 Two httpmock servers per provider (GitLab, Redmine, Gitea) show the second never gets the request.


== Changes

- `src/provider/http_tests.rs`: the red tests first, two servers per provider, plus a same-origin redirect that still follows.
- `src/provider/http.rs`: a custom redirect policy that refuses a cross-origin hop.
- `CHANGELOG.md`: the tracker bullets under `[Unreleased]` say a token never follows a redirect off its host.


== Out of scope

- octocrab's redirect layer. It already drops `Authorization` across origins.
- Per-request policies. reqwest sets the policy per client, and two clients per provider buy nothing while no API redirects across origins.


== Dogfood

`just canibump` checks this repo's own GitHub blockers; repology and deps.dev go through the same client and keep working.
