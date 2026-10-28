#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Follow mt 0.3's pack name and template checks in the config",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[mt check and the commit hooks read the backlog again],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Run `mt check` with mt 0.3.0 in this repo.
+ 🔴 It prints `error: .mindtape/config.toml:4: extends: unknown pack: 'mindtape-tasks'` and refuses every command. The pack is `mindtape:tasks` now.
+ With the name fixed, `mt check` prints three template errors, such as `slot `why` is `text`, so `mt add` files the code span in its example as literal backticks`.

== TO-BE

- 🟢 `extends` names `mindtape:tasks`, and `mt check` loads the config.
- 🟢 The slots `context` and `observation` in `finding.typ` and `why` in `tweak.typ` are `kind: "raw"`, so `mt check` passes.
- 🟢 The repo keeps its policy: no rule, status, tag or cap changes.

== Changes

- `.mindtape/config.toml`: rename the pack in `extends`.
- `.mindtape/templates/finding.typ`: make the `context` and `observation` slots raw.
- `.mindtape/templates/tweak.typ`: make the `why` slot raw.

== Rollback

- Revert the commit and pin an older `mt`. No task file changes.

== Out of scope

- Rewriting the open tasks to the new rules. `mt check` is clean without it.

== Dogfood

`mt check` and the commit hooks read this config on every commit.
