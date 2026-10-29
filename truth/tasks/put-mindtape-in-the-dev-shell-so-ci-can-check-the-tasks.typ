#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Put MindTape in the dev shell so CI can check the tasks",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
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
      sound-codebase: helps[`just check` passes in CI on what passes locally],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 29),
)

== Problem

+ Push main and let `ci.yml` run `nix develop --command just check`.
+ 🔴 `just check-tasks` fails with `mt: invalid argument ‘check’ for ‘operation’`: the dev shell has no MindTape, so `mt` is the runner's magnetic-tape tool.
+ 🔴 Locally it passes only because `mt` is installed on the host from a MindTape checkout 1806 commits ahead of `github:mlavrinenko/mindtape`, whose flake still ships a `mindtape` binary with another CLI.


== TO-BE

- 🟢 The dev shell takes `mt` from qahq, and it is the one `just check-tasks` and the git hooks run, locally and in CI.
- 🟢 `ci.yml`'s `Check, Lint, Test` goes green on main.


== Changes

- `flake.nix`: qahq's `mindtape` package in the `rust` shell, which the default shell includes.
- `flake.lock`: qahq bumped to the revision that ships it.
- `Justfile`: `check-tasks` runs `mt check` in CI too.


== Rollback

Revert the commit; the host's `mt` serves locally again, and CI stays red on `check-tasks`.

== Out of scope

- A `mindtape` input of this flake's own: qahq already pins one beside the other gates.


== Dogfood

Not applicable. It changes the dev shell, not canibump's checks.
