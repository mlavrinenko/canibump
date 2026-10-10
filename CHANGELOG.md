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
