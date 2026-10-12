# Scan

A workaround with no blocker is one canibump never checks. `[scan]` reads
files for upstream links and prints an `unlisted` line for each one no
blocker names, at any depth of a chain. Add the blocker and the line goes.

```toml
[scan]
paths = ["**/*.nix", "Justfile", ".github/workflows/*.yml"]  # relative to the manifest
exclude = ["tasks/**"]
```

```console
$ canibump
waiting   llama-blocking-sync  github:ggml-org/llama.cpp#26487 is open
unlisted  hosts/gpu.nix:41     github:NixOS/nixpkgs#412345 has no blocker
```

`*` stays inside a directory and `**` crosses them, but never into `.git` or
a symlinked directory, so a Nix `result` link can't drag in the store. An
`exclude` ending in `/**` skips its directory unread. The manifest itself is
never read, and neither is a line holding `canibump:ignore`, for a link
that's just background reading. A file that isn't UTF-8 is skipped. Only a
check scans, so `canibump status` prints no `unlisted` line.

An unlisted link adds 16 to the exit code, so one alone exits 24.
`fail-on = ["bumpable", "dead", "failed"]` leaves it out. `--json` lists each
under `unlisted` with its `file`, `line`, `ref` and `catcher`.

A catcher is a regex run over each line, and a `ref` template its named
groups fill. The ref is compared with the blockers' in the form canibump
prints it, ignoring case. These are on by default.

| Catcher | Catches | Ref |
| --- | --- | --- |
| `github` | `github.com/o/r/issues/1`, `.../pull/1` | `github:o/r#1` |
| `gitlab-issue` | `gitlab.com/g/p/-/issues/1` | `gitlab:g/p#1` |
| `gitlab-mr` | `gitlab.com/g/p/-/merge_requests/1` | `gitlab:g/p!1` |
| `codeberg` | `codeberg.org/o/r/issues/1`, `.../pulls/1` | `codeberg:o/r#1` |

`[[scan.catcher]]` adds one of your own. Reusing a built-in's id replaces
it, and `enabled = false` turns a catcher off.

```toml
[[scan.catcher]]
id = "jira"
pattern = 'https://jira\.example\.com/browse/(?<key>[A-Z]+-\d+)'
ref = "jira:jira.example.com#${key}"

[[scan.catcher]]
id = "gitlab-mr"
enabled = false
```
