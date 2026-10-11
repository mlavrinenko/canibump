//! Shared by the end-to-end suites: a temp repo, a mock upstream and the binary.
#![allow(dead_code, reason = "each suite uses a subset")]

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use serde_json::{Value, json};
use tempfile::TempDir;

pub mod shell;

pub struct Repo {
    pub dir: TempDir,
    pub server: MockServer,
}

impl Repo {
    pub fn new(manifest: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("canibump.toml"), manifest).expect("write manifest");
        Self {
            dir,
            server: MockServer::start(),
        }
    }

    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary should build");
        cmd.current_dir(self.dir.path())
            .env("CANIBUMP_GITHUB_API", self.server.base_url())
            .env("CANIBUMP_REPOLOGY_API", self.server.base_url())
            .env("CANIBUMP_DEPSDEV_API", self.server.base_url())
            .env("GH_TOKEN", "t0ken")
            .env_remove("GITHUB_TOKEN")
            .env_remove("CLICOLOR_FORCE")
            .env("XDG_CACHE_HOME", self.dir.path().join("cache"));
        cmd
    }

    /// Like [`Repo::cmd`], but with no token in the environment and a `gh`
    /// that logs each call and prints one: the log path says who asked.
    pub fn cmd_with_gh(&self) -> (Command, PathBuf) {
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

    pub fn issue(&self, number: u64, status: u16, body: &Value) -> Mock<'_> {
        self.server.mock(|when, then| {
            when.method(GET).path(format!("/repos/o/r/issues/{number}"));
            then.status(status).json_body(body.clone());
        })
    }

    pub fn cache_path(&self) -> PathBuf {
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
pub fn gh_calls(log: &Path) -> usize {
    std::fs::read_to_string(log).map_or(0, |text| text.lines().count())
}

pub fn blocker(id: &str, number: u64) -> String {
    format!(
        "[[blocker]]\nid = \"{id}\"\nnote = \"drop the {id} patch\"\nstate = \"github:o/r#{number}\"\n"
    )
}

pub fn open() -> Value {
    json!({"state": "open"})
}

pub fn fixed() -> Value {
    json!({"state": "closed", "state_reason": "completed"})
}

/// A repo holding a `waiting` blocker and one blocker per status in `kinds`
/// (`bumpable`, `dead` or `failed`, which is also its id), under `settings`.
pub fn repo_of(kinds: &[&str], settings: &str) -> Repo {
    let mut manifest = settings.to_owned() + &blocker("waits", 1);
    for (index, kind) in (2..).zip(kinds) {
        manifest += &blocker(kind, index);
    }
    let repo = Repo::new(&manifest);
    repo.issue(1, 200, &open());
    for (index, kind) in (2..).zip(kinds) {
        match *kind {
            "bumpable" => repo.issue(index, 200, &fixed()),
            "dead" => repo.issue(
                index,
                200,
                &json!({"state": "closed", "state_reason": "not_planned"}),
            ),
            _ => repo.issue(index, 404, &json!({"message": "Not Found"})),
        };
    }
    repo
}

/// A repo whose one blocker, `waits`, is open upstream, and its `--json` run.
pub fn waiting_json_run() -> (Repo, Value) {
    let repo = Repo::new(&blocker("waits", 1));
    repo.issue(1, 200, &open());
    let output = repo.cmd().arg("--json").output().expect("runs");
    let report = serde_json::from_slice(&output.stdout).expect("json");
    (repo, report)
}
