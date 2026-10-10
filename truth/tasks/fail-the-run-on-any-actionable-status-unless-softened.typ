#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail the run on any actionable status unless softened",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: helps[a dead blocker fails the run like a bumpable one, and a softer run says so in one flag],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[an upstream fix closed unmerged is never silent],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[one exit table in `--help`, README, docs and the site],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
)

== Problem

+ Point a `canibump.toml` blocker at a PR closed without merging, then run `canibump`.
+ 🔴 It prints `dead` and exits 0, so a cron job or CI step never notices the workaround needs rethinking.
+ 🔴 No run can say only failures matter: the exit code always counts bumpable and failed.


== TO-BE

- 🟢 `canibump` and `canibump status` exit 0 when nothing counted showed up, else 8 plus 1 for bumpable, 2 for dead and 4 for failed, summed: bumpable and dead together exit 11. 1 stays an unusable manifest, cache or argument.
- 🟢 `--fail-on bumpable,failed` (global), else `[settings] fail-on = ["failed"]`, else all three, picks what counts; `fail-on = []` never fails.
- 🟢 `--help`, README, docs/manifest.md, www and the `one-command` acid test in truth/goals.typ state the same table.


== Changes

- `tests/`: red e2e first: a dead blocker exits 10, `--fail-on failed` exits 0 on the same state, a settings `fail-on` applies and the flag overrides it.
- `src/report.rs`: the exit bits and the counted set.
- `src/cli.rs`: the global `--fail-on` flag and the exit table in `long_about`.
- `src/manifest.rs`: `[settings] fail-on`, then `just schema`.
- `truth/goals.typ`: the `one-command` acid test.
- README.md, docs/manifest.md, www/index.html, CHANGELOG.md (Changed).


== Alternatives

- Keep 10/11/12 and number each combination by hand: seven codes nobody remembers, where bits test as `(( code & 2 ))`.
- One code for anything actionable: scripts parse `--json` to tell a landed fix from a broken check.
- `--allow dead`, naming what to forgive: a list of what counts reads straight, and its default prints in `--help`.


== Out of scope

- `canibump status <id>`, which answers by its own 0/1/2 (answer-one-blockers-cached-state-by-exit-code).
- Per-blocker overrides, until a manifest needs one.


== Dogfood

`just canibump` here and in ~/.nix exits non-zero once a tracked upstream PR closes unmerged.

