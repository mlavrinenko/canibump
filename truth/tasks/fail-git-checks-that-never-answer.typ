#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail git checks that never answer",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("git", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: helps[a stalled remote no longer hangs the whole run],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a check that never answers reads failed],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[a virtual-time test pins the deadline without network],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Point a `reached` blocker at an `ssh://` remote that prompts for a password or stalls mid-fetch, and run `canibump`.
+ 🔴 `GitRemotes::blocking` in `src/provider/git.rs` runs the gix work in `spawn_blocking` and awaits it with no deadline, so the run waits forever. HTTP checks stop after 30 s, git ones never do.
+ 🔴 A stand-in that sleeps 5 s, under a paused clock, comes back `Ok(())` after the full 5 s: `a_git_call_that_never_answers_reads_failed` panics with `Ok(())`. Even with a timeout added, tokio's `Runtime` drop waits for every `spawn_blocking` task, so the process would still hang on exit.


== Environment

- `canibump` built from `main` at f018858, on Linux.


== TO-BE

- 🟢 A git check that gives no answer within its deadline reads `failed` with a message naming the source and the wait, and the run goes on.
- 🟢 The abandoned worker never holds up the end of the run: it runs on a detached thread, so dropping the runtime returns at once.


== Changes

- `src/provider/git_tests.rs`: the red tests first. A stalled call reads `Network` under virtual time, and dropping the runtime after it returns at once.
- `src/provider/git.rs`: run the work on a detached thread, await its answer under `tokio::time::timeout`, and map the timeout to `CheckError::Network`.


== Out of scope

- ssh options such as `BatchMode`. The deadline covers a prompt and a stall alike without overriding the user's `core.sshCommand`.
- Interrupting the abandoned fetch. It dies with the process, and its pack lands through a temp file.


== Dogfood

Not applicable. This repo's `canibump.toml` has no `git:` remote.

