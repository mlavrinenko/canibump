#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Release through cargo-release instead of the hand-rolled recipe",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
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
      sound-codebase: helps[the version bump and changelog dating stop being hand steps],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ `just release X.Y.Z` is a 95-line inline recipe that refuses unless Cargo.toml is already bumped and CHANGELOG.md already dated by hand.
+ 🔴 mindtape replaced the same recipe with cargo-release (its task `release-through-cargo-release-instead-of-a-hand-rolled-script`), so canibump keeps the older, longer copy.


== TO-BE

- 🟢 `release.toml` configures cargo-release: `v<version>` tag, `publish = false` (release.yml publishes on the tag), and `pre-release-replacements` that date `[Unreleased]`.
- 🟢 `just release X.Y.Z [--dry-run]` bumps, dates, runs a pre-release hook and commits, pushes main, waits for ci.yml, then tags and pushes the tag; every refusal of today's recipe still happens.


== Changes

- `release.toml`: new, modelled on ~/projects/home/mindtape/release.toml.
- `.just/scripts/release-hook.sh`: the checks cargo-release lacks, modelled on mindtape's.
- `Justfile`: the shorter `release` recipe.
- `flake.nix`: `cargo-release` in the dev shell.
- AGENTS.md, CONTRIBUTING.md: how releasing works.


== Rollback

Revert the commit; release.yml is untouched, so either recipe can cut a tag.


== Out of scope

- `.github/workflows/release.yml`, which keeps publishing on the tag.


== Dogfood

canibump's first release, 0.1.0, goes through it after `just release 0.1.0 --dry-run`.

