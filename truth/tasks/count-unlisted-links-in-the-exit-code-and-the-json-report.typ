#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count unlisted links in the exit code and the JSON report",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: helps[CI fails on a forgotten workaround through the same exit code bits],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: helps[`--json` carries each unlisted link with its file and line],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
  links: depends-on("report-upstream-links-that-no-blocker-lists.typ"),
)

== Problem

+ Configure `[scan]` and leave a GitHub issue link in a scanned file with no blocker.
+ Run `canibump; echo $?`.
+ 🔴 It prints the `unlisted` line but exits 0, so CI passes and nobody sees it.


== TO-BE

- 🟢 An unlisted link adds 16 to the exit code: 8 + 16 = 24 on its own.
- 🟢 `unlisted` joins `fail-on` and `--fail-on`, counted by default with the other three.
- 🟢 `--json` has an `unlisted` array of `{file, line, ref, catcher}`.


== Changes

- `tests/it/exit_codes.rs`: the red test first, an unlisted link exiting 24.
- `src/report.rs`: the `Unlisted` actionable, its bit, the JSON rows.
- `src/cli.rs`: hand the scan's hits to the exit code and the report.


== Alternatives

- Leave the exit code alone: an advisory line in CI logs is read by nobody.
- Fold unlisted into bit 1 (bumpable): the caller could not tell a landed fix from a missing blocker.


== Out of scope

- The state file and `canibump status`: they hold blocker results, and a scan is cheap to rerun.


== Dogfood

`just canibump` exits non-zero when a workaround comment lands in this repo without a blocker.

