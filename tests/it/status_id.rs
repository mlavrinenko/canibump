//! End-to-end: `canibump status <id>` answers one blocker by exit code,
//! offline: 0 bumpable, 1 waiting, 2 anything else. A bare `status` counts no
//! result checked against another condition.

use predicates::prelude::*;
use serde_json::{Value, json};

use crate::common::{Repo, blocker, open, repo_of};

const OFFLINE: &str = "http://127.0.0.1:9";

/// A repo checked once, so its state file holds all four statuses.
fn checked() -> Repo {
    let repo = repo_of(&["bumpable", "dead", "failed"], "");
    repo.cmd().assert().code(15);
    repo
}

fn write_manifest(repo: &Repo, text: &str) {
    std::fs::write(repo.dir.path().join("canibump.toml"), text).expect("write manifest");
}

#[test]
fn answers_0_for_bumpable_1_for_waiting_and_2_for_the_rest() {
    let repo = checked();
    let cases = [
        (
            "bumpable",
            0,
            "bumpable  bumpable  github:o/r#2 closed as completed (drop the bumpable patch)\n",
        ),
        ("waits", 1, "waiting   waits  github:o/r#1 is open\n"),
        (
            "dead",
            2,
            "dead      dead  github:o/r#3 closed as not planned (drop the dead patch)\n",
        ),
        ("failed", 2, "failed    failed  github:o/r#4: not found\n"),
    ];
    for (id, code, line) in cases {
        repo.cmd()
            .args(["status", id])
            .env("CANIBUMP_GITHUB_API", OFFLINE)
            .assert()
            .code(code)
            .stdout(line)
            .stderr("");
    }
}

#[test]
fn json_holds_only_that_entry_and_the_answer_as_its_exit_code() {
    let repo = checked();
    for (id, code) in [("bumpable", 0), ("waits", 1), ("dead", 2)] {
        let output = repo
            .cmd()
            .args(["--json", "status", id])
            .output()
            .expect("runs");
        assert_eq!(output.status.code(), Some(code), "{id}");
        let report: Value = serde_json::from_slice(&output.stdout).expect("json");
        assert_eq!(entry_ids(&report), [&json!(id)]);
        assert_eq!(report.get("exit_code"), Some(&json!(code)));
    }
}

#[test]
fn a_blocker_the_manifest_lacks_is_unknown() {
    let repo = checked();
    repo.cmd()
        .args(["status", "nope"])
        .assert()
        .code(2)
        .stdout("")
        .stderr("canibump: no blocker `nope` in canibump.toml\n");
}

#[test]
fn a_blocker_the_state_file_lacks_is_unknown() {
    let repo = checked();
    write_manifest(&repo, &(blocker("waits", 1) + &blocker("fresh", 5)));
    repo.cmd()
        .args(["status", "fresh"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("`fresh` has no result"))
        .stderr(predicate::str::contains("run `canibump`"));
}

#[test]
fn no_state_file_at_all_is_unknown() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.cmd()
        .args(["status", "waits"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("run `canibump` first"));
}

#[test]
fn an_edited_condition_is_unknown_until_the_next_check() {
    let repo = checked();
    write_manifest(&repo, &blocker("waits", 9));
    repo.cmd()
        .args(["status", "waits"])
        .assert()
        .code(2)
        .stdout("")
        .stderr("canibump: `waits` was checked against an older condition: run `canibump`\n");
    repo.issue(9, 200, &open());
    repo.cmd().assert().code(0);
    repo.cmd()
        .args(["status", "waits"])
        .assert()
        .code(1)
        .stdout("waiting   waits  github:o/r#9 is open\n");
}

#[test]
fn a_corrupt_state_file_is_unknown_not_waiting() {
    let repo = checked();
    std::fs::write(repo.cache_path(), "junk").expect("write junk");
    repo.cmd()
        .args(["status", "waits"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("not a canibump state file"));
}

#[test]
fn a_bad_manifest_is_unknown_where_a_plain_status_says_1() {
    let repo = checked();
    write_manifest(&repo, "[[blocker]]\nid = \"waits\"\nnote = \"n\"\n");
    repo.cmd()
        .args(["status", "waits"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "blocker `waits`: needs one condition",
        ));
    repo.cmd().arg("status").assert().code(1);
}

#[test]
fn flags_that_do_not_go_with_an_id_are_refused_as_unknown_too() {
    let repo = checked();
    repo.cmd()
        .args(["status", "waits", "--path"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
    repo.cmd()
        .args(["--refresh", "status", "waits"])
        .assert()
        .code(2)
        .stderr("canibump: --refresh goes with check, not status\n");
}

#[test]
fn a_bare_status_does_not_count_a_result_of_an_older_condition() {
    let repo = checked();
    write_manifest(&repo, &(blocker("waits", 1) + &blocker("bumpable", 9)));
    repo.cmd()
        .arg("status")
        .assert()
        .code(0)
        .stdout("waiting   waits  github:o/r#1 is open\n")
        .stderr(
            "canibump: warning: `bumpable` was checked against an older condition: run `canibump`\n",
        );
}

#[test]
fn a_bare_status_skips_a_blocker_the_manifest_dropped() {
    let repo = checked();
    write_manifest(&repo, &blocker("waits", 1));
    repo.cmd()
        .arg("status")
        .assert()
        .code(0)
        .stdout("waiting   waits  github:o/r#1 is open\n")
        .stderr("");
}

#[test]
fn a_bare_status_json_holds_only_the_current_entries() {
    let repo = checked();
    write_manifest(&repo, &(blocker("waits", 1) + &blocker("bumpable", 9)));
    let output = repo
        .cmd()
        .args(["--json", "status"])
        .output()
        .expect("runs");
    assert_eq!(output.status.code(), Some(0));
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(entry_ids(&report), [&json!("waits")]);
}

/// The `id` of each entry of a `--json` report.
fn entry_ids(report: &Value) -> Vec<&Value> {
    report
        .get("entries")
        .and_then(Value::as_array)
        .expect("entries")
        .iter()
        .filter_map(|row| row.get("id"))
        .collect()
}
