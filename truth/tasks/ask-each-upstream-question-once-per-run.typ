#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ask each upstream question once per run",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("github", "forges"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: helps[fewer round trips make every network run faster],
      many-targets: neutral,
      whole-chain: helps[a chain that names the same PR in several steps pays for it once],
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
  status: proposed(2026, 10, 10),
  links: depends-on("start-a-provider-only-when-a-blocker-needs-it.typ"),
)

== Problem

+ Run the `~/.nix` chain, whose `merged` step and `released` step both name `github:ggml-org/llama.cpp#26487`, against a mock GitHub and count hits.
+ 🔴 A PR's state costs two calls: `/issues/N`, then `/pulls/N`. A `reached` on a PR costs three: `/issues/N` and `/pulls/N` for the state, then `/pulls/N` again for the merge commit.
+ 🔴 So the chain makes five calls before the tag search where two answer it. Unauthenticated GitHub allows 60 an hour.


== TO-BE

- 🟢 Within one run, each item's state and each merge commit is fetched once, however many steps or blockers ask; concurrent askers share one in-flight request.
- 🟢 GitHub reads a PR's state from `/issues/N` alone: its `pull_request.merged_at` says whether it merged. `/pulls/N` is fetched only for the merge commit.
- 🟢 The dogfood chain makes exactly two calls before the tag search, pinned by httpmock hit counts.


== Changes

- `src/provider/github_tests.rs`: the red tests first, hit counts for a merged PR's state and for state plus merge commit.
- `src/provider/github.rs`: read `pull_request.merged_at` from the issue.
- `src/provider/memo.rs` (new) or `src/provider/mod.rs`: a memoizing wrapper for `Tracker::item_state` and `CodeHost::merge_commit`, keyed by `ItemRef`, single-flight, that the registry puts around every tracker and code host.
- `src/eval_tests.rs`: a chain asking one item twice hits the fake once.


== Alternatives

- Memoize inside each provider: the same cache code six times; one wrapper covers all.
- Memoize `contains` and `tags` too: tag probes rarely repeat within a run, so the gain is small for the extra keys.


== Out of scope

- Caching across runs: the state file already does that per blocker.


== Dogfood

The `~/.nix` chain spends two GitHub calls before its tag search instead of five.

