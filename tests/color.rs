//! End-to-end: `--color`, `NO_COLOR` and the pipe decide whether the status word is painted.

mod common;

use serde_json::{Value, json};

use common::{Repo, blocker, fixed, open};

const WAITING: &str = "waiting   waits  github:o/r#1 is open\n";

#[test]
fn color_always_paints_the_status_word_and_nothing_else() {
    let repo = Repo::new(&(blocker("waits", 1) + &blocker("lands", 2)));
    repo.issue(1, 200, &open());
    repo.issue(2, 200, &fixed());
    repo.cmd()
        .args(["--color", "always"])
        .assert()
        .code(9)
        .stdout(
            "\x1b[2mwaiting\x1b[0m   waits  github:o/r#1 is open\n\
             \x1b[32mbumpable\x1b[0m  lands  github:o/r#2 closed as completed (drop the lands patch)\n",
        );
}

#[test]
fn color_never_stays_plain_even_when_the_environment_forces_color() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    repo.cmd()
        .env("CLICOLOR_FORCE", "1")
        .args(["--color", "never"])
        .assert()
        .code(0)
        .stdout(WAITING);
}

#[test]
fn color_auto_follows_the_environment_and_a_pipe_is_plain() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    repo.cmd()
        .env("CLICOLOR_FORCE", "1")
        .assert()
        .code(0)
        .stdout("\x1b[2mwaiting\x1b[0m   waits  github:o/r#1 is open\n");
    repo.cmd()
        .env("CLICOLOR_FORCE", "1")
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(WAITING);
    repo.cmd()
        .args(["--color", "auto"])
        .assert()
        .code(0)
        .stdout(WAITING);
}

#[test]
fn json_is_never_colored() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    let output = repo
        .cmd()
        .args(["--json", "--color", "always"])
        .output()
        .expect("runs");
    assert!(!output.stdout.contains(&0x1b), "no escape codes in JSON");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(report.pointer("/entries/0/status"), Some(&json!("waiting")));
}

#[test]
fn a_dead_line_names_the_workaround_like_a_bumpable_one() {
    let repo = Repo::new(&blocker("gone", 4));
    let not_planned = json!({"state": "closed", "state_reason": "not_planned"});
    repo.issue(4, 200, &not_planned);
    repo.cmd()
        .assert()
        .code(10)
        .stdout("dead      gone  github:o/r#4 closed as not planned (drop the gone patch)\n");
    repo.cmd()
        .args(["--color", "always"])
        .assert()
        .code(10)
        .stdout(
            "\x1b[35mdead\x1b[0m      gone  github:o/r#4 closed as not planned (drop the gone patch)\n",
        );
}

#[test]
fn a_global_flag_works_before_a_subcommand() {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    repo.cmd().assert().code(0);
    repo.cmd()
        .args(["--color", "never", "status"])
        .env("CLICOLOR_FORCE", "1")
        .assert()
        .code(0)
        .stdout(WAITING);
}
