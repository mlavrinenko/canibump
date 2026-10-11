#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Find the cache dir on Windows, where HOME is unset",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("cache", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: helps[the Windows release binary runs without a `--cache` flag],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
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

+ On Windows, or anywhere with neither `XDG_CACHE_HOME` nor `HOME` set, run `canibump` next to a `canibump.toml`.
+ 🔴 It prints `canibump: no cache dir: set XDG_CACHE_HOME or HOME, or pass --cache` and exits 1, though `release.yml` ships an `x86_64-pc-windows-msvc` binary.


== Environment

- Commit `b312daa` on main, before 0.1.0.
- Reproduced on Linux with `env -u HOME -u XDG_CACHE_HOME canibump`.


== TO-BE

- 🟢 With neither set, the state file and git clones land under `%LOCALAPPDATA%\canibump`.
- 🟢 The `no cache dir` error names `LOCALAPPDATA` too.


== Changes

- `src/cache_tests.rs`: the red test first, `cache_home` falling back to `LOCALAPPDATA`.
- `src/cache.rs`: `cache_home` tries `LOCALAPPDATA` after `HOME`; the `NoHome` message names it.
- `docs/manifest.md`: the default path names the Windows fallback.


== Out of scope

- Reading glab's and tea's configs from their Windows dirs. Tokens from `GITLAB_TOKEN`, `GITEA_TOKEN` and `gh` still work there.


== Dogfood

Not applicable. This repo runs canibump on Linux only.
