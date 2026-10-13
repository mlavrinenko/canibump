#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Deploy the site on release and trust CI's pass on the tag",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
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
      sound-codebase: helps[a release spends no runner minutes proving what ci.yml proved on the same commit],
      docs-match-code: helps[the site documents the version crates.io serves, never unreleased main],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: depends-on("split-ci-by-toolchain.typ"),
)

== Problem

+ Run `just release X.Y.Z` with a change on main that `ci.yml` then fails.
+ 🔴 `pages.yml` already built and deployed main's docs on the push, so the site describes a version nobody can install, while `scripts/release-ci.sh` refuses the tag.
+ Run it again with CI green.
+ 🔴 `release.yml`'s `check` enters Nix and runs `just check` on the commit `ci.yml` just passed.


== TO-BE

- 🟢 `pages.yml` runs after `release.yml` succeeds, or by hand, and deploys the highest `v*` tag, so a re-run of an old release never rolls the site back.
- 🟢 `release.yml` refuses a tag whose commit has no green `ci.yml` run, and spends seconds on it.


== Changes

- `.github/workflows/pages.yml`: `workflow_run` on Release and `workflow_dispatch`; checks out the highest `v*` tag; the Nix action.
- `.github/workflows/release.yml`: `check` asks the Actions API for a green `ci.yml` run on the tag's commit.


== Rollback

- Revert the commit.


== Out of scope

- Reusing CI's release binaries. Artifacts expire, and a re-run of an old tag must still build.


== Dogfood

Not applicable. It changes CI, not canibump's checks.
