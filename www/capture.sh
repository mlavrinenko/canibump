# www/capture.sh: record commands and what they printed, for the docs pages.
# www/generate.sh sources it, sets `REPO`, `OUT` and `BIN`, and calls
# capture_init before the first capture.
#
# A record is www/generated/<slug>.json:
#
#   {"project": "quickstart",
#    "steps": [{"cmd": "canibump", "out": "...", "code": 9}]}
#
# `cmd` is the line bash ran, `out` is what the terminal showed (stdout and
# stderr, colour included) and `code` is the exit status. `project` names the
# fixture under www/examples/ the commands ran in. A page shows a record with
# terminal() in www/utils/ui.typ, and a fixture file with file(), so a page
# can't show a command that didn't run, or a manifest it didn't run on.
#
# Each command runs in a pseudo-terminal 100 columns wide, because a reader
# runs it in one, and canibump colours its status words there. The home, the
# cache and every token are the capture's own, so the builder's gh or glab
# login changes nothing. Every provider's API points at www/mock-upstream.py,
# which answers from the fixture's upstream.json, so no capture touches the
# network. A request it can't answer fails the build.
#
# Calls with one slug form a session: the steps run in order in one project,
# and each sees the state file the steps before it wrote. A step that exits
# other than expected (0 unless --exit says otherwise) fails the build. Exit 1
# and 2 are errors, so a capture expecting one names its reason with
# `--expect <regex>`, which the output must match once its colour is stripped.

CAPTURE_PATHS=()

capture_init() {
    CAPTURE_TMP=$(cd "$(mktemp -d)" && pwd -P)
    CAPTURE_HOME="$CAPTURE_TMP/home/you"
    mkdir -p "$CAPTURE_TMP/bin" "$CAPTURE_HOME"
    ln -s "$BIN" "$CAPTURE_TMP/bin/canibump"
    echo '{}' >"$CAPTURE_TMP/upstream.json"
    : >"$CAPTURE_TMP/misses"
    python3 -I "$REPO/www/mock-upstream.py" "$CAPTURE_TMP/upstream.json" \
        "$CAPTURE_TMP/port" "$CAPTURE_TMP/misses" &
    CAPTURE_MOCK=$!
    trap 'kill "$CAPTURE_MOCK" 2>/dev/null; rm -rf "$CAPTURE_TMP"' EXIT
    local i
    for i in $(seq 50); do
        [ -s "$CAPTURE_TMP/port" ] && break
        sleep 0.1
    done
    if [ ! -s "$CAPTURE_TMP/port" ]; then
        echo "capture_init: the mock upstream did not start" >&2
        return 1
    fi
    CAPTURE_API="http://127.0.0.1:$(cat "$CAPTURE_TMP/port")"
    # The capture's home reads as /home/you. Without the leading slash, the
    # pair also rewrites the state file's name, which holds the manifest's
    # absolute path.
    CAPTURE_PATHS+=("${CAPTURE_TMP#/}/home/=home/")
}

# capture_copy <var> <fixture>: copy www/examples/<fixture> into the capture
# area, store the copy's path in <var>, and make the fixture's upstream.json
# what the mock answers. The upstream*.json files stay out of the copy, so a
# `[scan]` never reads them. An earlier session's results on the fixture go,
# so each session starts with no state file.
capture_copy() {
    local dest_="$CAPTURE_HOME/$2"
    rm -rf "$dest_" "$CAPTURE_HOME/.cache/canibump/state$dest_"*
    mkdir -p "$dest_"
    tar -C "$REPO/www/examples/$2" --exclude='./upstream*.json' -cf - . | tar -C "$dest_" -xf -
    capture_upstream "$2" upstream.json
    printf -v "$1" "%s" "$dest_"
}

# capture_upstream <fixture> <file>: answer from www/examples/<fixture>/<file>,
# or from nothing when the fixture has none.
capture_upstream() {
    local file="$REPO/www/examples/$1/$2"
    if [ -f "$file" ]; then cp "$file" "$CAPTURE_TMP/upstream.json"; else echo '{}' >"$CAPTURE_TMP/upstream.json"; fi
}

# capture_scratch <var>: an empty directory, for commands that need no project.
capture_scratch() {
    local dest_="$CAPTURE_HOME/scratch"
    mkdir -p "$dest_"
    printf -v "$1" "%s" "$dest_"
}

