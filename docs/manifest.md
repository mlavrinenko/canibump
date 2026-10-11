# canibump.toml

Everything a manifest can say. The [README](../README.md) has the quick tour.

## Conditions

Each blocker takes exactly one condition.

`state` waits on an issue, PR or MR. Merged, or closed as completed, is bumpable.

```toml
state = "github:serde-rs/serde#2590"
# state = "gitlab:group/project!12"                    # MR on gitlab.com; #12 is an issue
# state = "gitlab:gitlab.example.org/group/project#3" # self-hosted
# state = "codeberg:owner/repo#5"                     # or gitea:host/owner/repo, forgejo:host/owner/repo
# state = "redmine:redmine.example.org#123"
# state = "jira:issues.apache.org/jira#KAFKA-42"       # a path when Jira lives under one
```

GitLab and Gitea keep no close reason. A closed GitLab issue counts only when
a merged MR closed it, and a closed Gitea issue reads `dead`, so point at the PR
instead. Redmine and Jira say why by status or resolution name, matched on
whole words: `Rejected`, `Won't Fix`, `Invalid` or `Abandoned` read dead,
`Fixed` or `Done` read bumpable, `Not done` does not.

`reached` waits for a commit, or a PR's merge commit, to land in a branch, a
tag, or the first tag matching a pattern, in version order. It reads history
on GitHub and any git remote, and a target elsewhere fails the manifest. An
issue has no merge commit and reads `failed`, open or closed, so wait on it
with `state`. A
short sha matching two commits fails and asks for more digits. One version
order covers semver, `v`-tags, dates and builds like `b7012`.

```
v1.0.0 = 1.0.0 = 1.0.0+build     # a letter prefix and +build drop
1.0 = 1.0.0                      # a missing trailing .0 counts as zero
b999 < b1000                     # numbers compare as numbers
1.0.0-rc9 < 1.0.0-rc10 < 1.0.0   # letters after the numbers mean pre-release
```

```toml
reached = "github:serde-rs/serde#2590"     # or "github:owner/repo@<sha>"
# reached = "git:https://github.com/rust-lang/glob.git@952da29"  # any git remote
in = { branch = "master" }
# in = { tag = "v1.0.185" }
# in = { tags = "v*", since = "1.0.180" }
```

Tags fall into release lines by their first two numbers (`v1.4.2` and
`v1.4.0-rc1` share `1.4`, all `b`-builds share one), so a fix shipped only on
an old line, in `v1.5.0` but no `v2`, still counts. canibump asks each line's
newest tag, then binary-searches the first line whose newest has it, a call
per line plus a few. A tag in that line cut from another branch can't make it
name the wrong tag. It costs extra calls instead, never more than one per tag. A line whose newest tag
lacks the commit counts as not having it, since an old line forked before the
fix looks just the same. `since` trims the lines to ask.

`version` waits for a package to reach some version. Repology covers distro
repos, nixpkgs channels included (`nix_unstable`, `nix_stable_25_11`).
deps.dev covers `cargo`, `npm`, `pypi`, `go`, `maven`, `nuget` and `rubygems`.

```toml
version = "repology:llama.cpp"
repo = "nix_unstable"
srcname = "llama-cpp"    # optional, when a repo ships several
at-least = "0.6.0"

# version = "deps.dev:cargo/serde"
# at-least = "2.0.0"
```

## Chains

Real blockers come in chains. `[[blocker.all]]` holds when every step does,
stopping at the first that doesn't. `[[blocker.any]]` holds when one does.
Steps nest, and an `id` hands the tag a `reached` step found to a later step.

```toml
[[blocker]]
id = "llama-blocking-sync"
note = "pkgs/llama-cpp.nix: drop the GGML_CUDA_BLOCKING_SYNC postPatch"

[[blocker.all]]
id = "merged"
state = "github:ggml-org/llama.cpp#26487"

[[blocker.all]]
id = "released"
reached = "github:ggml-org/llama.cpp#26487"
in = { tags = "v*" }

[[blocker.all]]
version = "repology:llama.cpp"
repo = "nix_unstable"
srcname = "llama-cpp"
at-least = { step = "released" }
```

```console
$ canibump
waiting   llama-blocking-sync  merged: github:ggml-org/llama.cpp#26487 is open
```

A `step` reference may only point at a step that surely held by then: an
earlier step of the same `all`, or of an enclosing one. An `any` may resolve
through another step, so canibump refuses a manifest that reaches into it.
Put both steps in one `all`.

