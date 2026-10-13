#import "../utils/page.typ": page

#let meta = (
  route: "/tokens-and-hosts/",
  label: "Tokens and hosts",
  title: "Tokens and hosts",
  summary: "Where canibump finds a token for each forge, and how to point it at a mirror or an enterprise host.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

canibump borrows the tokens your tools already have, so most setups need
nothing new. Public GitHub works with no token at all, just under a tighter
rate limit.

#table(
  columns: 2,
  table.header[Target][Token, first found wins],
  [GitHub], [`GITHUB_TOKEN`, `GH_TOKEN`, then `gh auth token`],
  [GitLab], [glab's config for the host, then `GITLAB_TOKEN` for `GITLAB_HOST`, which defaults to gitlab.com],
  [Gitea, Forgejo, Codeberg], [tea's logins, then `GITEA_TOKEN` for `GITEA_HOST`],
  [Redmine], [`REDMINE_API_KEY` for `REDMINE_URL`],
  [Jira], [`JIRA_TOKEN`, or `JIRA_USER` with `JIRA_API_TOKEN`, for `JIRA_URL`],
)

A token only ever goes to the host it belongs to, and only when a blocker
needs that host. A redirect to another host fails the check rather than carry
the token along.

= Mirrors and enterprise hosts

`CANIBUMP_<PROVIDER>_API` sends every request of one provider to another API
root, say a GitHub Enterprise box or a mirror. The provider is `GITHUB`,
`GITLAB`, `GITEA`, `REDMINE`, `JIRA`, `REPOLOGY` or `DEPSDEV`.

The override gets the token filed under its own host, never the item's host's
or github.com's. On GitHub Enterprise that's `GH_ENTERPRISE_TOKEN`,
`GITHUB_ENTERPRISE_TOKEN`, then `gh auth token --hostname <host>`, the way gh
picks one. So
`CANIBUMP_GITHUB_API=https://ghe.example.com/api/v3 GH_ENTERPRISE_TOKEN=ghp_xxx canibump`
keeps your github.com token at home.
