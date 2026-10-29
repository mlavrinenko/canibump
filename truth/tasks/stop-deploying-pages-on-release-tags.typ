#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop deploying Pages on release tags",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "found"),
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
      sound-codebase: helps[a release no longer ends with a red Deploy Pages run],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 30),
)

== Problem

+ Push a `v*` tag, as `just release 0.2.1` does.
+ 🔴 The Deploy Pages run fails: `Tag "v0.2.1" is not allowed to deploy to github-pages due to environment protection rules.`


== TO-BE

- 🟢 `pages.yml` triggers on pushes to `main` and by hand only; a release tag starts no Pages run.


== Changes

- `.github/workflows/pages.yml`: drop the `tags: ["v*"]` trigger and fix its comment.
- `www/README.md`: stop saying a `v*` tag deploys.


== Rollback

- Revert the commit; the tag trigger returns.


== Out of scope

- The `github-pages` environment's branch policy, which rightly admits only main.


== Dogfood

The next release ends with every workflow green.

