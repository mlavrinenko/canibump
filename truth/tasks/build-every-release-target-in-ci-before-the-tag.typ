#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build every release target in CI before the tag",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
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
      sound-codebase: helps[a target that fails to build stops the release before its tag, not after],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Break the build of one release target only, like `x86_64-pc-windows-msvc` or `aarch64-unknown-linux-gnu`.
+ Run `just release X.Y.Z`. It waits for `ci.yml`, which builds on Linux through Nix only, so it goes green and the tag is pushed.
+ 🔴 `release.yml`'s `build` fails on the tag, `publish` never runs, and a fix needs a new commit the pushed tag can never point at, so the version is burned.


== TO-BE

- 🟢 `ci.yml` builds every target `release.yml` ships on each push to main, so `scripts/release-ci.sh` waits on it before tagging.
- 🟢 One workflow holds the target matrix; `ci.yml` and `release.yml` both call it.


== Changes

- `.github/workflows/build.yml`: a reusable workflow with the target matrix, taking the ref to check out and whether to upload the binaries.
- `.github/workflows/release.yml`: `build` calls it on the tag, uploading.
- `.github/workflows/ci.yml`: a `build` job calls it on pushes to main and dispatches, not on pull requests.


== Rollback

Revert the commit; `release.yml` gets its inline matrix back.

== Out of scope

- Running the tests on every target. A build is what `release.yml` needs.
- Pull requests. The matrix is slow, and only main gets tagged.


== Dogfood

Not applicable. It changes CI, not canibump's checks.
