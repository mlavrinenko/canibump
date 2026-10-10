# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `canibump.toml` manifest: `[[blocker]]` entries with an `id`, a `note` and a
  `state = "github:owner/repo#N"` condition, plus `[settings]` `ttl` and
  `cache`. Errors name the blocker they sit in.
- Each check ends `waiting`, `bumpable`, `dead` or `failed`. A merged PR or an
  issue closed as completed is bumpable; closed unmerged, not planned,
  duplicate or closed without a reason is dead; a provider error is failed.
- GitHub issues and PRs through octocrab. The token comes from `GITHUB_TOKEN`,
  `GH_TOKEN` or `gh auth token`, else requests go unauthenticated. 401, 403,
  404 and rate limits fail the check, saying which.
- `canibump` checks every blocker and prints one line each; `--json` prints a
  document. Exit codes: 0 nothing bumpable, 10 bumpable, 11 a check failed,
  12 both, 1 unusable manifest.
- Results land in `$XDG_CACHE_HOME/canibump/state/<manifest path>.json` with
  `checked_at` and `fresh_until`; fresh ones are reused unless `--refresh`.
  `canibump status` reads only that file, and `status --path` prints it.
- `reached = "github:o/r#N"` or `"github:o/r@sha"` with `in = { branch = ... }`,
  `{ tag = ... }` or `{ tags = "v*", since = "0.5.0" }`: bumpable once the
  commit, or the PR's merge commit, is there. A pattern names the first tag
  that has it, found by binary search over tags in version order.
- Versions compare as semver when both parse, else leniently: a letter
  prefix drops (`b7012` = `7012`) and a letter suffix is a pre-release.
- GitHub answers `reached` through the compare API and lists tags through
  `git/matching-refs`, so no clone is needed.
- `reached = "git:<url>@sha"` works on any git remote: gix fetches only the
  refs asked about into a bare repo under `~/.cache/canibump/git/` and walks
  history in-process.
- `version = "repology:<project>"` (with `repo`, optional `srcname`) or
  `version = "deps.dev:<system>/<name>"` with `at-least = "<version>"`:
  bumpable once the highest version there reaches the bound.
- Repology and deps.dev clients. Repology is asked once per project per
  run, at most once a second, with a User-Agent naming canibump.
  `CANIBUMP_GITHUB_API`, `CANIBUMP_REPOLOGY_API` and `CANIBUMP_DEPSDEV_API`
  point a provider elsewhere.
- `[[blocker.all]]` and `[[blocker.any]]` chain steps, nested as deep as you
  like. An all-of stops at its first step that does not hold; an any-of
  resolves on its first that does. `at-least = { step = "<id>" }` uses the
  tag an earlier `reached` step found, checked when the manifest loads.
- GitLab issues (`gitlab:group/proj#N`) and MRs (`!N`), self-hosted with
  `gitlab:host/group/proj`. A closed issue is completed only when an MR
  closed it. Tokens come from glab's config and `GITLAB_TOKEN` for
  `GITLAB_HOST`, each sent only to its own host.
- Gitea and Forgejo items: `gitea:host/owner/repo#N`, `forgejo:...` and
  `codeberg:owner/repo#N`. A PR reads merged or closed unmerged; a closed
  issue reads closed without a reason, since Gitea keeps none. Tokens come
  from tea's config logins and `GITEA_TOKEN` for `GITEA_HOST`.
- Redmine (`redmine:host#N`) and Jira (`jira:host#KEY-N`) issues, with a
  path when the instance lives under one. Status and resolution names match
  on whole words, so `Abandoned` reads dead rather than done. Credentials:
  `REDMINE_API_KEY` for `REDMINE_URL`; `JIRA_TOKEN`, or `JIRA_USER` with
  `JIRA_API_TOKEN`, for `JIRA_URL`.
