#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Answer one blocker's cached state by exit code",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli", "cache"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a missing or failed cache entry answers unknown, never resolved],
      offline-readers: helps[a task tracker asks about one blocker with one offline call and reads the answer from the exit code],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the exit-code contract is documented next to the existing 0/10/11],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ MindTape task `qd0` (~/projects/home/mindtape/truth/tasks/let-a-configured-command-decide-whether-a-task-waits-on-something-outside-the-repo.typ) lets any tool decide a task's readiness by exit code: 0 resolved, 1 waiting, anything else unknown.
+ 🔴 `canibump status --help` takes no blocker id: it prints every entry and exits 0, so a caller must parse `--json` to learn one entry's state.


== TO-BE

- 🟢 `canibump status <id>` reads only the state file and exits 0 when the entry is bumpable, 1 when waiting, 2 when it is dead, failed, missing from the cache or absent from the manifest; it prints that entry's one line.
- 🟢 `canibump status` with no id keeps today's behaviour.


== Changes

- `tests/`: red e2e first, one entry per state against a seeded state file.
- `src/cli.rs` (or the status command's module): the optional `<id>` argument and its exit codes.
- `docs/` and README: the exit-code table for `status <id>`.


== Alternatives

- Reuse 10 for bumpable: callers like MindTape read 0 as resolved; a per-entry query answers a yes/no question, unlike the whole-manifest check.
- Leave callers to parse `--json`: every consumer re-implements the same mapping.


== Out of scope

- Refreshing from the network in `status`; `canibump --refresh` stays the only network path.


== Dogfood

~/.nix's MindTape config uses `canibump status {id}` as its checker once MindTape task qd0 lands.
