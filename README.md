<!-- Rendered from docs/src/readme.typ by `just docs-md`. Edit the source. -->

<h1 align="center"><img src="https://raw.githubusercontent.com/mlavrinenko/canibump/main/www/assets/images/logo.svg" alt="canibump" width="96" /><br />
canibump</h1>

<a href="https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/mlavrinenko/canibump/actions/workflows/ci.yml/badge.svg" /></a> <a href="https://crates.io/crates/canibump"><img alt="crates.io" src="https://img.shields.io/crates/v/canibump.svg" /></a> <a href="LICENSE-MIT"><img alt="License: MIT" src="https://img.shields.io/crates/l/canibump.svg" /></a>

Tells you which of your patches, pins and workarounds can go, because upstream finally landed what they wait on.

`canibump.toml`

```toml
#:schema https://raw.githubusercontent.com/mlavrinenko/canibump/main/schema/canibump.schema.json

[[blocker]]
id = "libfoo-crash"
note = "drop the libfoo crash patch"
state = "github:libfoo/libfoo#412"

[scan]
paths = ["**/*.nix"]
```

```
$ canibump
waiting   libfoo-crash  github:libfoo/libfoo#412 is open
$ canibump --refresh
bumpable  libfoo-crash  github:libfoo/libfoo#412 merged (drop the libfoo crash patch)
          at pkgs/libfoo.nix:4
```

You write each patch, pin or override down once, with the issue, release or package it waits on. canibump asks upstream and says which ones can go, and where they sit in your files. [The quickstart](https://mlavrinenko.github.io/canibump/quickstart/) walks you through it.

## Install

- `cargo install canibump`
- `nix profile install github:mlavrinenko/canibump`
- A prebuilt binary from the [releases page](https://github.com/mlavrinenko/canibump/releases/latest)

## Docs

Everything lives at [mlavrinenko.github.io/canibump](https://mlavrinenko.github.io/canibump/).

- **Wait for.** [An issue or PR](https://mlavrinenko.github.io/canibump/wait-on-a-pr/), [A fix to ship](https://mlavrinenko.github.io/canibump/wait-on-a-release/), [A package](https://mlavrinenko.github.io/canibump/wait-on-a-package/), [A chain](https://mlavrinenko.github.io/canibump/chain-steps/)
- **Track workarounds.** [Mark them](https://mlavrinenko.github.io/canibump/mark-workarounds/), [Catch forgotten ones](https://mlavrinenko.github.io/canibump/catch-forgotten-workarounds/)
- **Automate.** [Run it in CI](https://mlavrinenko.github.io/canibump/run-in-ci/), [Read results offline](https://mlavrinenko.github.io/canibump/read-results-offline/), [Tokens and hosts](https://mlavrinenko.github.io/canibump/tokens-and-hosts/)
- **Reference.** [CLI](https://mlavrinenko.github.io/canibump/cli/), [Manifest](https://mlavrinenko.github.io/canibump/manifest/), [JSON report](https://mlavrinenko.github.io/canibump/report/), [State file](https://mlavrinenko.github.io/canibump/state-file/), [How a run works](https://mlavrinenko.github.io/canibump/how-it-works/)

## License

[MIT](LICENSE-MIT).
