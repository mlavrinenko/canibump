//! End-to-end: the built binary against an httpmock GitHub.

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::TempDir;

struct Repo {
    dir: TempDir,
    server: MockServer,
}

impl Repo {
    fn new(manifest: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("canibump.toml"), manifest).expect("write manifest");
        Self {
            dir,
            server: MockServer::start(),
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary should build");
        cmd.current_dir(self.dir.path())
            .env("CANIBUMP_GITHUB_API", self.server.base_url())
            .env("CANIBUMP_REPOLOGY_API", self.server.base_url())
            .env("CANIBUMP_DEPSDEV_API", self.server.base_url())
            .env("GH_TOKEN", "t0ken")
            .env_remove("GITHUB_TOKEN")
            .env("XDG_CACHE_HOME", self.dir.path().join("cache"));
        cmd
    }

    /// Like [`Repo::cmd`], but with no token in the environment and a `gh`
    /// that logs each call and prints one: the log path says who asked.
    fn cmd_with_gh(&self) -> (Command, PathBuf) {
        let bin = self.dir.path().join("bin");
        std::fs::create_dir_all(&bin).expect("mkdir");
        let log = self.dir.path().join("gh.log");
        let script = bin.join("gh");
        let body = format!(
            "#!/bin/sh\necho called >> '{}'\necho t0ken\n",
            log.display()
        );
        std::fs::write(&script, body).expect("write gh");
        std::fs::set_permissions(&script, Permissions::from_mode(0o755)).expect("chmod gh");
        let mut cmd = self.cmd();
        cmd.env_remove("GH_TOKEN").env("PATH", &bin);
        (cmd, log)
    }

    fn issue(&self, number: u64, status: u16, body: &Value) -> Mock<'_> {
        self.server.mock(|when, then| {
            when.method(GET).path(format!("/repos/o/r/issues/{number}"));
            then.status(status).json_body(body.clone());
        })
    }

    fn cache_path(&self) -> PathBuf {
        let manifest = self.dir.path().join("canibump.toml");
        let canonical = manifest.canonicalize().expect("canonical");
        let relative = canonical.strip_prefix("/").expect("absolute");
        self.dir
            .path()
            .join("cache/canibump/state")
            .join(format!("{}.json", relative.display()))
    }
}

/// How often the fake `gh` ran.
fn gh_calls(log: &Path) -> usize {
    std::fs::read_to_string(log).map_or(0, |text| text.lines().count())
}

fn blocker(id: &str, number: u64) -> String {
    format!(
        "[[blocker]]\nid = \"{id}\"\nnote = \"drop the {id} patch\"\nstate = \"github:o/r#{number}\"\n"
    )
}

fn open() -> Value {
    json!({"state": "open"})
}

fn fixed() -> Value {
    json!({"state": "closed", "state_reason": "completed"})
}

#[test]
fn prints_one_line_per_blocker_and_exits_10_when_one_is_bumpable() {
    let repo = Repo::new(&(blocker("waits", 1) + &blocker("lands", 2)));
    repo.issue(1, 200, &open());
    repo.issue(2, 200, &fixed());
    repo.cmd().assert().code(10).stdout(
        "waiting   waits  github:o/r#1 is open\n\
         bumpable  lands  github:o/r#2 closed as completed (drop the lands patch)\n",
    );
}

#[test]
fn exits_0_when_nothing_is_bumpable() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
}

#[test]
fn exits_11_on_a_failed_check_and_12_with_a_bumpable_one_too() {
    let repo = Repo::new(&(blocker("gone", 3) + &blocker("lands", 2)));
    repo.issue(3, 404, &json!({"message": "Not Found"}));
    repo.cmd()
        .arg("check")
        .assert()
        .code(11)
        .stdout(predicate::str::contains(
            "failed    gone   github:o/r#3: not found",
        ));
    repo.issue(2, 200, &fixed());
    repo.cmd().assert().code(12);
}

#[test]
fn reuses_a_fresh_result_and_rechecks_on_refresh() {
    let repo = Repo::new(&blocker("waits", 1));
    let mock = repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
    repo.cmd().assert().code(0);
    mock.assert_calls(1);
    repo.cmd().arg("--refresh").assert().code(0);
    mock.assert_calls(2);
}

#[test]
fn refresh_works_on_check_and_bare_but_status_refuses_it() {
    let repo = Repo::new(&blocker("waits", 1));
    let mock = repo.issue(1, 200, &open());
    repo.cmd().arg("check").assert().code(0);
    repo.cmd().args(["check", "--refresh"]).assert().code(0);
    mock.assert_calls(2);
    repo.cmd().arg("--refresh").assert().code(0);
    mock.assert_calls(3);
    repo.cmd().args(["--refresh", "check"]).assert().code(0);
    mock.assert_calls(4);
    repo.cmd()
        .args(["--refresh", "status"])
        .assert()
        .code(1)
        .stderr("canibump: --refresh goes with check, not status\n");
    mock.assert_calls(4);
}

