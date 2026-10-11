#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give each git remote its own clone dir, with no credentials in it",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("git", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[one remote's refs can no longer answer for another's],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[unit tests pin an injective dir name and redacted userinfo],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ `clone_dir` in `src/provider/git.rs` turns every character but `[A-Za-z0-9.-]` into `_`.
+ 🔴 `/srv/a_b.git` and `/srv/a/b.git` both map to `/c/_srv_a_b.git.git`, while each URL has its own lock and its own "fetched" entry, so two remotes fetch into one repo and one's refs can answer for the other.
+ Write a remote with credentials, `git:https://user:s3cret@example.org/a/b.git`.
+ 🔴 The token lands in the dir name, `https___user_s3cret_example.org_a_b.git.git`, and `Source`'s `Display` prints `git:https://user:s3cret@example.org/a/b.git` into every error message and outcome detail, which the state file then stores.


== Environment

- `canibump` built from `main` at f018858, on Linux.


== TO-BE

- 🟢 The clone dir is a readable slug of the URL without its userinfo plus a hash of the full URL, so two URLs never share one.
- 🟢 `Source::Git` displays its URL with the userinfo replaced by `***`, so no message or state file carries a token.


== Changes

- `src/provider/git_tests.rs`: the red tests first. Two URLs differing only in `_` versus `/` get two dirs, and a URL with credentials yields a dir name without them.
- `src/refs_tests.rs`: the red test that a `git:` source with userinfo displays redacted.
- `src/provider/git.rs`: the slug plus a hash of the full URL.
- `src/refs.rs`: redact userinfo in `Display for Source`.


== Out of scope

- Migrating clone dirs of earlier runs. The old ones turn into dead weight under the cache dir and a new run re-fetches.
- Credentials gix itself may echo inside its own error text. Fetch errors are prefixed with the redacted source, and gix redacts URLs it prints.


== Dogfood

Not applicable. This repo's `canibump.toml` has no `git:` remote.

