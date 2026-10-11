//! `scripts/release-hook.sh` is the `pre-release-hook` of release.toml: every
//! refusal `just release` makes that cargo-release does not. Each test builds a
//! checkout with a bare origin, puts stub `gh`, `curl`, `just`, `cargo` and
//! `outdatty` first on PATH, and runs the hook as cargo-release does, through
//! the `NEW_VERSION`, `DRY_RUN` and `WORKSPACE_ROOT` variables.
//!
//! Unix only: the hook needs `bash` and `git`.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::common::shell::{commit, git, stderr, write};

const HOOK: &str = "scripts/release-hook.sh";
const CHANGELOG: &str = "# Changelog\n\n## [Unreleased]\n\n### Added\n\n- a thing\n";
const DATED: &str =
    "# Changelog\n\n## [Unreleased]\n\n## [0.2.0] - 2026-10-11\n\n### Added\n\n- a thing\n";

struct Release {
    dir: tempfile::TempDir,
    envs: Vec<(&'static str, &'static str)>,
}

impl Release {
    /// A checkout of `main` level with its origin, holding the hook, a
    /// manifest, a changelog with an entry and a `release.yml`.
    fn new() -> Self {
        let release = Self {
            dir: tempfile::tempdir().expect("tempdir"),
            envs: Vec::new(),
        };
        let root = release.dir.path();
        git(root, &["init", "-q", "--bare", "-b", "main", "origin"]);
        let work = release.work();
        std::fs::create_dir_all(&work).expect("mkdir");
        git(&work, &["init", "-q", "-b", "main"]);
        git(&work, &["config", "user.email", "t@example.com"]);
        git(&work, &["config", "user.name", "t"]);
        git(&work, &["config", "commit.gpgsign", "false"]);
        git(&work, &["remote", "add", "origin", "../origin"]);
        let hook = std::fs::read_to_string(checkout().join(HOOK)).expect("hook");
        write(&work.join(HOOK), &hook);
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nrepository = \"https://example.com/demo\"\n";
        write(&work.join("Cargo.toml"), manifest);
        write(&work.join("CHANGELOG.md"), CHANGELOG);
        write(
            &work.join(".github/workflows/release.yml"),
            "name: Release\n",
        );
        commit(&work, "base");
        git(&work, &["push", "-q", "origin", "main"]);
        release.stub("gh", "exit 0");
        release.stub("curl", "printf '%s' \"${CURL_STATUS:-404}\"");
        release.stub("just", "exit ${JUST_RC:-0}");
        release.stub("cargo", "exit ${CARGO_RC:-0}");
        release.stub(
            "outdatty",
            "echo \"outdatty $*\" >> \"$WORKSPACE_ROOT/../calls\"",
        );
        release
    }

    fn work(&self) -> PathBuf {
        self.dir.path().join("work")
    }

