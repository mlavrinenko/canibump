# Scan

A workaround with no blocker is one canibump never checks, and a note that
says where the workaround sits goes stale the moment someone edits above it.
`[scan]` reads your files for both. It flags each upstream link no blocker
names, and it finds the markers that tell each blocker where it lives.

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

## Markers

Drop `canibump:<id>` in a comment next to the workaround and canibump knows
where it sits. The why and the exit plan stay in the manifest, once.

```nix
# canibump:llama-blocking-sync
postPatch = "...";
```

A bumpable or dead line then tells you which files to open.

```console
$ canibump
bumpable  llama-blocking-sync  github:ggml-org/llama.cpp#26487 is merged (drop the postPatch)
          at pkgs/llama-cpp.nix:12
```

One blocker can sit in many places, so mark each one. A marker naming no
blocker prints as `unlisted  file:3  canibump:old-id has no blocker`. That's
why an id takes only ASCII letters, digits, `_` and `-`. `canibump: foo` with
a space, `canibump::path` and `xcanibump:foo` mark nothing. `--json` gives
each blocker's `sites` as `{file, line}`.

## What gets read

`*` stays inside a directory and `**` crosses them, but never into `.git` or
a symlinked directory, so a Nix `result` link can't drag in the store. An
`exclude` ending in `/**` skips its directory unread. The manifest itself is
never read, and neither is a line holding `canibump-ignore`, for a link
that's just background reading. A file that isn't UTF-8 is skipped. The scan
is local and cheap, so `canibump status` runs it too.

An unlisted link or marker adds 16 to the exit code, so one alone exits 24.
`fail-on = ["bumpable", "dead", "failed", "unmarked"]` leaves it out.
`--json` lists each under `unlisted` with its `file`, `line`, `ref` and
`catcher`, which is `marker` for a marker.

## Unmarked blockers

A blocker no marker names is one whose workaround may already be gone, or
one that lives in a file `paths` misses. Either way it shows up.

```console
$ canibump
unmarked  serde-pin  no canibump:serde-pin in the scanned files
```

It adds 32 to the exit code, so one alone exits 40. Mark the workaround,
widen `paths`, or drop the blocker. Leave `unmarked` out of `fail-on` to let
it pass. `--json` lists each under `unmarked` with its `id` and `detail`.
Without `[scan]` only declared sites are read, so only they turn up unmarked.

## Files that take no comment

JSON has no comments, so a pin in `package.json` can't carry a marker. Point
the blocker at it instead. `file` is relative to the manifest and `pattern`
is a regex run over each line.

```toml
[[blocker]]
id = "left-pad-pin"
note = "unpin left-pad"
state = "github:left-pad/left-pad#42"
sites = [{ file = "package.json", pattern = '"left-pad": "=1\.3\.0"' }]
```

Every matching line is a site, listed with the markers. Declared sites are
read with or without `[scan]`, and a blocker that declares any never needs a
marker too. Once the pin goes, so does the match.

```console
$ canibump
unmarked  left-pad-pin  no line of package.json matches `"left-pad": "=1\.3\.0"`
```

A missing file reads `package.json is missing`, also `unmarked`. A bad
regex fails the manifest, naming the blocker.

## Catchers

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
