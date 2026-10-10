# canibump.toml

Everything a manifest can say. The [README](../README.md) has the quick tour.

## Conditions

Each blocker takes exactly one condition.

`state` waits on an issue, PR or MR. Merged or closed as completed is bumpable.

```toml
state = "github:serde-rs/serde#2590"
# state = "gitlab:group/project!12"                    # MR on gitlab.com; #12 is an issue
# state = "gitlab:gitlab.example.org/group/project#3" # self-hosted
# state = "codeberg:owner/repo#5"                     # or gitea:host/owner/repo, forgejo:host/owner/repo
# state = "redmine:redmine.example.org#123"
# state = "jira:issues.apache.org/jira#KAFKA-42"       # a path when Jira lives under one
```

GitLab keeps no close reason, so a closed GitLab issue counts only when an MR
closed it. Gitea keeps none either, so a closed Gitea issue reads `dead`;
point at the PR instead. Redmine and Jira say why by status or resolution
name, matched on whole words, so `Rejected`, `Won't Fix`, `Invalid` or
`Abandoned` read dead and `Fixed` or `Done` read bumpable.

`reached` waits for a commit, or a PR's merge commit, to land in a branch, a tag, or
any tag matching a pattern. With a pattern, canibump names the first tag
that has it, in version order. One order covers semver, `v`-tags, dates and
builds like `b7012`:

```
v1.0.0 = 1.0.0 = 1.0.0+build     # a letter prefix and +build drop
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

GitHub answers through its API, no clone. A `git:` remote gets a bare clone of
just the refs you ask about under `~/.cache/canibump/git/`, via gix. If two
commits there start with your short sha, the check fails and asks for more digits.

`version` waits for a package to reach some version. Repology covers distro repos,
nixpkgs channels included (`nix_unstable`, `nix_stable_25_11`). deps.dev
covers `cargo`, `npm`, `pypi`, `go`, `maven`, `nuget` and `rubygems`.

```toml
version = "repology:llama.cpp"
repo = "nix_unstable"
srcname = "llama-cpp"    # optional, when a repo ships several
at-least = "0.6.0"

# version = "deps.dev:cargo/serde"
# at-least = "2.0.0"
```

canibump asks Repology once per project per run, at most once a second, with
a User-Agent naming itself, as Repology's API rules ask. Blockers that ask
about one project at the same time share that one request.

## Chains

Real blockers come in chains. `[[blocker.all]]` holds when every step does,
stopping at the first that doesn't; `[[blocker.any]]` when one does. Steps
nest, and an `id` hands the tag a `reached` step found to a later step. A
`step` reference may only point at a step that surely held by then: an earlier
step of the same `all`, or of an enclosing `all`. An `any` may resolve through
another step, so ids inside it are out of reach after it; canibump refuses such
a manifest at load. Put both steps in one `all`.

Steps run in turn, since each may hand a value on. Whole blockers run side by
side, eight at a time.

```toml
[[blocker]]
id = "llama-blocking-sync"
note = "apps/llama-cpp-16gb.nix: drop the GGML_CUDA_BLOCKING_SYNC postPatch"

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

That one is real. It guards a patch in the author's NixOS flake.

```console
$ canibump
waiting   llama-blocking-sync  merged: github:ggml-org/llama.cpp#26487 is open
```

A dead or failed step never lets the chain read bumpable. In an any-of, a
failed step beats a waiting one, and the whole thing is dead only when every
step is.

A run asks each question once. Name the same PR in three steps, or in three
blockers, and its state and its merge commit are fetched once, even when the
asks overlap. That chain costs GitHub two calls before the tag search:
`/issues/26487` for the state, `/pulls/26487` for the merge commit. A failed
fetch is kept too, so every asker hears the same reason.

## Cache

Every run writes `$XDG_CACHE_HOME/canibump/state/<absolute manifest path>.json`
(`~/.cache/...` without XDG), so other tools read the state offline.

```json
{"version": 1, "manifest": "/home/me/repo/canibump.toml", "entries": [
  {"id": "serde-pin", "note": "Cargo.toml: unpin serde", "status": "bumpable",
   "detail": "github:serde-rs/serde#2590 (110af31) is in v1.0.185",
   "checked_at": "2026-10-10T12:00:00Z", "fresh_until": "2026-10-10T18:00:00Z",
   "spec": "reached = ..."}]}
```

Results stay fresh for `[settings] ttl = "6h"` (the default). Move the file
with `--cache FILE` or `[settings] cache = "FILE"` (relative to the
manifest). A failed check is never fresh, so the next run retries it.

A state file canibump cannot parse is dropped with a warning on stderr, and
the run rewrites it:

```text
canibump: warning: ignoring /home/me/.cache/canibump/state/...json: not a canibump state file: expected value at line 1 column 1
```

## Exit codes

A run exits 0 when nothing counted showed up, else 8 plus 1 for bumpable, 2 for
dead and 4 for failed, summed: bumpable and dead together exit 11. 1 is an
unusable manifest or cache. `canibump status` follows the same rules.

All three count unless `[settings] fail-on` lists the ones that do. The
`--fail-on` flag beats it. `waiting` never counts, so neither can name it, and
a status that does not count still prints its line.

```toml
[settings]
fail-on = ["failed"]  # dead and bumpable print, the run still exits 0
# fail-on = []        # never fail
```

## Auth

GitHub takes `GITHUB_TOKEN`, then `GH_TOKEN`, then whatever `gh auth token`
says. No token works too, just with GitHub's tighter rate limit.
GitLab takes glab's config per host and `GITLAB_TOKEN` for `GITLAB_HOST`
(default gitlab.com). Gitea takes tea's config logins and `GITEA_TOKEN` for
`GITEA_HOST`. Redmine takes `REDMINE_API_KEY` for `REDMINE_URL`. Jira takes
`JIRA_TOKEN`, or `JIRA_USER` with `JIRA_API_TOKEN`, for `JIRA_URL`. A token
only ever goes to its own host. Credentials are read when a blocker first
needs them: a manifest with no GitHub blocker never runs `gh`, and a run that
reuses every cached result reads none.


## Schema

`canibump schema` prints a JSON Schema for the manifest, generated from the
types that parse it and committed as
[schema/canibump.schema.json](../schema/canibump.schema.json). A test fails
when the two differ; `just schema` regenerates the file. The README shows the
`#:schema` line that wires it into taplo and tombi.
