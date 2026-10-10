#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check GitHub issue and PR state from a canibump.toml manifest",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  kind: "milestone",
  tags: ("cli", "manifest", "cache", "github"),
  estimate: (src-files: 9),
  x-goals: (
    product: (
      one-command: helps[`canibump` prints one line per blocker and maps the outcome to an exit code],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a closed-unmerged PR and a wontfix issue are `dead`, a failed check is `failed`, neither reads as bumpable],
      offline-readers: helps[every result lands in a cache file with its check time and TTL, and `canibump status` reads only that],
      typed-not-scripted: helps[GitHub goes through octocrab behind a provider trait],
    ),
    process: (
      sound-codebase: helps[the evaluator is tested against fake providers, the GitHub client against a mock server],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ Carry a workaround that waits on an upstream PR, e.g. a pin until `github:serde-rs/serde#2590` merges.
+ 🔴 Nothing in the repo says what it waits on; someone has to remember to open the PR now and then.


== TO-BE

- 🟢 `canibump.toml` lists `[[blocker]]` entries with `id`, `note` and a `state = "github:owner/repo#N"` condition.
- 🟢 `canibump` prints one line per entry: `waiting`, `bumpable`, `dead` or `failed`, and `--json` prints the same as JSON.
- 🟢 Exit 0 means nothing bumpable, 10 something bumpable, 11 a check failed, 12 both.
- 🟢 Results go to a cache file with `checked_at` and a TTL; `canibump status` reads only it.
- 🟢 GitHub auth comes from `GITHUB_TOKEN`, `GH_TOKEN` or `gh auth token`, and works without one.


== Changes

- `src/manifest.rs`, `src/refs.rs`: parse the manifest and reference strings into typed conditions.
- `src/eval.rs`, `src/outcome.rs`: evaluate a condition through provider traits into an outcome.
- `src/provider/mod.rs`, `src/provider/github.rs`: the provider traits and the GitHub implementation.
- `src/cache.rs`, `src/report.rs`, `src/cli.rs`: the cache file, the line and JSON output, the exit codes.


== Out of scope

- Reachability, package versions and composites. Each has its own milestone.


== Dogfood

`~/.nix/canibump.toml` gets its first entry once the GitHub state check runs; milestone 6 completes it.