    fn stub(&self, name: &str, body: &str) {
        let path = self.dir.path().join("bin").join(name);
        write(&path, &format!("#!/usr/bin/env bash\n{body}\n"));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    fn with(mut self, key: &'static str, value: &'static str) -> Self {
        self.envs.push((key, value));
        self
    }

    /// Run the hook for 0.2.0, as cargo-release does: dry (`DRY_RUN=true`) or real.
    fn run(&self, dry: bool) -> Output {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new("bash")
            .arg(HOOK)
            .current_dir(self.work())
            .env("PATH", path)
            .env("NEW_VERSION", "0.2.0")
            .env("DRY_RUN", if dry { "true" } else { "false" })
            .env("WORKSPACE_ROOT", self.work())
            .envs(self.envs.iter().copied())
            .output()
            .expect("bash runs")
    }

    /// Run the hook and expect a refusal that prints `error` on its first line.
    fn refuses(&self, dry: bool, error: &str) -> String {
        let out = self.run(dry);
        let err = stderr(&out);
        assert!(!out.status.success(), "the hook passed:\n{err}");
        assert!(err.starts_with(&format!("error: {error}")), "got:\n{err}");
        err
    }
}

fn checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn a_clean_tree_with_changelog_entries_passes_a_dry_run() {
    let release = Release::new();
    let out = release.run(true);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn a_dirty_tree_is_refused() {
    let release = Release::new();
    write(&release.work().join("notes.txt"), "x");
    let err = release.refuses(true, "the working tree has uncommitted changes");
    assert!(err.contains("hint: commit or discard them, then re-run"));
}

#[test]
fn a_real_run_tolerates_the_bump_but_not_other_edits() {
    let release = Release::new();
    write(&release.work().join("CHANGELOG.md"), DATED);
    write(&release.work().join("Cargo.lock"), "# bumped\n");
    let out = release.run(false);
    assert!(out.status.success(), "{}", stderr(&out));
    write(&release.work().join("src.rs"), "x");
    let err = release.refuses(false, "the working tree has uncommitted changes");
    assert!(err.contains("git restore . drops the version bump"));
}

#[test]
fn a_branch_other_than_main_is_refused() {
    let release = Release::new();
    git(&release.work(), &["switch", "-q", "-c", "topic"]);
    release.refuses(true, "HEAD is on 'topic', not main");
}

#[test]
fn an_origin_main_that_main_lacks_commits_of_is_refused() {
    let release = Release::new();
    let work = release.work();
    write(&work.join("more.txt"), "x");
    commit(&work, "more");
    git(&work, &["push", "-q", "origin", "main"]);
    git(&work, &["reset", "-q", "--hard", "HEAD~1"]);
    let err = release.refuses(true, "origin/main has commits that main lacks");
    assert!(err.contains("hint: git pull --ff-only origin main, then re-run"));
}

#[test]
fn an_origin_without_main_asks_for_a_first_push_by_hand() {
    let release = Release::new();
    let origin = release.dir.path().join("origin");
    git(&origin, &["update-ref", "-d", "refs/heads/main"]);
    let err = release.refuses(true, "origin has no main yet");
    assert!(err.contains("hint: push it once by hand: git push -u origin main"));
}

#[test]
fn an_existing_tag_is_refused_local_or_pushed() {
    let release = Release::new();
    git(&release.work(), &["tag", "v0.2.0"]);
    let err = release.refuses(true, "'v0.2.0' already exists");
    assert!(err.contains("gh workflow run release.yml -f tag=v0.2.0"));
    git(&release.work(), &["push", "-q", "origin", "v0.2.0"]);
    git(&release.work(), &["tag", "-d", "v0.2.0"]);
    release.refuses(true, "'v0.2.0' already exists");
}

#[test]
fn a_version_already_on_crates_io_is_refused() {
    let release = Release::new().with("CURL_STATUS", "200");
    let err = release.refuses(true, "'demo 0.2.0' is already on crates.io");
    assert!(err.contains("a published version is never re-published"));
}

#[test]
fn an_unreachable_crates_io_is_refused() {
    let release = Release::new().with("CURL_STATUS", "503");
    release.refuses(true, "crates.io answered HTTP '503' for 'demo 0.2.0'");
}

#[test]
fn a_changelog_without_entries_is_refused() {
    let release = Release::new();
    let work = release.work();
    write(
        &work.join("CHANGELOG.md"),
        "# Changelog\n\n## [Unreleased]\n\n",
    );
    commit(&work, "empty changelog");
    let err = release.refuses(
        true,
        "CHANGELOG.md has nothing under [Unreleased] to release",
    );
    assert!(err.contains("hint: add the release's entries under '## [Unreleased]'"));
}

#[test]
fn a_real_run_needs_the_dated_section_the_replacement_makes() {
    let release = Release::new();
    let err = release.refuses(false, "CHANGELOG.md has no dated section for '0.2.0'");
    assert!(err.contains("hint: add the release's entries under '## [Unreleased]'"));
}

#[test]
fn failing_gates_are_refused() {
    let release = Release::new().with("JUST_RC", "1");
    let err = release.refuses(true, "the gates have not passed on this tree");
    assert!(err.contains("hint: fix what just check reports above"));
}

#[test]
fn a_failing_publish_dry_run_is_refused() {
    let release = Release::new().with("CARGO_RC", "1");
    let err = release.refuses(true, "cargo publish --dry-run failed");
    assert!(err.contains("hint: fix the packaging error above"));
}

#[test]
fn a_real_run_confirms_the_release_notes_and_a_dry_run_leaves_them() {
    let release = Release::new();
    release.run(true);
    let calls = release.dir.path().join("calls");
    assert!(!calls.exists(), "a dry run wrote the drift lock");
    write(&release.work().join("CHANGELOG.md"), DATED);
    let out = release.run(false);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = std::fs::read_to_string(&calls).expect("outdatty was called");
    assert_eq!(text, "outdatty update --group release-notes\n");
}

#[test]
fn a_dry_run_expects_no_edit_at_all() {
    let release = Release::new();
    write(&release.work().join("Cargo.lock"), "# edited\n");
    release.refuses(true, "the working tree has uncommitted changes");
}
