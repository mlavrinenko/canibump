set quiet := true

# `-eo pipefail` on every recipe line. No `-u`: recipes read optional env
# and `{{ }}` values that may be empty. A body longer than ~5 lines lives in
# scripts/. Every recipe has a `[doc()]` and a group: `daily` is what bare
# `just` lists, `gate` the arms of `check`, `extra` the rest.
set shell := ["bash", "-eo", "pipefail", "-c"]
set ignore-comments := true

[private]
default:
    just --list --group daily

# clippy-fix runs before fmt: an autofix can land unformatted.
[doc("Apply fixes, then run check")]
[group("daily")]
fix-check: eject clippy-fix fmt check

# Each arm is memoized: mmz skips it while its inputs in .mmz/config.yaml
# are unchanged since it last passed.
[doc("Run every gate, skipping those whose inputs did not change")]
[group("daily")]
[parallel]
check: check-code check-docs

# CI runs the halves as separate jobs, each in its own dev shell: `.#rust`
# here, `.#docs` for check-docs.
[doc("Run the code gates of check")]
[group("gate")]
[parallel]
check-code: (mmz "fmt-check") (mmz "clippy") (mmz "test") (mmz "machete") (mmz "check-file-size") (mmz "check-dry") (mmz "check-tasks") (mmz "outdatty-check")

[doc("Run the docs gates of check")]
[group("gate")]
[parallel]
check-docs: (mmz "docs-build") (mmz "check-doc-blocks") (mmz "check-doc-prose")

# The `--doc` pass is canibump:nextest-doctests. Green prints the summary.
# The status level is in .config/nextest.toml, shared with `cover`.
[doc("Run the tests; ARGS filter them")]
[group("daily")]
[positional-arguments]
test *ARGS:
    out=$(cargo nextest run --workspace "$@" 2>&1) || { printf '%s\n' "$out"; exit 1; }; grep -m1 'Summary' <<< "$out"
    scripts/quiet.sh cargo test --workspace --doc "$@"

[doc("Build the binary")]
[group("daily")]
build *args:
    cargo build --workspace -q {{ args }}

[doc("Format the Rust code")]
[group("daily")]
fmt:
    cargo fmt --all

# A gate too: the build fails on a capture that exits wrong, a dead link or a
# schema type no reference page renders.
[doc("Build the docs site into www/public, silent on success")]
[group("daily")]
[group("gate")]
docs-build:
    www/build.sh

[doc("Render README.md from docs/src/readme.typ")]
[group("daily")]
docs-md:
    www/generate.sh
    docs/generate-md.sh

[doc("Serve the docs with live reload on http://127.0.0.1:5278/canibump/")]
[group("daily")]
docs-serve:
    www/generate.sh
    cd www && tola serve

[doc("Show which gates check would re-run")]
[group("daily")]
status:
    mmz --status

[doc("Record that docs caught up with their sources")]
[group("daily")]
outdatty-update:
    outdatty update

# `wt` places it at `../canibump.<slug>` unless the user configures otherwise.
# Extra flags go to `wt switch`, e.g. `--base=@` to branch from HEAD, not main.
[doc("Fork a sibling worktree on a new branch")]
[group("daily")]
[positional-arguments]
fork SLUG *ARGS:
    wt switch --create --yes "$@"

# Run in the worktree. `--no-squash --no-commit` keeps each commit's `Refs:`
# footer and refuses a dirty tree instead of committing it; the `pre-merge`
# hooks in .config/wt.toml run the gate and the wip check before the
# fast-forward.
[doc("Gate, fast-forward the branch onto main and remove its worktree")]
[group("daily")]
[positional-arguments]
land *ARGS:
    wt merge --no-squash --no-commit --yes "$@"

# Writes the schema next to the binary for `#:schema` lines in canibump.toml;
# ~/.nix's `just install canibump` relies on both.
[doc("Install this checkout's binary and JSON schema")]
[group("daily")]
install:
    cargo install --path . --locked --quiet
    mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/canibump"
    "${CARGO_HOME:-$HOME/.cargo}/bin/canibump" schema > "${XDG_CONFIG_HOME:-$HOME/.config}/canibump/canibump.schema.json"

# A non-zero exit is the answer here (9: a workaround can go), not a failure.
[doc("Check the blockers in canibump.toml (needs network)")]
[group("daily")]
[no-exit-message]
canibump *ARGS:
    cargo run --quiet -- {{ ARGS }}

# `mmz just X` makes `just X` the cache identity in .mmz/config.yaml.
[doc("Run a recipe through mmz")]
[group("gate")]
[no-exit-message]
mmz *ARGS:
    mmz just {{ ARGS }}

[doc("Check Rust formatting")]
[group("gate")]
fmt-check:
    cargo fmt --all -- --check

[doc("Lint with clippy")]
[group("gate")]
clippy:
    cargo clippy --workspace --all-targets -q -- -D warnings

[doc("Apply clippy's fixes")]
[group("gate")]
clippy-fix:
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings

[doc("Find unused dependencies")]
[group("gate")]
machete:
    scripts/quiet.sh cargo machete

