//! `reached` targets the manifest refuses before any network call.

use predicates::prelude::*;

use crate::common::Repo;

#[test]
fn a_gitlab_mr_in_reached_is_a_manifest_error_with_its_line() {
    let repo = Repo::new(
        "[[blocker]]\nid = \"b\"\nnote = \"n\"\nreached = \"gitlab:g/p!12\"\nin = { tag = \"v1\" }\n",
    );
    repo.cmd()
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains(
            "canibump.toml:1: blocker `b`: `reached` takes github:owner/repo#N, github:owner/repo@<sha> or git:<url>@<sha>, not `gitlab:g/p!12`",
        ));
}
