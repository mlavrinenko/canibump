//! End-to-end: which statuses fail a run, and the exit code they add up to.

mod common;

use predicates::prelude::*;
use serde_json::{Value, json};

use common::repo_of;

#[test]
fn a_dead_blocker_fails_the_run_with_10() {
    let repo = repo_of(&["dead"], "");
    repo.cmd()
        .assert()
        .code(10)
        .stdout(predicate::str::contains("dead      dead"));
}

#[test]
fn the_code_adds_a_bit_per_status_that_showed_up() {
    let cases: [(&[&str], i32); 7] = [
        (&["bumpable"], 9),
        (&["dead"], 10),
        (&["bumpable", "dead"], 11),
        (&["failed"], 12),
        (&["bumpable", "failed"], 13),
        (&["dead", "failed"], 14),
        (&["bumpable", "dead", "failed"], 15),
    ];
    for (kinds, code) in cases {
        repo_of(kinds, "").cmd().assert().code(code);
    }
}

#[test]
fn fail_on_picks_what_counts_and_the_lines_still_print() {
    let repo = repo_of(&["bumpable", "dead", "failed"], "");
    repo.cmd().args(["--fail-on", "failed"]).assert().code(12);
    repo.cmd()
        .args(["--fail-on", "dead"])
        .assert()
        .code(10)
        .stdout(predicate::str::contains("bumpable  bumpable"));
    let both = ["--fail-on", "bumpable,dead"];
    repo.cmd().args(both).assert().code(11);
    let repeated = ["--fail-on", "bumpable", "--fail-on", "dead"];
    repo.cmd().args(repeated).assert().code(11);
    repo.cmd()
        .args(["check", "--fail-on", "dead"])
        .assert()
        .code(10);
}

#[test]
fn a_status_that_does_not_count_exits_0_on_the_same_state() {
    let repo = repo_of(&["dead"], "");
    repo.cmd().assert().code(10);
    repo.cmd()
        .args(["--fail-on", "failed"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("dead      dead"));
}

#[test]
fn a_settings_fail_on_applies_and_the_flag_overrides_it() {
    let repo = repo_of(&["dead"], "[settings]\nfail-on = [\"failed\"]\n");
    repo.cmd().assert().code(0);
    repo.cmd().args(["--fail-on", "dead"]).assert().code(10);
}

#[test]
fn an_empty_settings_fail_on_never_fails() {
    let repo = repo_of(
        &["bumpable", "dead", "failed"],
        "[settings]\nfail-on = []\n",
    );
    repo.cmd().assert().code(0);
    repo.cmd().args(["--fail-on", "dead"]).assert().code(10);
}

#[test]
fn status_applies_the_same_policy_as_check() {
    let repo = repo_of(
        &["bumpable", "dead", "failed"],
        "[settings]\nfail-on = [\"bumpable\", \"failed\"]\n",
    );
    repo.cmd().assert().code(13);
    repo.cmd().arg("status").assert().code(13);
    repo.cmd()
        .args(["status", "--fail-on", "dead"])
        .assert()
        .code(10);
}

#[test]
fn json_reports_the_code_the_policy_picked() {
    let repo = repo_of(&["dead"], "");
    let output = repo
        .cmd()
        .args(["--json", "--fail-on", "failed"])
        .output()
        .expect("runs");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.get("exit_code"), Some(&json!(0)));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn waiting_cannot_be_named() {
    let repo = repo_of(&[], "");
    repo.cmd()
        .args(["--fail-on", "waiting"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value 'waiting'"))
        .stderr(predicate::str::contains("bumpable, dead, failed"));
}

#[test]
fn an_unknown_settings_fail_on_is_a_manifest_error_with_its_line() {
    let repo = repo_of(&[], "[settings]\nfail-on = [\"waiting\"]\n");
    repo.cmd()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("line 2"))
        .stderr(predicate::str::contains("unknown variant `waiting`"));
}

#[test]
fn help_states_the_table_and_the_default() {
    let repo = repo_of(&[], "");
    repo.cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("8 plus 1 for bumpable"))
        .stdout(predicate::str::contains("else bumpable, dead and failed"))
        .stdout(predicate::str::contains(
            "`status <id>` answers for one blocker",
        ));
}
