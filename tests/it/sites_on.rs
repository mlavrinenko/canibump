//! End-to-end: which lines print their sites. A check prints them under
//! bumpable and dead lines unless `sites-on` or `--sites-on` says otherwise;
//! `status` and `status <id>` print them under every line.

use crate::common::{Repo, repo_of};

const SCAN: &str = "[scan]\npaths = [\"*.nix\"]\n\n";

const WAITS: &str = "waiting   waits     github:o/r#1 is open\n";

const BUMPABLE: &str =
    "bumpable  bumpable  github:o/r#2 closed as completed (drop the bumpable patch)\n";

const WAITS_AT: &str = "          at pin.nix:1\n";

const BUMPABLE_AT: &str = "          at pin.nix:2\n";

/// A waiting and a bumpable blocker under `settings`, each marked in
/// `pin.nix`, checked once.
fn marked(settings: &str) -> Repo {
    let repo = repo_of(&["bumpable"], &(settings.to_owned() + SCAN));
    std::fs::write(
        repo.dir.path().join("pin.nix"),
        "# canibump:waits\n# canibump:bumpable\n",
    )
    .expect("write");
    repo.cmd().assert().code(9);
    repo
}

#[test]
fn a_check_prints_sites_under_bumpable_and_dead_lines_by_default() {
    marked("")
        .cmd()
        .assert()
        .code(9)
        .stdout(format!("{WAITS}{BUMPABLE}{BUMPABLE_AT}"));
}

#[test]
fn status_prints_every_lines_sites() {
    marked("")
        .cmd()
        .arg("status")
        .assert()
        .code(9)
        .stdout(format!("{WAITS}{WAITS_AT}{BUMPABLE}{BUMPABLE_AT}"));
}

#[test]
fn status_of_one_blocker_prints_its_sites() {
    marked("")
        .cmd()
        .args(["status", "waits"])
        .assert()
        .code(1)
        .stdout(format!("waiting   waits  github:o/r#1 is open\n{WAITS_AT}"));
}

#[test]
fn sites_on_picks_the_statuses_and_the_flag_beats_it() {
    let repo = marked("[settings]\nsites-on = [\"waiting\"]\n\n");
    repo.cmd()
        .assert()
        .stdout(format!("{WAITS}{WAITS_AT}{BUMPABLE}"));
    repo.cmd()
        .args(["--sites-on", "bumpable"])
        .assert()
        .stdout(format!("{WAITS}{BUMPABLE}{BUMPABLE_AT}"));
    marked("")
        .cmd()
        .args(["status", "--sites-on", "waiting,dead"])
        .assert()
        .stdout(format!("{WAITS}{WAITS_AT}{BUMPABLE}"));
}
