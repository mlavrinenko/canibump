#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the llama.cpp blocking-sync patch in ~/.nix with canibump",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  kind: "milestone",
  tags: ("dogfood",),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      one-command: helps[one run in `~/.nix` says whether the postPatch can go],
      many-targets: neutral,
      whole-chain: helps[merged, released in a `v*` tag, and packaged in nix_unstable at least at that tag],
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
  links: depends-on(
    "check-that-a-commit-or-merged-pr-reached-a-branch-tag-or-release.typ",
    "check-package-versions-through-deps-dev-and-repology.typ",
    "combine-conditions-with-all-of-and-any-of.typ",
  ),
)

== Problem

+ `~/.nix/apps/llama-cpp-16gb.nix` patches `GGML_CUDA_BLOCKING_SYNC` into ggml-cuda.cu while ggml-org/llama.cpp#26487 is open.
+ 🔴 The comment says "open as of 2026-08-17"; nothing re-checks it.


== TO-BE

- 🟢 `~/.nix/canibump.toml` carries one entry for the patch, and `canibump` there prints its state in one line.


== Changes

- `~/.nix/canibump.toml`: the entry, an all-of of the PR merging, a `v*` tag containing it, and Repology `nix_unstable` at least at that tag.


== Out of scope

- Committing in `~/.nix`. Its own session commits the manifest.


== Dogfood

This task is the dogfood: the real run's output goes in the report.

