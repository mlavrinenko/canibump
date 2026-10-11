#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let the first release push main before the hook fetches it",
  kind: "docs",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling", "docs", "found"),
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
      sound-codebase: neutral,
      docs-match-code: helps[the release rule in AGENTS.md matches what the hook can do on a fresh origin],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Clone a repo whose GitHub origin has no branches yet, as `mlavrinenko/canibump` had before 0.1.0.
+ Run `just release 0.1.0 --dry-run`.
+ 🔴 `scripts/release-hook.sh` runs `git fetch --quiet origin main` and refuses with `error: can't fetch 'origin main'`.
+ 🔴 `AGENTS.md` still says "don't push main first", which a first release cannot follow.


== TO-BE

- 🟢 `AGENTS.md` says the first push of main is made by hand before the first release, and every later release keeps "don't push main first".
- 🟢 When origin has no main at all, the hook's `hint:` names that way out.


== Source of truth

- `scripts/release-hook.sh`: the `git fetch origin main` refusal.
- `Justfile`: the `release` recipe, which pushes main only after the hook passes.


== Changes

- `AGENTS.md`: reword the release bullet's "don't push main first" so it excepts a repo's first push.
- `scripts/release-hook.sh`: when `git ls-remote --exit-code origin main` finds nothing, refuse with a hint to push main once by hand.
- `CONTRIBUTING.md`: the Releasing section says the same, if it repeats the rule.


== Out of scope

- Making the hook push main itself. Pushing stays the recipe's step after the gates.
- Doing it before 0.1.0 ships. The maintainer pushes main by hand this once.


== Dogfood

Not applicable. It changes the release procedure, not canibump's checks.

