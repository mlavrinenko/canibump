#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Choose which statuses print their sites, and print all of them in status",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "manifest"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: helps[`canibump status` shows where every workaround sits],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: helps[`status <id>` names its blocker's sites without network],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Mark a waiting blocker's workaround `canibump:a`, run `canibump status`.
+ 🔴 It prints `waiting  a  …` and no `at` line: sites print only under bumpable and dead lines, with no way to ask for more.
+ Run `canibump status a`.
+ 🔴 It prints no sites either: `status <id>` does not scan.


== TO-BE

- 🟢 `[settings] sites-on = ["waiting", "bumpable", "dead", "failed"]` picks which lines print their `at` line; it defaults to bumpable and dead.
- 🟢 `--sites-on waiting,bumpable` overrides it, as `--fail-on` does `fail-on`.
- 🟢 `canibump status` and `canibump status <id>` print sites under every line unless `--sites-on` says otherwise; `status <id>` scans to find them.
- 🟢 The exit codes stay as they are.


== Changes

- `tests/it/status_id.rs`, `tests/it/cli.rs`: the red tests first: `status` and `status a` print a waiting line's sites, `--sites-on` and `sites-on` pick the statuses.
- `src/report.rs`: `lines` takes the statuses that print sites.
- `src/manifest.rs`: `sites-on`.
- `src/cli.rs`: `--sites-on`, and resolving it per command; `status <id>` scans.
- `src/outcome.rs`: `Status` parses as a flag value.
- `docs/scan.md`, `docs/manifest.md`, `CHANGELOG.md`, `schema/*`.


== Alternatives

- Sites under every line always: doubles the output of a check for nothing to act on.
- A boolean `all-sites`: cannot ask for, say, failed alongside bumpable.


== Out of scope

- Unlisted and unmarked lines in `status <id>`: it answers for one blocker.


== Dogfood

`canibump status` in this repo prints `at flake.nix:96` under `llvm-cov-path-fallback`.

