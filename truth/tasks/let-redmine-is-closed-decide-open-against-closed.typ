#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let Redmine is_closed decide open against closed",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("trackers", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a Redmine issue in the default Resolved status, which is_closed marks open, no longer reads bumpable],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[table cases pin is_closed false against closing names],
      docs-match-code: helps[names stand in only when `is_closed` is missing, as the module doc says],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Point `state = "redmine:host#N"` at an issue in Redmine's default `Resolved` status. The server sends `{"name": "Resolved", "is_closed": false}`.
+ 🔴 `state_of` tests `status.is_closed == Some(true) || has(["closed", "resolved", "done", "fixed"])`, so the name wins, the issue reads `Completed` and the entry prints `bumpable` while the issue is open. The test `maps_status_names_and_is_closed` fails on `#10` with `left: Ok(Completed)` / `right: Ok(Open)`.
+ The module doc says names stand in only on instances that omit `is_closed`.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 When `is_closed` is present it decides open against closed. `false` reads `Open` whatever the name says, `Rejected` and `Duplicate` included.
- 🟢 When it is `true`, the name only picks the reason: not planned, duplicate, else completed.
- 🟢 When it is absent, names stand in as today.


== Changes

- `src/provider/redmine_tests.rs`: the red cases first, `Resolved`, `Fixed` and `Rejected` with `is_closed: false` reading `Open`.
- `src/provider/redmine.rs`: branch on `is_closed` first; fix the module doc to match.
- `CHANGELOG.md`: the `state` bullet under `[Unreleased]` says what decides.


== Out of scope

- Negated names such as `Not done`. They get their own task.


== Dogfood

Not applicable. `~/.nix/canibump.toml` has no Redmine entry.

