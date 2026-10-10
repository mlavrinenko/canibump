#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read Redmine and Jira issue state",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("trackers",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[`redmine:host#N` and `jira:host#KEY-N` check like any other item],
      whole-chain: neutral,
      honest-state: helps[Rejected, won't fix and duplicate resolutions read dead, never bumpable],
      offline-readers: neutral,
      typed-not-scripted: helps[two thin typed reqwest clients behind `Tracker`],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: parent("add-gitlab-gitea-forgejo-redmine-and-jira-providers.typ")
    + depends-on("read-gitlab-issue-and-mr-state-self-hosted-included.typ"),
)

== Problem

+ Declare `state = "redmine:redmine.example.org#123"`.
+ 🔴 It fails: `unknown scheme redmine`.


== TO-BE

- 🟢 Redmine maps its status name and `is_closed`; Jira maps the status category and resolution.
- 🟢 `REDMINE_API_KEY` goes to `REDMINE_URL`'s host; `JIRA_TOKEN`, or `JIRA_USER` with `JIRA_API_TOKEN`, to `JIRA_URL`'s host.


== Changes

- `src/provider/redmine_tests.rs`, `src/provider/jira_tests.rs`: the red tests first.
- `src/refs.rs`: the `redmine:` and `jira:` schemes.
- `src/provider/redmine.rs`, `src/provider/jira.rs`: the clients.
- `src/auth.rs`, `src/provider/defaults.rs`: credentials and registration.


== Alternatives

- Per-tracker status config in the manifest: a knob nobody needs until a tracker's names surprise them.


== Out of scope

- Custom Jira workflows beyond status category and resolution.


== Dogfood

Not applicable: `~/.nix` waits on nothing in Redmine or Jira.

