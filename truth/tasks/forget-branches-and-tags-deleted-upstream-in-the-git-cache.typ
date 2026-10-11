#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Forget branches and tags deleted upstream in the git cache",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("git", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: helps[`in = { tags = "v*" }` names only tags the remote still has],
      honest-state: helps[a retracted tag or a deleted branch can no longer read bumpable from a stale local ref],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[tests against a local bare upstream pin the deletion across two runs],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Point a `reached` blocker at a `git:` remote's branch `side`, or at its tags `v*`, and run `canibump`. The bare clone under the cache dir keeps the fetched refs.
+ Delete the branch upstream, or retract the tag `v0.2.0`, then run `canibump` again.
+ 🔴 `Inner::fetch` in `src/provider/git.rs` treats gix's `NoMapping` (the remote has no such ref) as success, so the stale local ref answers. The test `a_branch_deleted_upstream_since_an_earlier_run_reads_not_found` gets `Ok(true)` where `NotFound` is due.
+ 🔴 The `+refs/tags/*` fetch never prunes, so `tags()` keeps the retracted tag: `a_tag_retracted_upstream_since_an_earlier_run_is_gone` gets `Ok(["v0.1.0", "v0.2.0"])` for `Ok(["v0.1.0"])`, and a step can read `bumpable "… is in v0.2.0"` off a tag that no longer exists.
+ 🔴 A remote with every tag deleted still lists both: `Ok(["v0.1.0", "v0.2.0"])` for `Ok([])`.


== Environment

- `canibump` built from `main` at f018858, on Linux.


== TO-BE

- 🟢 When the remote has no ref the spec asks for, the local copy is deleted and the check reads `NotFound`.
- 🟢 A fetch of every tag deletes local tags the remote no longer advertises, so `tags()` lists exactly the remote's tags.


== Changes

- `src/provider/git_tests.rs`: the red tests first. A branch deleted between two runs reads `NotFound`, a retracted tag leaves `tags()` and reads `NotFound`, a remote with no tags left lists none.
- `src/provider/git.rs`: after each fetch, delete the local refs the spec owns that the remote did not map, on success and on `NoMapping` alike.


== Out of scope

- Garbage-collecting the objects the dropped refs held. They cost disk only, and `contains` reads through refs.


== Dogfood

Not applicable. This repo's `canibump.toml` waits on GitHub items, not on a `git:` remote.