A dead or failed step never lets the chain read bumpable. In an `any`, failed
beats waiting, and the whole thing is dead only when every step is.

## Cache

Every run writes `$XDG_CACHE_HOME/canibump/state/<absolute manifest path>.json`
(`~/.cache/...` without XDG, `%LOCALAPPDATA%\...` on Windows), so other tools
read the state offline.

```json
{"version": 1, "manifest": "/home/me/repo/canibump.toml", "entries": [
  {"id": "serde-pin", "note": "Cargo.toml: unpin serde", "status": "bumpable",
   "detail": "github:serde-rs/serde#2590 (110af31) is in v1.0.185",
   "checked_at": "2026-10-10T12:00:00Z", "fresh_until": "2026-10-10T18:00:00Z",
   "spec": "reached = ..."}]}
```

Results stay fresh for `[settings] ttl = "6h"`, the default. It takes weeks,
days, hours, minutes and seconds (`"1d"`, `"2w"`, `"1d12h"`), a day being 24
hours. A failed check is never fresh, so the next run retries it. Move the
file with `--cache FILE` or `[settings] cache = "FILE"`, relative to the
manifest. A state file canibump can't parse is dropped with a warning, and the
run rewrites it.

## Exit codes

A run exits 0 when nothing counted showed up, else 8 plus 1 for bumpable, 2
for dead and 4 for failed, summed, so bumpable and dead exit 11. 1 is an
unusable manifest or cache, 2 a bad argument.

All three count unless `[settings] fail-on` lists the ones that do, and
`--fail-on` beats it. `waiting` never counts. A status that doesn't count
still prints its line.

```toml
[settings]
fail-on = ["failed"]  # dead and bumpable print, the run still exits 0
# fail-on = []        # never fail
```

`canibump status <id>` answers for one blocker from the state file alone. 0 is
bumpable, 1 waiting, and 2 anything else: dead, failed, an unknown id, a
result checked against an older condition, and every error. It prints that
blocker's line, or with `--json` a report holding only it.

```console
$ canibump status serde-pin; echo $?
bumpable  serde-pin  github:serde-rs/serde#2590 merged (Cargo.toml: unpin serde)
0
```

A bare `canibump status` prints the state file as it stands, minus what the
manifest has moved past. A result checked against an older condition is not
counted and earns a warning on stderr, and a blocker the manifest dropped is
skipped.

## Auth

| Target | Token |
| --- | --- |
| GitHub | `GITHUB_TOKEN`, `GH_TOKEN`, then `gh auth token`; none works too, with a tighter rate limit |
| GitLab | glab's config per host, `GITLAB_TOKEN` for `GITLAB_HOST` (default gitlab.com) |
| Gitea | tea's config logins, `GITEA_TOKEN` for `GITEA_HOST` |
| Redmine | `REDMINE_API_KEY` for `REDMINE_URL` |
| Jira | `JIRA_TOKEN`, or `JIRA_USER` with `JIRA_API_TOKEN`, for `JIRA_URL` |

A token only ever goes to its own host, and only when a blocker needs it. A
redirect to another host fails the check rather than carry the token along.

`CANIBUMP_<PROVIDER>_API` (`GITHUB`, `GITLAB`, `GITEA`, `REDMINE`, `JIRA`,
`REPOLOGY`, `DEPSDEV`) sends every request of that provider to another API
root, say a GitHub Enterprise box or a mirror. The override gets the token
filed under its own host, never the item's host's or github.com's. On GitHub
Enterprise that's `GH_ENTERPRISE_TOKEN`, `GITHUB_ENTERPRISE_TOKEN`, then
`gh auth token --hostname <host>`, the way gh picks one.

```console
$ CANIBUMP_GITHUB_API=https://ghe.example.com/api/v3 GH_ENTERPRISE_TOKEN=ghp_xxx canibump
$ CANIBUMP_GITLAB_API=https://mirror.example.com GITLAB_HOST=mirror.example.com GITLAB_TOKEN=glpat_xxx canibump
```

Here `GITHUB_TOKEN` and `GH_TOKEN` stay home, and the mirror never sees the
gitlab.com token.

## Schema

`canibump schema` prints the manifest's JSON Schema, generated from the types
that parse it, and `canibump schema report` and `canibump schema state` those
of `--json` and the state file. All three are committed under
[schema/](../schema/). `just install` copies the manifest's to
`~/.config/canibump/`, and the Nix package ships it as
`share/canibump/canibump.schema.json`, so an editor finds it offline.
