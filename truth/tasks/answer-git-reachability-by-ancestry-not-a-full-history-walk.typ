#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Answer git reachability by ancestry, not a full history walk",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.75,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("git", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[`git:` remotes the size of llvm or linux stay cheap to check],
      whole-chain: neutral,
      honest-state: helps[a short sha can no longer match a different commit and read bumpable],
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

+ `reached = "git:<url>@<sha>"` with `in = { branch = "main" }` on a large repo, before the commit lands.
+ 🔴 `Inner::contains` in `src/provider/git.rs` walks every commit from the tip, formats each id as a hex `String` and compares prefixes. The usual waiting case walks the entire history, on every check and on every probe of the tag binary search.
+ 🔴 It matches by prefix, so a 7-digit sha matches the first walked commit sharing those digits. In a large repo another commit can answer `bumpable`.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 The sha resolves once to a full object id. An ambiguous prefix fails, asking for more digits; one unknown after the fetch is not reached, since the fetch brought the ref's whole history.
- 🟢 Containment is an ancestry check (merge base of commit and tip equals the commit, or an equivalent gix graph query), with no string per commit.
- 🟢 Tests: an ambiguous short sha fails, an absent commit reads waiting, a present one is found, against local fixture repos.


== Changes

- `src/provider/git_tests.rs`: the red tests first, including an ambiguous prefix built from a fixture repo with two commits sharing digits, or a stubbed resolver if one cannot be made cheaply.
- `src/provider/git.rs`: resolve, then ancestry.
- `Cargo.toml`: a gix feature, only if the ancestry query needs one.


== Out of scope

- Shallow or partial clones: an ancestry answer needs the history between commit and tip.
- Pruning the clone cache.


== Dogfood

Not applicable: `~/.nix/canibump.toml` has no `git:` blocker yet; the repo's own manifest may add one.

