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
  estimate: (src-files: 3),
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
      docs-match-code: helps[the `status <id>` codes are documented beside the check exit codes],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: depends-on(
    "fail-the-run-on-any-actionable-status-unless-softened.typ",
  ),
)

== Problem

+ MindTape task `qd0` (~/projects/home/mindtape/truth/tasks/let-a-configured-command-decide-whether-a-task-waits-on-something-outside-the-repo.typ) lets any tool decide a task's readiness by exit code: 0 resolved, 1 waiting, anything else unknown.
+ 🔴 `canibump status --help` takes no blocker id: it prints every entry and exits with the whole manifest's code, so a caller must parse `--json` to learn one entry's state.


== TO-BE

- 🟢 `canibump status <id>` reads only the state file and exits 0 when the entry is bumpable, 1 when waiting, 2 for anything else; it prints that entry's one line, and `--json` a report holding only it.
- 🟢 Anything else covers dead, failed, absent from the manifest, missing from the cache, checked against another condition than the manifest's now, and every error (bad manifest, unreadable state file): the global 1 would read as waiting.
- 🟢 `status <id> --path` is refused; `canibump status` with no id keeps today's behaviour.


== Changes

- `tests/`: red e2e first, one case per answer against a seeded state file, the edited condition and a corrupt state file included.
- `src/cli.rs`: the optional `<id>` argument, its exit codes and its error mapping.
- `src/main.rs`, `src/report.rs`: only if the error mapping needs them.
- README.md, docs/manifest.md, www/index.html: the `status <id>` table beside the check exit codes.


== Alternatives

- Reuse the check exit codes: callers like MindTape read 0 as resolved; a per-entry query answers a yes/no question, unlike the whole-manifest check.
- Leave callers to parse `--json`: every consumer re-implements the same mapping.


== Out of scope

- Refreshing from the network in `status`; `canibump --refresh` stays the only network path.


== Dogfood

~/.nix's MindTape config uses `canibump status {id}` as its checker once MindTape task qd0 lands.
