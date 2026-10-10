#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Order versions by one total order",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("packages", "git", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[`in = { tags = "v*" }` names the true first tag, and `at-least` compares against it consistently],
      honest-state: helps[a mixed tag list can no longer make the binary search name a tag that lacks the commit],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[a transitivity test over a mixed tag corpus pins the order],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ Compare three tags with `version::compare`: `("1.0.0", "1.0.0+zz")`, `("1.0.0+zz", "1.0.0zzz")` and `("1.0.0zzz", "1.0.0")`.
+ 🔴 All three return `Less`: a cycle. `compare` picks semver when both sides parse, else the lenient order, and mixing two orders by pair is not transitive.
+ 🔴 `version::sort` and the binary search in `eval::first_containing` need a total order, so a mixed tag list can name the wrong first tag. Rust 1.81+ `sort_by` may also panic on such a comparator, and `panic = "abort"` then crashes with no state written.
+ 🔴 The orders disagree on everyday tags: semver puts `1.0.0-rc10` before `1.0.0-rc9`.
+ `compare` re-parses both strings on every call of an O(n log n) sort.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 One version key type whose `Ord` is total by construction, built once per string. `compare` compares keys, `sort` uses `sort_by_cached_key`, and `eval_version` takes the max by key.
- 🟢 It keeps today's documented behaviour. A letter prefix drops (`v1.0.0` = `1.0.0`, `b7012` = `7012`). A letter run after the numbers is a pre-release (`1.0rc1` < `1.0`, `1.0.0-rc.1` < `1.0.0`). `1.2` < `1.2.1`. Every existing test in `src/version_tests.rs` passes unchanged.
- 🟢 Build metadata after `+` is ignored, as semver says (`1.0.0+zz` = `1.0.0`), and `1.0.0-rc9` < `1.0.0-rc10`.
- 🟢 A corpus test checks antisymmetry and transitivity over every triple of a mixed list: semver, `v`-tags, `b`-builds, rc, `+build`, dates such as `2024.01`.


== Changes

- `src/version_tests.rs`: the red tests first. The cycle above, `rc9` < `rc10`, `+build` equality, then the corpus test.
- `src/version.rs`: the key type, with the semver/lenient split removed.
- `src/eval.rs`: take the highest version by key, if it does not already go through `compare`.
- `Cargo.toml`: drop `semver` once nothing uses it.
- `docs/manifest.md`, `README.md`: the sentence on version order ("semver, or lenient") describes the one order.


== Out of scope

- Ecosystem-specific orders (PEP 440 epochs, Maven qualifiers). The one lenient order covers the tags canibump reads today.


== Dogfood

The `released` step in `~/.nix/canibump.toml` sorts llama.cpp's `v*` tags with the new key.

