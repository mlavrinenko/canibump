//! End-to-end: which statuses fail a run, and the exit code they add up to.

use predicates::prelude::*;
use serde_json::{Value, json};

use crate::common::repo_of;

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
        .stdout(predicate::str::contains("16 for an unlisted link"))
        .stdout(predicate::str::contains(
            "1 is an unusable manifest or cache, 2 a bad argument",
        ))
        .stdout(predicate::str::contains(
            "else bumpable, dead, failed, unlisted and",
        ))
        .stdout(predicate::str::contains(
            "`status <id>` answers for one blocker",
        ));
}

const SCAN: &str = "[scan]\npaths = [\"*.nix\"]\n\n";

/// [`repo_of`] with a `[scan]` and a `pin.nix` of `text`.
fn scanned(kinds: &[&str], settings: &str, text: &str) -> crate::common::Repo {
    let repo = repo_of(kinds, &(settings.to_owned() + SCAN));
    std::fs::write(repo.dir.path().join("pin.nix"), text).expect("write");
    repo
}

/// [`scanned`] with a `pin.nix` marking every blocker and linking an issue no
/// blocker lists.
fn with_unlisted(kinds: &[&str], settings: &str) -> crate::common::Repo {
    let marks: String = kinds
        .iter()
        .map(|kind| format!(" canibump:{kind}"))
        .collect();
    let text = format!("# https://github.com/o/r/issues/99\n# canibump:waits{marks}\n");
    scanned(kinds, settings, &text)
}

#[test]
fn an_unlisted_link_adds_16_unless_fail_on_leaves_it_out() {
    with_unlisted(&[], "").cmd().assert().code(24);
    let repo = with_unlisted(&["bumpable"], "");
    repo.cmd().assert().code(25);
    repo.cmd()
        .args(["--fail-on", "bumpable"])
        .assert()
        .code(9)
        .stdout(predicate::str::contains("unlisted  pin.nix:1"));
    repo.cmd().args(["--fail-on", "unlisted"]).assert().code(24);
    with_unlisted(&["bumpable"], "[settings]\nfail-on = [\"unlisted\"]\n")
        .cmd()
        .assert()
        .code(24);
}

#[test]
fn the_json_report_lists_each_unlisted_link() {
    let output = with_unlisted(&[], "")
        .cmd()
        .arg("--json")
        .output()
        .expect("runs");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.get("exit_code"), Some(&json!(24)));
    assert_eq!(
        report.get("unlisted"),
        Some(&json!([{"file": "pin.nix", "line": 1, "ref": "github:o/r#99", "catcher": "github"}]))
    );
    let output = repo_of(&[], "").cmd().arg("--json").output().expect("runs");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.get("unlisted"), Some(&json!([])));
}

#[test]
fn a_bumpable_line_names_where_its_marker_sits() {
    scanned(&["bumpable"], "", "x\n# canibump:bumpable canibump:waits\n")
        .cmd()
        .assert()
        .code(9)
        .stdout(predicate::str::contains("\n          at pin.nix:2\n"));
}

#[test]
fn an_unmarked_blocker_adds_32_unless_fail_on_leaves_it_out() {
    let repo = scanned(&["bumpable"], "", "# canibump:bumpable\n");
    repo.cmd()
        .assert()
        .code(41)
        .stdout(predicate::str::contains(
            "\nunmarked  waits  no canibump:waits in the scanned files\n",
        ));
    repo.cmd().args(["--fail-on", "bumpable"]).assert().code(9);
    repo.cmd().args(["--fail-on", "unmarked"]).assert().code(40);
    scanned(&[], "", "").cmd().assert().code(40);
    repo_of(&[], "").cmd().assert().code(0);
}

#[test]
fn the_json_report_lists_each_unmarked_blocker() {
    let output = scanned(&["dead"], "", "")
        .cmd()
        .arg("--json")
        .output()
        .expect("runs");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.get("exit_code"), Some(&json!(42)));
    assert_eq!(report.get("unmarked"), Some(&json!(["waits", "dead"])));
    let (_, report) = crate::common::waiting_json_run();
    assert_eq!(report.get("unmarked"), Some(&json!([])));
}

#[test]
fn status_scans_as_a_check_does() {
    let repo = scanned(&[], "", "# https://github.com/o/r/issues/99\n");
    repo.cmd().assert().code(56);
    repo.cmd()
        .arg("status")
        .assert()
        .code(56)
        .stdout(predicate::str::contains("unlisted  pin.nix:1"))
        .stdout(predicate::str::contains("unmarked  waits"));
}
