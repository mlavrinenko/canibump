#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a step reference that can never have a value",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("manifest", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[a chain that hands a tag on is refused at load unless the tag is sure to exist],
      honest-state: helps[a blocker no longer reads `failed` forever because of its own shape],
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
)

== Problem

+ Write a blocker with `[[blocker.any]]` steps: first `id = "released"`, `reached = "github:o/r#1"`, `in = { tags = "v*" }`; then `version = "deps.dev:cargo/x"`, `at-least = { step = "released" }`.
+ 🔴 `canibump` loads it, because `check_steps` accepts a reference to any earlier step id.
+ 🔴 While `released` waits, the version step reads `` failed: step `released` found no tag ``, `any` ranks failed first, and every run exits 11 and re-hits the network, since a failure is never fresh. Once `released` holds, the version step never runs. The reference can never help.
+ 🔴 Same for a reference into an earlier `any` inside an `all`: when that `any` resolves through another step, the named step has no value and the blocker reads failed.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Loading refuses a `step` reference unless the named step surely held when the reference runs: an earlier step of the same `all`, or of an enclosing `all` reached only through `all`s. The error names the fix, e.g. `` blocker `x`: `at-least` names step `released`, which sits in an `any` and may not have held; put both in one `all` ``.
- 🟢 Step ids stay unique across the whole blocker, inside `any`s too.
- 🟢 A reference that stays inside a nested `all`, even one under an `any`, is still accepted, as is the dogfood chain.
- 🟢 The runtime "found no tag" branch in `src/eval.rs` stays, as a defensive `failed` that a loaded manifest can no longer reach.


== Changes

- `src/manifest_tests.rs` (where `check_steps` is exercised): the red tests first. Both shapes above are refused, while the dogfood all-chain and a reference inside a nested `all` under an `any` are accepted.
- `src/condition.rs`: `check_in` scopes visibility. Ids registered inside one `any` step are visible within that step's own subtree only, and nothing inside an `any` is visible after it. Uniqueness is tracked in a separate set over the whole blocker.
- `docs/manifest.md`: one sentence on where a `step` reference may point.


== Out of scope

- Changing what `any` or `all` evaluate to.


== Dogfood

`~/.nix/canibump.toml` keeps loading; its chain is an all-of.

