#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the llama.cpp blocking-sync blocker into ~/.nix/canibump.toml",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("dogfood",),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      one-command: helps[`canibump` in `~/.nix` says in one line whether the postPatch can go],
      many-targets: neutral,
      whole-chain: helps[the entry is merged AND in a `v*` tag AND packaged in nix_unstable at least at that tag],
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
  links: parent(
    "track-the-llama-cpp-blocking-sync-patch-in-nix-with-canibump.typ",
  ),
)

== Problem

+ Read `~/.nix/apps/llama-cpp-16gb.nix`: its postPatch waits on ggml-org/llama.cpp#26487, "open as of 2026-08-17".
+ 🔴 Nothing re-checks it.


== TO-BE

- 🟢 `canibump` in `~/.nix` prints `waiting llama-blocking-sync merged: github:ggml-org/llama.cpp#26487 is open` today, and `bumpable` once the chain holds.


== Changes

- `~/.nix/canibump.toml`: one all-of entry, `merged`, `released` (tags `v*`, since nixpkgs follows llama.cpp's semver stream, not the `b` builds) and `packaged` (Repology `llama.cpp`, `nix_unstable`, srcname `llama-cpp`).
- `README.md`: say the chain example is that real entry, with its real output.


== Rollback

- Delete `~/.nix/canibump.toml`.


== Out of scope

- Committing in `~/.nix`. Its own session does.


== Dogfood

This task is the dogfood.

