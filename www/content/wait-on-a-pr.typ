#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/wait-on-a-pr/",
  label: "An issue or PR",
  title: "Wait for an issue or PR",
  summary: "Hold a workaround until an issue gets fixed or a PR merges, on any forge or tracker.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

The simplest wait there is. You carry a patch until upstream fixes the bug,
and `state` points at the issue or PR that does it. One blocker per forge
here, each landed a different way.

#file("wait-on-a-pr", "canibump.toml")

#terminal("wait-on-a-pr")

= Bumpable or dead

A merged PR, or an issue closed as completed, is `bumpable`. Closed unmerged,
won't fix and duplicate are `dead`, because the fix you wait for won't come
that way. Forges keep different records, so the rest depends on where the
item lives.

- *GitHub* records why an issue closed, so completed and not planned come
  apart cleanly.
- *GitLab* keeps no close reason. A closed issue counts only when a merged MR
  closed it, so point at the MR when there is one.
- *Gitea, Forgejo and Codeberg* keep no reason either, and a closed issue
  reads `dead` until you look. Point at the PR instead.
- *Redmine and Jira* say why in a status or resolution name. canibump matches
  whole words. Rejected, Won't Fix, Invalid or Abandoned read `dead`, Fixed or
  Done read `bumpable`, and Not done doesn't.

= Writing the ref

The #link(u("/manifest/#blocker"))[manifest reference] lists every form
`state` takes. Three trip people up.

- On GitLab, `!88` is a merge request and `#88` is an issue.
- A self-hosted forge puts its host first, as in
  `gitlab:gitlab.example.org/group/project#3`.
- A Jira that lives under a path keeps it, as in
  `jira:issues.apache.org/jira#KAFKA-42`.

Private repos and rate limits need a token.
#link(u("/tokens-and-hosts/"))[Tokens and hosts] says where canibump finds
one.