#[test]
fn global_flags_work_before_a_subcommand() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
    repo.cmd()
        .args(["--json", "status"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"status\": \"waiting\""));
    repo.cmd()
        .args(["-m", "canibump.toml", "status"])
        .assert()
        .code(0)
        .stdout("waiting   waits  github:o/r#1 is open\n");
}

#[test]
fn never_caches_a_failure_as_fresh() {
    let repo = Repo::new(&blocker("gone", 3));
    let mock = repo.issue(3, 500, &json!({"message": "boom"}));
    repo.cmd().assert().code(11);
    repo.cmd().assert().code(11);
    assert!(mock.calls() >= 2, "a failure must be re-checked");
}

#[test]
fn status_reads_the_cache_without_network() {
    let repo = Repo::new(&blocker("lands", 2));
    let mut mock = repo.issue(2, 200, &fixed());
    repo.cmd().assert().code(10);
    mock.delete();
    repo.cmd()
        .args(["status", "--json"])
        .env("CANIBUMP_GITHUB_API", "http://127.0.0.1:9")
        .assert()
        .code(10)
        .stdout(predicate::str::contains("\"status\": \"bumpable\""));
    let path = repo.cache_path();
    let state: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cache file")).expect("json");
    let Some([entry]) = state
        .get("entries")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
    else {
        panic!("one entry expected: {state}");
    };
    assert_eq!(entry.get("id"), Some(&json!("lands")));
    assert_eq!(entry.get("status"), Some(&json!("bumpable")));
    assert!(
        entry.get("checked_at").is_some_and(Value::is_string),
        "{entry}"
    );
    assert!(
        entry.get("fresh_until").is_some_and(Value::is_string),
        "{entry}"
    );
}

#[test]
fn status_marks_stale_entries() {
    let repo = Repo::new(&("[settings]\nttl = \"0s\"\n".to_owned() + &blocker("waits", 1)));
    repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
    repo.cmd().arg("status").assert().code(0).stdout(
        predicate::str::is_match(
            r"^waiting   waits  github:o/r#1 is open \(stale, checked \S+\)\n$",
        )
        .expect("regex"),
    );
}

#[test]
fn status_path_prints_where_the_cache_lives() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.cmd()
        .args(["status", "--path"])
        .assert()
        .success()
        .stdout(format!("{}\n", repo.cache_path().display()));
}

#[test]
fn status_without_a_cache_says_to_run_a_check() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.cmd()
        .arg("status")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("run `canibump` first"));
}

#[test]
fn json_check_output_lists_entries() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    let output = repo.cmd().arg("--json").output().expect("runs");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.pointer("/entries/0/id"), Some(&json!("waits")));
    assert_eq!(report.pointer("/entries/0/status"), Some(&json!("waiting")));
    assert_eq!(report.pointer("/entries/0/stale"), Some(&json!(false)));
    assert_eq!(report.get("exit_code"), Some(&json!(0)));
}

#[test]
fn a_bad_manifest_exits_1_naming_the_problem() {
    let repo = Repo::new("[[blocker]]\nid = \"x\"\nnote = \"n\"\n");
    repo.cmd()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("blocker `x`: needs one condition"));
}

#[test]
fn a_toml_syntax_error_prints_once_naming_the_manifest() {
    let repo = Repo::new("[[blocker]]\nid = \"x\"\nnote = \"n\"\nstate = github\n");
    let output = repo.cmd().output().expect("runs");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("canibump: canibump.toml: TOML parse error at line 4, column 9\n"),
        "{stderr}"
    );
    assert_eq!(stderr.matches("TOML parse error").count(), 1, "{stderr}");
}

#[test]
fn a_blocker_error_names_the_manifest_as_given_and_the_blocker_line() {
    let repo = Repo::new(&(blocker("a", 1) + &blocker("x", 2) + "foo = 1\n"));
    std::fs::rename(
        repo.dir.path().join("canibump.toml"),
        repo.dir.path().join("deps.toml"),
    )
    .expect("rename");
    repo.cmd()
        .args(["-m", "./deps.toml"])
        .assert()
        .code(1)
        .stderr("canibump: ./deps.toml:5: blocker `x`: unknown field `foo`, expected `state`\n");
}

#[test]
fn a_provider_that_cannot_start_says_why_on_its_blockers_line() {
    let repo = Repo::new(
        &(blocker("x", 1)
            + "\n[[blocker]]\nid = \"pkg\"\nnote = \"n\"\nversion = \"deps.dev:cargo/serde\"\nat-least = \"1.0.0\"\n"),
    );
    repo.server.mock(|when, then| {
        when.method(GET).path("/v3/systems/cargo/packages/serde");
        then.status(200)
            .json_body(json!({"versions": [{"versionKey": {"version": "1.0.228"}}]}));
    });
    repo.cmd()
        .env("CANIBUMP_GITHUB_API", "not-a-url")
        .assert()
        .code(12)
        .stdout(
            "failed    x    github: API URL `not-a-url`: relative URL without a base\n\
             bumpable  pkg  deps.dev:cargo/serde is at 1.0.228, at least 1.0.0 (n)\n",
        );
}

