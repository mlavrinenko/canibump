#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report a blocker whose workaround has no site as unmarked",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: helps[CI fails when a workaround goes but its blocker stays],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a blocker with no workaround left stops reading as a live wait],
      offline-readers: helps[`canibump status` shows sites and scan findings without network],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: depends-on(
    "mark-each-workaround-with-canibump-id-and-print-where-it-sits.typ",
  ),
)

== Problem

+ Configure `[scan]`, mark a workaround `canibump:a`, run `canibump`.
+ Delete the workaround and its marker, keep blocker `a`, run `canibump; echo $?`.
+ 🔴 It prints `waiting  a` and exits 0.
+ Run `canibump status`.
+ 🔴 It prints no sites and no `unlisted` lines: only `check` scans.


== TO-BE

- 🟢 With `[scan]`, a blocker with no site prints `unmarked  a  no canibump:a in the scanned files` and adds 32 to the exit code: 40 alone.
- 🟢 `unmarked` joins `fail-on`, counted by default. `--json` lists the ids under `unmarked`.
- 🟢 `canibump status` scans as `check` does.
- 🟢 Without `[scan]`, nothing is unmarked.


== Changes

- `tests/it/exit_codes.rs`: the red test first, an unmarked blocker exiting 40.
- `src/report.rs`: the `Unmarked` actionable, its bit, the JSON list.
- `src/cli.rs`: scan on `status`.
- `docs/scan.md`, `README.md`, `CHANGELOG.md`, `schema/*`.


== Alternatives

- An opt-in `require-sites` knob: a site outside the scan paths is a config bug worth flagging, and `fail-on` already opts out.
- Fold it into `unlisted`: the fix differs, a marker to add versus a blocker to add.


== Out of scope

- `status <id>`: it answers one blocker by exit code.


== Dogfood

`just canibump` fails when a marker leaves `flake.nix` while its blocker stays.

