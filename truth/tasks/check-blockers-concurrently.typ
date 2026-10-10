#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check blockers concurrently",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "forges", "packages", "git"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: helps[a run takes about as long as its slowest blocker, not the sum of all of them],
      many-targets: helps[a manifest can grow to dozens of blockers without the run growing with it],
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
  status: done(2026, 10, 10),
  links: depends-on("start-a-provider-only-when-a-blocker-needs-it.typ"),
)

== Problem

+ `cd ~/.nix && time canibump --refresh`: one GitHub blocker, about 1.9 s, nearly all of it waiting on the network.
+ 🔴 `src/run.rs` awaits each blocker in a `for` loop, so a run costs the sum of every blocker's round trips: a 15-blocker manifest waits about 30 s, on a tokio runtime that could overlap them all.


== TO-BE

- 🟢 Blockers run concurrently, at most 8 in flight, and entries come back in manifest order. Steps inside one `all`/`any` stay sequential, since they hand values on.
- 🟢 Politeness holds: Repology stays at one request a second, and concurrent asks for one project share a single request.
- 🟢 One git remote is fetched by one worker at a time (a lock per URL), so two blockers on one remote never race `init_bare` or a fetch.
- 🟢 A test with a fake provider that sleeps shows N blockers finishing in about one delay, not N.


== Changes

- `src/run.rs` (+ tests): the red timing test first, then an ordered bounded map (`futures_util::stream::iter(...).buffered(8)` borrows the manifest, keeps order, needs no `'static`).
- `src/provider/repology.rs`: single-flight per project (a per-project once-cell), keeping the one-second spacing.
- `src/provider/git.rs`: a lock per remote URL around open-or-init and fetch.
- `Cargo.toml`: `futures-util` if it is not reachable already (it is in the tree through reqwest).


== Alternatives

- `tokio::spawn` per blocker: needs `'static` data, so clones of the manifest and registry, and reorders results by hand.
- A `JoinSet` with indices: same ordering bookkeeping that `buffered` gives for free.


== Out of scope

- Deduplicating the same item asked by two blockers: the memoization task covers it.


== Dogfood

`canibump --refresh` in `~/.nix` stays as fast as its single chain; a second blocker adds no wall time.

