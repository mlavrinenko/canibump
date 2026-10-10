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
