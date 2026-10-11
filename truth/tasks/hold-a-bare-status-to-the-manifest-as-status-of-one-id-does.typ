#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold a bare status to the manifest as status of one id does",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "cache", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: helps[`canibump status` and `canibump status <id>` agree],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a bare status no longer reports a blocker as bumpable against a condition the manifest has since changed],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `canibump`, change a blocker's `state = "github:o/r#2"` to `#9`, then run `canibump status`.
+ 🔴 It prints `bumpable  bumpable  github:o/r#2 closed as completed (drop the bumpable patch)` and exits 15 (`a_bare_status_does_not_count_a_result_of_an_older_condition` fails with `left: Some(15), right: Some(0)`), while `canibump status bumpable` answers 2 with "was checked against an older condition".
+ 🔴 Delete a blocker from the manifest: bare `status` keeps listing its cached entry and counts it in the exit code.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Bare `status` treats an entry whose condition differs from the manifest's as stale: it prints no line for it, counts it in no exit code, and says so on stderr (``canibump: warning: `bumpable` was checked against an older condition: run `canibump` ``).
- 🟢 Bare `status` skips an entry whose blocker the manifest dropped, silently, in the lines, in `--json` and in the exit code.


== Changes

- `tests/it/status_id.rs`: the red tests first, a changed condition, a dropped blocker and the `--json` form.
- `src/cli.rs`: `status` keeps only the entries the manifest still holds under the same condition and warns for each changed one.
- `CHANGELOG.md`: refine the `status` bullet under `[Unreleased]`.


== Out of scope

- A blocker the manifest holds but the state file lacks. It stays unlisted, as today.


== Dogfood

Not applicable, `just canibump` checks first and reads the answer after.

