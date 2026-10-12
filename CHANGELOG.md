# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `[scan]` reads the files you list for issue, PR and MR links and prints an
  `unlisted` line for each one no blocker names, so a workaround nobody filed
  still shows up. GitHub, GitLab and Codeberg links are caught out of the
  box. `[[scan.catcher]]` adds a regex of your own, replaces a built-in or
  turns one off, and `canibump:ignore` skips a line.
- An unlisted link adds 16 to the exit code and counts by default, and
  `fail-on` takes `unlisted`. `--json` lists each under `unlisted`.

## [0.1.0] - 2026-10-11


### Added

- `canibump` reads `canibump.toml`, checks every blocker and prints one line
  each, `waiting`, `bumpable`, `dead` or `failed`. A closed-unmerged PR or a
  wontfix issue reads dead, and a broken check never reads as landed.
- `state` waits on an issue, PR or MR on GitHub, GitLab, Gitea, Forgejo,
  Codeberg, Redmine and Jira, self-hosted ones included. A closed GitLab
  issue counts only when a merged MR closed it, and Redmine trusts the
  server's `is_closed` over the status name. A name like `Not done` never
  reads bumpable.
- `reached` waits for a commit or a merged PR to land in a branch, a tag, or
  the first tag matching a pattern, on GitHub or any git remote. The manifest
  refuses a target anywhere else, and an issue reads failed even while open,
  since it has no merge commit. Tags sort in one version order that covers
  semver, `v`-tags, dates and builds like `b7012`, where a missing `.0` counts
  as zero (`2.0` is `2.0.0`). A fix shipped on an old release line only still
  finds its tag, and a tag cut from another branch can't hide the first one.
  A branch or tag deleted upstream stops counting on the next run, and a git
  remote that stalls reads failed after five minutes.
- `version` waits for a package version on Repology (distros, nixpkgs
  channels) or deps.dev (crates.io, npm, PyPI, Go, Maven, NuGet, RubyGems).
- `[[blocker.all]]` and `[[blocker.any]]` chain steps, like merged, then
  released, then in nixpkgs, and a step can use the tag an earlier one found.
  `at-least = { step = ... }` refuses any other key, like every manifest table.
- The exit code is 8 plus 1 for bumpable, 2 for dead and 4 for failed, summed,
  so a script tests one bit. `--fail-on` or `[settings] fail-on` picks what
  counts.
- Results land in a state file other tools read offline (under
  `%LOCALAPPDATA%` on Windows), and stay fresh for 6 hours by default
  (`ttl = "1d"` and `"2w"` work too). `canibump status` reads it without
  network and skips results of a changed or removed blocker, and
  `canibump status <id>` answers for one blocker by exit code.
- `--json` prints the results as JSON. `canibump schema report` and
  `canibump schema state` print JSON Schemas of it and of the state file,
  generated from the code.
- Tokens come from what you already use: `gh`, glab and tea configs, or
  environment variables. Each goes only to its own host, never follows a
  redirect off it or a `CANIBUMP_*_API` override, and goes only when a
  blocker needs it. A password in a `git:` URL shows as `***` in every
  message and in the state file.
- Each question is asked once per run, up to eight blockers at once, and
  Repology gets at most one request a second.
- `canibump schema` prints the manifest's JSON Schema, so taplo and tombi give
  completion and hover docs. `canibump completions <shell>` prints tab
  completion, and the Nix package installs both.
- The crate is a binary first. Its library is not a stable API, and its
  modules stay out of the docs.
