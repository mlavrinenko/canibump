# canibump

[![CI](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml/badge.svg)](https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canibump.svg)](https://crates.io/crates/canibump)
[![License: MIT](https://img.shields.io/crates/l/canibump.svg)](LICENSE-MIT)

Check whether upstream fixes you wait on have landed, so you know when to bump

## Install

### From crates.io

```bash
cargo install canibump
```

### From binary releases

Download a pre-built binary from the
[latest release](https://github.com/mlavrinenko/canibump/releases/latest).

## Usage

```bash
canibump
```

## Development

Prerequisites: [Nix](https://nixos.org/) with flakes enabled.

```bash
direnv allow         # or: nix develop
just install-hooks   # pre-commit hook running the (memoized) gate

just check           # fmt, clippy, tests, unused deps, file size, drift, copy-paste
just deny            # cargo-deny: advisories, licenses, bans (needs network)
just build
just test
just cover           # code coverage via cargo-llvm-cov (70% minimum)
just fmt             # format code
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for coding conventions.

## License

MIT
