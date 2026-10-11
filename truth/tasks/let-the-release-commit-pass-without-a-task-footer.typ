#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let the release commit pass without a task footer",
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
      sound-codebase: helps[`just release` commits its bump without a hand-made task],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `just release 0.1.0`.
+ 🔴 cargo-release commits `chore(release): 0.1.0`, and the `commit-msg` hook refuses it: `commits.rules.work-names-a-task: name the task this commit works on in a Refs: <stem> footer`. Nothing is pushed or tagged.


== TO-BE

- 🟢 A `chore(release): X.Y.Z` commit passes `mt check --commit-msg` with no `Refs:` footer.
- 🟢 Every other commit outside `truth/` still needs one.


== Changes

- `.mindtape/config.toml`: override the pack's `work-names-a-task` so its `when` skips a `chore(release):` subject.


== Rollback

Revert the commit; the release commit is refused again.

== Out of scope

- A task per release. The version bump is no work of its own; the CHANGELOG entries it dates were each committed under their task.


== Dogfood

Not applicable. It changes the commit rules, not canibump's checks.
