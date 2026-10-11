#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Skip the task check in CI until the dev shell has MindTape",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling", "found"),
  estimate: (src-files: 0),
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
      sound-codebase: helps[CI goes green on what passes locally, and says which arm it skipped],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Push main and let `ci.yml` run `nix develop --command just check`.
+ 🔴 `just check-tasks` fails with `mt: invalid argument ‘check’ for ‘operation’`, so `ci.yml` and `release.yml` stay red and `just release` never tags.


== TO-BE

- 🟢 With `CI` set, `just check-tasks` prints `check-tasks: skipped in CI, whose dev shell has no MindTape` and passes.
- 🟢 Locally it still runs `mt check -q`, and the `commit-msg` hook still runs `mt check --commit-msg` on every commit.


== Changes

- `Justfile`: `check-tasks` skips when `CI` is set, with a comment naming the task that lifts it.
- `CONTRIBUTING.md`: the Task Backlog section says CI skips the arm.


== Rollback

Revert the commit; CI goes red on `check-tasks` again.

== Out of scope

- Putting MindTape in the dev shell. `put-mindtape-in-the-dev-shell-so-ci-can-check-the-tasks` does that once MindTape's GitHub flake ships `mt`.


== Dogfood

Not applicable. No upstream item to track; MindTape is unpublished on purpose for now.
