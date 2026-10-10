# canibump

[![CI](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml/badge.svg)](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canibump.svg)](https://crates.io/crates/canibump)
[![License: MIT](https://img.shields.io/crates/l/canibump.svg)](LICENSE-MIT)

Can I bump yet? Your repo carries workarounds that wait on upstream. A patch
until a PR lands, a pin until a release ships. List them in `canibump.toml`,
run `canibump`, and it tells you which ones you can finally drop.

## Install

```bash
cargo install canibump
```

Or grab a binary from the [latest release](https://github.com/mlavrinenko/canibump/releases/latest).

## Usage

Drop a `canibump.toml` in your repo.

```toml
[[blocker]]
id = "llama-blocking-sync"
note = "apps/llama-cpp-16gb.nix: drop the GGML_CUDA_BLOCKING_SYNC postPatch"
state = "github:ggml-org/llama.cpp#26487"

[[blocker]]
id = "serde-pin"
note = "Cargo.toml: unpin serde"
reached = "github:serde-rs/serde#2590"
in = { tags = "v*" }
```

```console
$ canibump
waiting   llama-blocking-sync  github:ggml-org/llama.cpp#26487 is open
bumpable  serde-pin            github:serde-rs/serde#2590 (110af31) is in v1.0.185 (Cargo.toml: unpin serde)
$ echo $?
10
```

One line per blocker, and the status is one of these.

| Status | Means |
| --- | --- |
| `waiting` | not landed yet |
| `bumpable` | landed, go drop the workaround |
| `dead` | closed unmerged, wontfix, duplicate, or closed with no reason. Rethink the workaround |
| `failed` | the check broke (network, auth, rate limit). Never counts as landed |

Three conditions and two combinators cover every target in one manifest.

| Condition | Waits for | Targets |
| --- | --- | --- |
| `state` | an issue fixed, a PR or MR merged | GitHub, GitLab, Gitea/Forgejo/Codeberg, Redmine, Jira |
| `reached` | a commit or merged PR in a branch, a tag, or the first tag matching `v*` | GitHub, any git remote |
| `version` | a package at least some version | Repology (distros, nixpkgs), deps.dev (crates.io, npm, PyPI, Go, Maven, NuGet, RubyGems) |
| `all` / `any` | a chain of the above, e.g. merged AND released AND packaged | |

The full grammar, chains, the cache format and auth live in
[docs/manifest.md](docs/manifest.md).

### Exit codes

| Code | Means |
| --- | --- |
| 0 | nothing bumpable, every check ran |
| 10 | something is bumpable |
| 11 | a check failed, nothing bumpable |
| 12 | something is bumpable and a check failed |
| 1 | bad manifest, cache or arguments |

A bumpable blocker keeps exiting 10 until you delete it. That's the reminder.

### Commands

```bash
canibump                 # check, reusing results younger than the TTL
canibump --refresh       # check everything now
canibump --json          # same, as JSON
canibump status          # last results from the cache, no network
canibump status --path   # where the cache lives
canibump schema          # print the JSON Schema for canibump.toml
canibump -m path/to/canibump.toml
```

### Editor support

The manifest has a JSON Schema, generated from the same Rust types that parse it, so it
can't go stale. taplo and tombi give you completion, hover docs and squiggles on typos
once the first line of `canibump.toml` points at it.

```toml
#:schema https://raw.githubusercontent.com/mlavrinenko/canibump/main/schema/canibump.schema.json
```

Prefer a local copy that works offline? `just install` writes one next to the binary
and `canibump schema > FILE` makes one anywhere.

```toml
#:schema file:///home/<you>/.config/canibump/canibump.schema.json
```

## Development

Needs [Nix](https://nixos.org/) with flakes.

```bash
direnv allow   # or nix develop
just check     # fmt, clippy, tests, unused deps, file size, drift, copy-paste, tasks
just cover     # coverage, 70% minimum
```

Conventions live in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
