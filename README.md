# canibump

[![CI](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml/badge.svg)](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canibump.svg)](https://crates.io/crates/canibump)
[![License: MIT](https://img.shields.io/crates/l/canibump.svg)](LICENSE-MIT)

Can I bump yet? Your repo carries workarounds that wait on upstream: a patch
until a PR lands, a pin until a release ships. List them in `canibump.toml`,
run `canibump`, and it tells you which ones you can finally drop.

## Install

```bash
cargo install canibump
```

Or grab a binary from the [latest release](https://github.com/mlavrinenko/canibump/releases/latest).

## Usage

Drop a `canibump.toml` in your repo:

```toml
[settings]
ttl = "6h"            # reuse results this long (default 6h)

[[blocker]]
id = "llama-blocking-sync"
note = "apps/llama-cpp-16gb.nix: drop the GGML_CUDA_BLOCKING_SYNC postPatch"
state = "github:ggml-org/llama.cpp#26487"

[[blocker]]
id = "serde-pin"
note = "Cargo.toml: unpin serde"
state = "github:serde-rs/serde#2590"
```

Then:

```console
$ canibump
waiting   llama-blocking-sync  github:ggml-org/llama.cpp#26487 is open
bumpable  serde-pin            github:serde-rs/serde#2590 merged (Cargo.toml: unpin serde)
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

### Conditions

Each blocker takes exactly one condition.

`state`: an issue or PR. Merged or closed as completed is bumpable.

```toml
state = "github:serde-rs/serde#2590"
```

`reached`: a commit, or a PR's merge commit, landed in a branch, a tag, or
any tag matching a pattern. With a pattern, canibump names the first tag
that has it, in version order (semver, or lenient for stuff like `b7012`).

```toml
reached = "github:serde-rs/serde#2590"     # or "github:owner/repo@<sha>"
# reached = "git:https://github.com/rust-lang/glob.git@952da29"  # any git remote
in = { branch = "master" }
# in = { tag = "v1.0.185" }
# in = { tags = "v*", since = "1.0.180" }
```

GitHub answers through its API, no clone. A `git:` remote gets a bare clone of
just the refs you ask about under `~/.cache/canibump/git/`, via gix, reused
across runs.

```console
$ canibump
bumpable  serde-released  github:serde-rs/serde#2590 (110af31) is in v1.0.185 (Cargo.toml: unpin serde)
```

### Exit codes

| Code | Means |
| --- | --- |
| 0 | nothing bumpable, every check ran |
| 10 | something is bumpable |
| 11 | a check failed, nothing bumpable |
| 12 | something is bumpable and a check failed |
| 1 | bad manifest, cache or arguments |

A bumpable blocker keeps exiting 10 until you delete its entry. That's the
reminder.

### Commands

```bash
canibump                 # check, reusing results younger than the TTL
canibump --refresh       # check everything now
canibump --json          # same, as JSON
canibump status          # last results from the cache, no network
canibump status --path   # where the cache lives
canibump -m path/to/canibump.toml
```

### Cache

Every run writes the results to
`$XDG_CACHE_HOME/canibump/state/<absolute manifest path>.json`
(`~/.cache/...` without XDG), so other tools read the state offline:

```json
{
  "version": 1,
  "manifest": "/home/me/repo/canibump.toml",
  "entries": [
    {
      "id": "serde-pin",
      "note": "Cargo.toml: unpin serde",
      "status": "bumpable",
      "detail": "github:serde-rs/serde#2590 merged",
      "checked_at": "2026-10-10T12:00:00Z",
      "fresh_until": "2026-10-10T18:00:00Z",
      "spec": "state = \"github:serde-rs/serde#2590\"\n"
    }
  ]
}
```

Move it with `--cache FILE` or `[settings] cache = "FILE"` (relative to the
manifest). A failed check is never fresh, so the next run retries it.

### Auth

GitHub takes `GITHUB_TOKEN`, then `GH_TOKEN`, then whatever `gh auth token`
says. No token works too, just with GitHub's tighter rate limit.

## Development

Prerequisites: [Nix](https://nixos.org/) with flakes enabled.

```bash
direnv allow         # or: nix develop
just install-hooks   # pre-commit hook running the (memoized) gate

just check           # fmt, clippy, tests, unused deps, file size, drift, copy-paste, tasks
just deny            # cargo-deny: advisories, licenses, bans (needs network)
just build
just test
just cover           # code coverage via cargo-llvm-cov (70% minimum)
just fmt             # format code
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for coding conventions.

## License

MIT