[doc("Fail on files over the linecop limits")]
[group("gate")]
check-file-size:
    linecop

[doc("Fail on duplicated Rust blocks of 70+ tokens")]
[group("gate")]
check-dry:
    scripts/quiet.sh jscpd --no-tips -k 70 -f rust --exit-code 1 -r ai src tests

[doc("Fail on a code block typed into a doc source")]
[group("gate")]
check-doc-blocks:
    scripts/check-doc-blocks.sh

[doc("Fail on history words, em dashes, sentence colons and shouting in the docs")]
[group("gate")]
check-doc-prose:
    scripts/check-doc-prose.sh

[doc("Validate the task backlog")]
[group("gate")]
check-tasks:
    # CI's dev shell has no MindTape until truth/tasks/put-mindtape-in-the-dev-shell-so-ci-can-check-the-tasks.typ
    # lands; the commit-msg hook runs `mt check` on every commit meanwhile.
    if [ -n "${CI:-}" ]; then echo "check-tasks: skipped in CI, whose dev shell has no MindTape"; else mt check -q; fi

[doc("Fail when docs lag behind their sources")]
[group("gate")]
outdatty-check:
    outdatty check

[doc("Move inline tests out of Rust files near the size limit")]
[group("gate")]
eject PCT='90':
    scripts/eject.sh '{{ PCT }}'

[doc("Measure coverage, failing under 70% of lines")]
[group("gate")]
cover:
    scripts/cover.sh

[doc("Fail on complex, undertested functions; needs a fresh cover")]
[group("gate")]
crap:
    scripts/crap.sh

# cargo-release bumps, dates CHANGELOG.md, runs scripts/release-hook.sh (every
# refusal), commits and pushes main; the tag waits for ci.yml to go green on it.
[arg('DRY_RUN', long='dry-run', value='true', help='run every check, then say what would be pushed')]
[arg('VERSION', pattern='\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?', help='semver, no leading v (e.g. 0.1.0)')]
[doc("Release: bump, date, push main, wait for CI, then tag (--dry-run first)")]
[extension('.sh')]
[group("extra")]
[no-exit-message]
[script]
release VERSION $DRY_RUN='':
    args=(--no-confirm)
    [ -n "$DRY_RUN" ] || args+=(--execute)
    cargo release {{ VERSION }} "${args[@]}" --no-tag
    if [ -n "$DRY_RUN" ]; then
        echo "dry run: every check passed"
        echo "would push main, wait for ci.yml on it, then tag and push 'v{{ VERSION }}'"
        exit 0
    fi
    scripts/release-ci.sh {{ VERSION }}
    cargo release tag "${args[@]}"
    cargo release push "${args[@]}" \
        || { git tag -d "v{{ VERSION }}" >/dev/null; echo "error: origin rejected the push of 'v{{ VERSION }}'" >&2; echo "hint: re-run just release {{ VERSION }}" >&2; exit 1; }
    echo "pushed 'v{{ VERSION }}'; follow it with: gh run list --workflow release.yml --limit 1"

[doc("Audit dependencies: advisories, licenses, bans (needs network)")]
[group("extra")]
deny:
    scripts/quiet.sh cargo deny check --hide-inclusion-graph advisories licenses bans

[doc("List newer dependency versions (needs network)")]
[group("extra")]
outdated:
    cargo outdated --workspace

# Through a temp file, so a failed build never truncates the schema.
[doc("Regenerate the JSON Schemas in schema/ from the types")]
[group("extra")]
schema:
    cargo run --quiet -- schema manifest > schema/canibump.schema.json.tmp && mv schema/canibump.schema.json.tmp schema/canibump.schema.json
    cargo run --quiet -- schema report > schema/report.schema.json.tmp && mv schema/report.schema.json.tmp schema/report.schema.json
    cargo run --quiet -- schema state > schema/state.schema.json.tmp && mv schema/state.schema.json.tmp schema/state.schema.json

[doc("Mutation-test what this branch changed")]
[group("extra")]
mutants-diff *ARGS:
    mkdir -p target
    git diff "$(git merge-base main HEAD)" > target/branch.diff
    CARGO_MUTANTS_JOBS='{{ env("CARGO_MUTANTS_JOBS", "3") }}' cargo mutants --in-diff target/branch.diff {{ ARGS }}

[doc("Mutation-test every file; slow")]
[group("extra")]
mutants *ARGS:
    cargo mutants {{ ARGS }}

# `nextest list` reads the suite without running it; doctests are not counted.
[doc("Count the tests")]
[group("extra")]
count-tests:
    cargo nextest list --workspace --cargo-quiet --message-format json \
        | jq -r '[.["rust-suites"][].testcases[] | select(.ignored == false)] | "\(length) tests"'

[doc("Show the 20 longest files")]
[group("extra")]
file-sizes:
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | awk 'NR <= 20'

[doc("Point git at .githooks; the dev shell does it on entry")]
[group("extra")]
install-hooks:
    git config core.hooksPath .githooks
    rm -f "$(git rev-parse --git-common-dir)/hooks/pre-commit"
    chmod +x .githooks/pre-commit
    echo "installed .githooks"
