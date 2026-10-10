#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace just fork and just integrate with worktrunk",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 4),
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
      sound-codebase: helps[one copy of the delegation hooks, in ~/.nix, instead of a per-repo port that drifts],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ `.just/scripts/fork.sh` and `integrate.sh` (301 lines) plus two recipes ported mindtape's delegation here on 2026-10-10.
+ 🔴 ~/.nix now ships worktrunk with user hooks doing the same (`apps/worktrunk.nix`), and mindtape already deleted its copy, so this port is a second copy that drifts.


== TO-BE

- 🟢 `wt switch --create <slug>` and `wt merge` delegate and land work here; the scripts and recipes are gone.
- 🟢 AGENTS.md says so; nothing names a specific agent tool.


== Changes

- `.just/scripts/fork.sh`, `.just/scripts/integrate.sh`: deleted.
- `Justfile`: the `fork` and `integrate` recipes removed.
- `outdatty.yaml`: `.just/scripts/**` out of `dev-docs`, if nothing else lives there.
- AGENTS.md: the delegation bullet.


== Rollback

Revert the commit; the scripts come back unchanged.


== Out of scope

- The user hooks themselves, which live in ~/.nix.


== Dogfood

The next delegated task here forks with `wt switch --create` and lands with `wt merge`.

