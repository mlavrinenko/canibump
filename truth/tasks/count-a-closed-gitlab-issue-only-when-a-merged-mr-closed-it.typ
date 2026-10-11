#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count a closed GitLab issue only when a merged MR closed it",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("forges", "trackers", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a closed GitLab issue with only an open or closed-unmerged MR no longer reads bumpable],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[the fixture carries the MR state, so it can no longer hide the gap],
      docs-match-code: helps[`docs/manifest.md` already promises that an MR closed it; the code now keeps that promise],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Close a GitLab issue by hand as wontfix while an open MR says `Closes #N`, then point a `state = "gitlab:group/proj#N"` entry at it.
+ 🔴 `GET /issues/N/closed_by` returns that MR in its `opened` state, `item_state` reads `Completed` and the entry prints `bumpable`. The test `a_closed_issue_counts_only_when_a_closing_mr_merged` fails with `open MR` / `left: Ok(Completed)` / `right: Ok(ClosedNoReason)`.
+ GitLab documents `closed_by` as the MRs that close the issue when merged, in any state. `src/provider/gitlab.rs` deserializes each closer as `IgnoredAny`, so only the count matters, and the old fixture `[{"iid": 7}]` never showed it.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Each closer's `state` is read. A closed issue is `Completed` only when one closer is `merged`, else `ClosedNoReason`.
- 🟢 An open or closed-unmerged closing MR no longer makes the issue bumpable. A merged one among others still does.


== Changes

- `src/provider/gitlab_tests.rs`: the red test first, open, closed and mixed closers; the old fixture gains a `state`.
- `src/provider/gitlab.rs`: deserialize the closer's `state` and test for `merged`.
- `CHANGELOG.md`: the `state` bullet under `[Unreleased]` says what counts.


== Out of scope

- The token lookup in the same file, and GitLab MR states. Both already read right.


== Dogfood

Not applicable. `~/.nix/canibump.toml` has no GitLab issue entry.

