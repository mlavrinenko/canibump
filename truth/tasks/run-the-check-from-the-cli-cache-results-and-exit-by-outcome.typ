#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run the check from the CLI, cache results and exit by outcome",
  kind: "feat",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("cli", "cache"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: helps[`canibump` prints one line per blocker and exits 0, 10, 11 or 12],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a failed check exits 11 and is never cached as fresh],
      offline-readers: helps[`canibump status` and the cache file give every entry's state and check time without network],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the README's exit-code table and output examples come from the tests that pin them],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  )
    + depends-on("read-github-issue-and-pr-state-through-octocrab.typ"),
)

== Problem

+ Run `canibump` in a repo with a `canibump.toml`.
+ 🔴 It prints `Hello from canibump!`.


== TO-BE

- 🟢 `canibump` prints `<status>  <id>  <detail>` per blocker; `--json` prints the same as JSON.
- 🟢 Exit 0 nothing bumpable, 10 something bumpable, 11 a check failed, 12 both.
- 🟢 Results land in `$XDG_CACHE_HOME/canibump/state/<manifest path>.json` with `checked_at` and `fresh_until`; a fresh entry is not re-checked unless `--refresh`.
- 🟢 `canibump status` prints the cache alone, marking stale entries; `--path` prints where it lives.


== Changes

- `tests/cli.rs`: the red tests first, against httpmock.
- `src/cli.rs`: clap commands and the run loop.
- `src/cache.rs`: the state file, TTL and reuse.
- `src/report.rs`: lines, JSON and exit codes.
- `src/main.rs`, `src/lib.rs`: wire it and drop the template's greeter.


== Alternatives

- A cache next to the manifest: every consuming repo would have to gitignore it.


== Out of scope

- Parallel checks. Sequential keeps Repology's one request per second simple.


== Dogfood

`canibump` runs against `~/.nix/canibump.toml` and the cache feeds `canibump status`.

