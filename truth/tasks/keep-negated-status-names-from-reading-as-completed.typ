#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep negated status names from reading as completed",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("trackers", "found"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[`Not done`, `Not fixed` and `Won't do` no longer read bumpable on Redmine or Jira],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[one shared negation check instead of two copies],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Close a Jira issue with the resolution `Not Done`, or move a Redmine issue to a closed status named `Not done` or `Won't do`.
+ 🔴 Whole-word matching finds `done` in `Not Done`, so Jira reads `Completed`. The test `maps_status_category_and_resolution` fails on `CORE-13` with `left: Ok(Completed)` / `right: Ok(ClosedNoReason)`.
+ 🔴 Redmine with `is_closed: true` reads `Completed` for any name it does not know, `Won't do` and `Not fixed` included. Without `is_closed`, `Not done` reads `Completed` too: the test fails on `#16` with `left: Ok(Completed)` / `right: Ok(Open)`.
+ Jira's `statusCategory` already decides done against open. The resolution name is the only field that says why, so it stays the source and needs the negation guard.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 A name with a negator (`not`, `no`, `never`, `won't`, `can't`) never reads `Completed`. Jira gives `ClosedNoReason`, Redmine with `is_closed: true` the same, and Redmine without `is_closed` gives `Open`.
- 🟢 `Won't do` reads `NotPlanned` on Redmine, as it already does on Jira.
- 🟢 The check lives once in `provider/mod.rs` next to `names_any`.


== Changes

- `src/provider/jira_tests.rs`, `src/provider/redmine_tests.rs`: the red cases first, `Not Done`, `Not Fixed`, `Won't Do`.
- `src/provider/mod.rs`: a `negated` helper, with its own unit test.
- `src/provider/jira.rs`, `src/provider/redmine.rs`: guard the completed branch with it; add `won't do` to the Redmine not-planned names.
- `CHANGELOG.md`: the `state` bullet under `[Unreleased]` says negated names do not count.


== Out of scope

- Translated status names. The word lists stay English.


== Dogfood

Not applicable. `~/.nix/canibump.toml` has no Redmine or Jira entry.

