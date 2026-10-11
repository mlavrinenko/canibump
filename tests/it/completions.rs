//! `canibump completions`: a script per shell, from an empty directory.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// The binary in an empty directory: no manifest to read.
fn bare(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary should build");
    cmd.current_dir(dir.path());
    cmd
}

#[test]
fn completions_print_a_bash_script_naming_subcommands_and_flags() {
    let dir = tempfile::tempdir().expect("tempdir");
    bare(&dir)
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("status")
                .and(predicate::str::contains("--refresh"))
                .and(predicate::str::contains("--color")),
        );
}

#[test]
fn completions_cover_every_shell_without_a_manifest() {
    let dir = tempfile::tempdir().expect("tempdir");
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        bare(&dir)
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("canibump").and(predicate::str::contains("status")))
            .stderr("");
    }
}

#[test]
fn completions_refuse_an_unknown_shell() {
    let dir = tempfile::tempdir().expect("tempdir");
    bare(&dir)
        .args(["completions", "tcsh"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value 'tcsh'"));
}
