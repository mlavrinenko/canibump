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

- `release.toml`: new, modelled on ~/projects/home/mindtape/release.toml (mindtape commits c914a88b, 14580544). CHANGELOG.md here has no version link lines yet, so the replacement is a plain `## [Unreleased]` to `## [Unreleased]` plus `## [{{version}}] - {{date}}`, with `exactly = 1`.
- `.just/scripts/release-hook.sh`: modelled on mindtape's. It holds every refusal of today's `release` recipe (Justfile, from line 370), with the same messages and hints. It reads `NEW_VERSION`, `DRY_RUN` and `WORKSPACE_ROOT`, and passes `--allow-dirty` to `cargo publish --dry-run`.
- `Justfile`: `release VERSION $DRY_RUN=''`, built on the cargo-release steps `version replace hook commit`, then push, `gh run watch` on ci.yml, then `tag` and `push`.
- `flake.nix`: `cargo-release` in the dev shell.
- AGENTS.md, CONTRIBUTING.md: how releasing works.

== Rollback

Revert the commit; release.yml is untouched, so either recipe can cut a tag.


== Out of scope

- `.github/workflows/release.yml`, which keeps publishing on the tag.


== Dogfood

canibump's first release, 0.1.0, goes through it after `just release 0.1.0 --dry-run`.

