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
nix profile install github:mlavrinenko/canibump
```

Or grab a binary from the [latest release](https://github.com/mlavrinenko/canibump/releases/latest).

Tab completion is one line in your shell config. The Nix package installs it for bash, zsh
and fish on its own.

```bash
source <(canibump completions bash)   # ~/.bashrc
source <(canibump completions zsh)    # ~/.zshrc, after compinit
canibump completions fish | source    # ~/.config/fish/config.fish
```

`elvish` and `powershell` work too.

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
9
```

One line per blocker, in manifest order. Blockers are checked eight at a time,
so a long manifest costs about its slowest chain, not the sum. The status is
one of these.

| Status | Means |
| --- | --- |
| `waiting` | not landed yet |
| `bumpable` | landed, go drop the workaround |
| `dead` | closed unmerged, wontfix, duplicate, or closed with no reason. Rethink the workaround |
| `failed` | the check broke (network, auth, rate limit). Never counts as landed |

Bumpable and dead lines end with the blocker's note, so you see which workaround to drop or
rethink. A result past its TTL says so, as in `(stale, checked 7h ago)`.

On a terminal the status word gets color. Bumpable is green, dead magenta, failed red, waiting
dimmed. `NO_COLOR` switches it off, `--color always` or `--color never` overrides the guess,
and pipes and `--json` stay plain.

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
| 0 | nothing counted showed up, every check ran |
| 9 to 15 | 8 plus 1 for bumpable, 2 for dead, 4 for failed, summed |
| 1 | bad manifest or cache |
| 2 | bad arguments |

So bumpable alone exits 9, dead alone 10, bumpable and dead 11, a failed check 12 and all
three 15. A script tests one bit, as in `(( code & 2 ))` for dead.

All three count by default. `--fail-on` (comma-separated or repeated) or `[settings] fail-on`
picks which ones do, and the flag wins. The lines print either way.

```bash
canibump --fail-on failed          # only a broken check fails the run
canibump --fail-on bumpable,dead   # a flaky network won't page you
```

`fail-on = []` in the manifest never fails the run. `waiting` never counts, so you can't name it.

A counted blocker keeps failing the run until you delete it. That's the reminder.

`canibump status <id>` answers for one blocker, offline, so a script or a task tracker asks
with one call and reads the exit code.

| Code | Means |
| --- | --- |
| 0 | bumpable |
| 1 | waiting |
| 2 | anything else |

Anything else is dead, failed, an id the manifest or the cache lacks, a result checked
against an older condition (run `canibump` again) and every error. Errors exit 2 here, never
1, which would read as waiting. It prints that blocker's usual line, and `--json` a report
holding only it.

### Commands

```bash
canibump                 # check, reusing results younger than the TTL
canibump --refresh       # check everything now
canibump --json          # same, as JSON
canibump status          # last results from the cache, no network
canibump status <id>     # one blocker, by exit code
canibump status --path   # where the cache lives
canibump schema          # print the JSON Schema for canibump.toml
canibump completions zsh # print a completion script for a shell
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
and `canibump schema > FILE` makes one anywhere. The Nix package ships one too, as
`share/canibump/canibump.schema.json` in your profile.

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
