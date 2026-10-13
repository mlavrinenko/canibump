# canibump

[![CI](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml/badge.svg)](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canibump.svg)](https://crates.io/crates/canibump)
[![License: MIT](https://img.shields.io/crates/l/canibump.svg)](LICENSE-MIT)

Run `canibump` next to a `canibump.toml` to see which workarounds you can
drop. This tool was created to track patches and pins that wait on upstream
(a PR to merge, a release to ship, a package to reach nixpkgs), so they don't
outlive their reason.

Ready to install? Run `cargo install canibump` or
`nix profile install github:mlavrinenko/canibump`. No cargo or flakes? Grab a
binary from the [releases page](https://github.com/mlavrinenko/canibump/releases/latest).
`canibump completions <shell>` prints tab completion; the Nix package installs
it for bash, zsh and fish.

## Manifest example

```toml
[[blocker]]
id = "llama-blocking-sync"
note = "drop the GGML_CUDA_BLOCKING_SYNC postPatch"
state = "github:ggml-org/llama.cpp#26487"

[[blocker]]
id = "serde-pin"
note = "unpin serde"
reached = "github:serde-rs/serde#2590"
in = { tags = "v*" }
```

## Output example

```console
$ canibump
waiting   llama-blocking-sync  github:ggml-org/llama.cpp#26487 is open
bumpable  serde-pin            github:serde-rs/serde#2590 (110af31) is in v1.0.185 (unpin serde)
```

## Possible statuses

| Status | Means |
| --- | --- |
| `waiting` | not landed yet |
| `bumpable` | landed, go drop the workaround |
| `dead` | closed unmerged, wontfix, duplicate or with no reason, so rethink the workaround |
| `failed` | the check broke (network, auth, rate limit), never counts as landed |
| `unlisted` | a link or marker in your files no blocker names, see [forgotten workarounds](#forgotten-workarounds) |
| `unmarked` | a blocker nothing in your files places, no `canibump:<id>` and no matching declared site, so its workaround may be gone |

## Possible conditions

| Condition | Waits for | Targets |
| --- | --- | --- |
| `state` | an issue fixed, a PR or MR merged | GitHub, GitLab, Gitea, Forgejo, Codeberg, Redmine, Jira |
| `reached` | a commit or a merged PR in a branch, a tag, or the first tag matching `v*` | GitHub, any git remote |
| `version` | a package at some version or newer | Repology (distros, nixpkgs), deps.dev (crates.io, npm, PyPI, Go, Maven, NuGet, RubyGems) |
| `all` / `any` | a chain of the above, like merged and released and packaged | |

The full grammar, the cache and auth live in [the docs](https://mlavrinenko.github.io/canibump/).

## Forgotten workarounds

A pin nobody wrote a blocker for never shows up, so it lives forever. Give
canibump a `[scan]` and it reads your files for issue, PR and MR links, then
flags every one no blocker names.

```toml
[scan]
paths = ["**/*.nix", "Justfile"]
```

```console
$ canibump
unlisted  hosts/gpu.nix:41  github:NixOS/nixpkgs#412345 has no blocker
```

GitHub, GitLab and Codeberg links are caught out of the box. Bring your own
regex for anything else, or switch a built-in off.

Mark the workaround itself with `# canibump:<id>` and the note stops needing
a line number that rots. A bumpable line points you at the spot.

```console
$ canibump
bumpable  serde-pin  github:serde-rs/serde#2590 (110af31) is in v1.0.185 (unpin serde)
          at Cargo.toml:12
```

See [Catch forgotten workarounds](https://mlavrinenko.github.io/canibump/catch-forgotten-workarounds/).

## Commands

```bash
canibump                 # check, reusing results younger than 6h
canibump --refresh       # check everything now
canibump status          # last results, no network
canibump status <id>     # one blocker, by exit code
canibump schema          # the JSON Schema of canibump.toml
canibump -m path/to/canibump.toml
```

## Exit codes

| Code | Means |
| --- | --- |
| 0 | nothing counted showed up |
| 8 + 1, 2, 4, 16, 32 | bumpable, dead, failed, unlisted, unmarked, summed, so test a bit with `(( code & 2 ))` |
| 1 | bad manifest or cache |
| 2 | bad arguments |

All five count by default. `--fail-on failed` or `[settings] fail-on` picks
which do. `canibump status <id>` has its own codes, 0 bumpable, 1 waiting,
2 anything else.

## Interop usage

Pass `--json` to get machine-readable data. See
[its schema](schema/report.schema.json). Every run also saves the results to a
state file other tools can read offline, see
[its schema](schema/state.schema.json). `canibump status --path` says where it
is, and `canibump schema report` or `canibump schema state` prints the schema
that matches your binary.

## Editor support

Point the first line of `canibump.toml` at the schema, and taplo or tombi
give you completion, hover docs and squiggles on typos.

```toml
#:schema https://raw.githubusercontent.com/mlavrinenko/canibump/main/schema/canibump.schema.json
```

## Architecture

See [How a run works](https://mlavrinenko.github.io/canibump/how-it-works/).