#[test]
fn a_corrupt_state_file_warns_on_stderr_and_the_run_goes_on() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    let path = repo.cache_path();
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, "junk").expect("write junk");
    repo.cmd()
        .assert()
        .code(0)
        .stdout("waiting   waits  github:o/r#1 is open\n")
        .stderr(predicate::str::starts_with(format!(
            "canibump: warning: ignoring {}: not a canibump state file",
            path.display()
        )));
}

#[test]
fn a_manifest_without_a_github_blocker_never_runs_gh() {
    let repo = Repo::new(
        "[[blocker]]\nid = \"nix\"\nnote = \"drop the overlay\"\nversion = \"repology:llama.cpp\"\nrepo = \"nix_unstable\"\nat-least = \"0.6.0\"\n",
    );
    repo.server.mock(|when, then| {
        when.method(GET).path("/project/llama.cpp");
        then.status(200)
            .json_body(json!([{"repo": "nix_unstable", "version": "0.6.0"}]));
    });
    let (mut cmd, log) = repo.cmd_with_gh();
    cmd.assert().code(10);
    assert_eq!(gh_calls(&log), 0, "no blocker needs GitHub");
}

#[test]
fn a_github_state_and_a_reached_blocker_ask_gh_once() {
    let repo = Repo::new(&format!(
        "{}\n[[blocker]]\nid = \"landed\"\nnote = \"n\"\nreached = \"github:o/r@abc1234\"\nin = {{ branch = \"main\" }}\n",
        blocker("waits", 1)
    ));
    repo.issue(1, 200, &open());
    repo.server.mock(|when, then| {
        when.method(GET).path("/repos/o/r/compare/abc1234...main");
        then.status(200).json_body(json!({"status": "ahead"}));
    });
    let (mut cmd, log) = repo.cmd_with_gh();
    cmd.assert().code(10);
    assert_eq!(gh_calls(&log), 1, "tracker and code host share one client");
}

#[test]
fn a_chain_on_one_pr_spends_two_calls_before_its_tag_search() {
    let repo = Repo::new(
        "[[blocker]]\nid = \"chain\"\nnote = \"n\"\n\n\
[[blocker.all]]\nid = \"merged\"\nstate = \"github:o/r#1\"\n\n\
[[blocker.all]]\nid = \"released\"\nreached = \"github:o/r#1\"\nin = { tags = \"v*\" }\n",
    );
    let merged = json!({"state": "closed", "pull_request": {"merged_at": "2025-11-02T10:30:00Z"}});
    let issue = repo.issue(1, 200, &merged);
    let pull = repo.server.mock(|when, then| {
        when.method(GET).path("/repos/o/r/pulls/1");
        then.status(200)
            .json_body(json!({"merged": true, "merge_commit_sha": "abc1234"}));
    });
    let tags = repo.server.mock(|when, then| {
        when.method(GET).path("/repos/o/r/git/matching-refs/tags/v");
        then.status(200).json_body(json!([]));
    });
    repo.cmd().assert().code(0);
    assert_eq!(
        issue.calls(),
        1,
        "the state step and the reached step share it"
    );
    assert_eq!(pull.calls(), 1, "only the merge commit needs the pull");
    assert_eq!(tags.calls(), 1);
}

#[test]
fn a_manifest_setting_moves_the_cache() {
    let repo =
        Repo::new(&("[settings]\ncache = \"state.json\"\n".to_owned() + &blocker("waits", 1)));
    repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
    assert!(repo.dir.path().join("state.json").is_file());
    let explicit: &Path = &repo.dir.path().join("elsewhere.json");
    repo.cmd().arg("--cache").arg(explicit).assert().code(0);
    assert!(explicit.is_file());
}

#[test]
fn reports_version() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .expect("binary should build")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn checks_package_versions_on_repology_and_deps_dev() {
    let repo = Repo::new(
        "[[blocker]]\nid = \"nix\"\nnote = \"drop the overlay\"\nversion = \"repology:llama.cpp\"\nrepo = \"nix_unstable\"\nat-least = \"0.6.0\"\n\n\
         [[blocker]]\nid = \"crate\"\nnote = \"unpin\"\nversion = \"deps.dev:cargo/serde\"\nat-least = \"2.0.0\"\n",
    );
    repo.server.mock(|when, then| {
        when.method(GET).path("/project/llama.cpp");
        then.status(200).json_body(
            json!([{"repo": "nix_unstable", "srcname": "llama-cpp", "version": "0.6.0"}]),
        );
    });
    repo.server.mock(|when, then| {
        when.method(GET).path("/v3/systems/cargo/packages/serde");
        then.status(200)
            .json_body(json!({"versions": [{"versionKey": {"version": "1.0.228"}}]}));
    });
    repo.cmd().assert().code(10).stdout(
        "bumpable  nix    repology:llama.cpp in nix_unstable is at 0.6.0, at least 0.6.0 (drop the overlay)\n\
         waiting   crate  deps.dev:cargo/serde is at 1.0.228, waiting for 2.0.0\n",
    );
}