# Run <command> in <dir> the way every capture runs one, printing what the
# terminal showed and exiting with the command's status.
_capture_run() { # <dir> <command>
    (cd "$1" && env -u BASH_ENV -u NO_COLOR -u CLICOLOR -u CLICOLOR_FORCE -u COLUMNS -u LINES \
        -u GITHUB_TOKEN -u GH_TOKEN -u GITHUB_ENTERPRISE_TOKEN -u GITLAB_TOKEN -u GITLAB_HOST \
        -u GITEA_TOKEN -u GITEA_HOST -u REDMINE_API_KEY -u REDMINE_URL -u JIRA_TOKEN -u JIRA_USER \
        -u JIRA_API_TOKEN -u JIRA_URL -u XDG_CONFIG_HOME -u XDG_DATA_HOME -u GIT_CONFIG_GLOBAL \
        HOME="$CAPTURE_HOME" XDG_CACHE_HOME="$CAPTURE_HOME/.cache" TERM=xterm-256color \
        PATH="$CAPTURE_TMP/bin:$PATH" GH_ENTERPRISE_TOKEN=docs \
        CANIBUMP_GITHUB_API="$CAPTURE_API" CANIBUMP_GITLAB_API="$CAPTURE_API" \
        CANIBUMP_GITEA_API="$CAPTURE_API" CANIBUMP_REDMINE_API="$CAPTURE_API" \
        CANIBUMP_JIRA_API="$CAPTURE_API" CANIBUMP_REPOLOGY_API="$CAPTURE_API" \
        CANIBUMP_DEPSDEV_API="$CAPTURE_API" CAPTURE_CMD="$2" \
        script -qec 'stty cols 100 rows 50 -onlcr; exec bash -c "$CAPTURE_CMD"' /dev/null \
        </dev/null)
}

# capture_prep <dir> <command>: set a session up, recording nothing. A
# failure prints the output and fails the build.
capture_prep() {
    local out
    out=$(_capture_run "$1" "$2") || _capture_fail "$2" "$out" "capture_prep: the command above failed in $1"
}

# _capture_fail <cmd> <out> <message>: print the step and why it failed.
_capture_fail() {
    printf '%s\n' "\$ $1" "$2" "$3" >&2
    return 1
}

# capture [--exit <code>] [--expect <regex>] <slug> <dir> <command>: run
# <command> in <dir> and append it to the record <slug>.
capture() {
    local expect=0 reason=
    while [ "${1:-}" = --exit ] || [ "${1:-}" = --expect ]; do
        if [ "$1" = --exit ]; then expect=$2; else reason=$2; fi
        shift 2
    done
    local slug=$1 dir=$2 cmd=$3 out plain code=0 pair project file="$OUT/$1.json"
    if { [ "$expect" = 1 ] || [ "$expect" = 2 ]; } && [ -z "$reason" ]; then
        echo "capture $slug: --exit $expect is an error, so name it with --expect <regex>" >&2
        return 1
    fi
    project=${dir#"$CAPTURE_HOME"/}
    [ "$project" = scratch ] && project=
    out=$(_capture_run "$dir" "$cmd") || code=$?
    for pair in "${CAPTURE_PATHS[@]}"; do out=${out//"${pair%%=*}"/"${pair#*=}"}; done
    plain=$(printf '%s' "$out" | sed -E $'s/\x1b\\[[0-9;]*[A-Za-z]//g')
    [ "$code" = "$expect" ] || _capture_fail "$cmd" "$out" "capture $slug: exited $code, expected $expect" || return 1
    if [ -n "$reason" ] && ! [[ "$plain" =~ $reason ]]; then
        _capture_fail "$cmd" "$out" "capture $slug: the output does not match --expect '$reason'" || return 1
    fi
    if [ -s "$CAPTURE_TMP/misses" ]; then
        _capture_fail "$cmd" "$out" "capture $slug: the mock upstream has no route for:"$'\n'"$(cat "$CAPTURE_TMP/misses")" || return 1
    fi
    case "$out" in *"$CAPTURE_TMP"* | *"$CAPTURE_API"*)
        _capture_fail "$cmd" "$out" "capture $slug: the output names the capture area or the mock; add a CAPTURE_PATHS pair" || return 1 ;;
    esac
    if [ -f "$file" ]; then
        jq --arg project "$project" --arg cmd "$cmd" --arg out "$out" --argjson code "$code" \
            'if .project == $project then .steps += [{$cmd, $out, $code}]
             else error("one record, two projects: \(.project) and \($project)") end' \
            "$file" >"$file.tmp" && mv "$file.tmp" "$file"
    else
        jq -n --arg project "$project" --arg cmd "$cmd" --arg out "$out" --argjson code "$code" \
            '{$project, steps: [{$cmd, $out, $code}]}' >"$file"
    fi
}

# capture_remote <var> <url>: a local bare repo that every capture reaches at
# <url>, through a git `insteadOf` in the capture's home, and a clone of it in
# <var> to commit and tag in with capture_git.
capture_remote() {
    local name=${2##*/}
    local bare_="$CAPTURE_TMP/remotes/${name%.git}.git" clone_="$CAPTURE_TMP/remotes/${name%.git}"
    git init -q --bare -b main "$bare_"
    git clone -q "$bare_" "$clone_" 2>/dev/null
    git config -f "$CAPTURE_HOME/.gitconfig" "url.file://$bare_.insteadOf" "$2"
    printf -v "$1" "%s" "$clone_"
}

# capture_git <dir> <git args>: run git with a fixed author and clock, so a
# commit's hash, which a manifest names, is the same on every build.
capture_git() {
    local dir=$1
    shift
    (cd "$dir" && GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 \
        GIT_AUTHOR_NAME=docs GIT_AUTHOR_EMAIL=docs@example.org GIT_AUTHOR_DATE=2026-01-01T00:00:00Z \
        GIT_COMMITTER_NAME=docs GIT_COMMITTER_EMAIL=docs@example.org GIT_COMMITTER_DATE=2026-01-01T00:00:00Z \
        git "$@")
}
