#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Deploy the site on every main push",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
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
      sound-codebase: helps[a docs change deploys without spending a crate release],
      docs-match-code: helps[the site rebuilds from every main push, so it tracks the code instead of the last release],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
  links: supersedes(
    "deploy-the-site-on-release-and-trust-cis-pass-on-the-tag.typ",
  ),
)

== Problem

+ Fix a typo in `www/content/quickstart.typ` and push `main`.
+ 🔴 Nothing deploys. `pages.yml` runs only after a Release, and it checks out the highest `v*` tag, so the typo goes live only when a crate release is cut.
+ Cut that release, and `scripts/release-hook.sh` refuses an empty `[Unreleased]`, so a docs fix needs a CHANGELOG entry and a crates.io publish. The docs ride on an app version that did not change.


== TO-BE

- 🟢 `pages.yml` deploys on every push to `main`, on every `v*` tag push, and by hand.
- 🟢 It builds the commit that was pushed, so a docs change is live minutes after the push.
- 🟢 `release.yml`, `release.toml` and the release hook keep no docs role; a release is only ever about the crate.


== Changes

- `.github/workflows/pages.yml`: trigger on `push` to `main` and `v*` tags plus `workflow_dispatch`; drop the `workflow_run`, the deploy `if:` and the tag checkout, so the job builds the pushed commit.
- `www/README.md`: the deploy paragraph says push-to-main, not after-release.
- `outdatty.lock`: refresh, since `www/README.md` is a dependent of `pages.yml`.


== Rollback

- Revert the commit. The site returns to deploying the highest `v*` tag after each release.


== Out of scope

- A "development docs" banner. The maintainer accepted that `main` can run ahead of the last tag, and the gap stays short. Add one only if a reader hits a flag their version lacks.
- Path filters on the trigger. A missed dependency serves a stale site, and the build is minutes.
- Versioned channels, `/` from the tag and `/next/` from `main`. That is the 1.0 upgrade path.


== Dogfood

Not applicable. It changes CI, not canibump's checks.

