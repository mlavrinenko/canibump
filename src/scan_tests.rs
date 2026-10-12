use std::path::Path;

use tempfile::TempDir;

use super::*;

/// A temp repo holding `files` and a `canibump.toml` of `manifest`: the
/// unlisted links its scan finds, as `file:line ref (catcher)`.
fn found(manifest: &str, files: &[(&str, &str)]) -> Vec<String> {
    let dir = TempDir::new().expect("tempdir");
    for (name, text) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }
    let manifest_path = dir.path().join("canibump.toml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");
    let manifest = Manifest::parse(manifest).expect("valid manifest");
    unlisted(&manifest, &manifest_path)
        .into_iter()
        .map(|hit| {
            format!(
                "{}:{} {} ({})",
                hit.file.display(),
                hit.line,
                hit.reference,
                hit.catcher
            )
        })
        .collect()
}

fn first_error(text: &str) -> String {
    match Manifest::parse(text) {
        Ok(manifest) => panic!("expected an error, parsed {manifest:?}"),
        Err(error) => error.to_string(),
    }
}

const NIX: &str = "[scan]\npaths = [\"**/*.nix\"]\n";

const PIN: &str =
    "{\n  # pin until https://github.com/NixOS/nixpkgs/issues/412345 is fixed\n  x = 1;\n}\n";

#[test]
fn reports_a_github_issue_link_no_blocker_lists_with_its_file_and_line() {
    assert_eq!(
        found(NIX, &[("hosts/gpu.nix", PIN)]),
        ["hosts/gpu.nix:2 github:NixOS/nixpkgs#412345 (github)"]
    );
}

#[test]
fn a_manifest_without_scan_reads_no_file() {
    assert_eq!(found("", &[("hosts/gpu.nix", PIN)]), Vec::<String>::new());
}

#[test]
fn a_link_some_blocker_names_is_listed_even_nested_in_a_chain() {
    let manifest = format!(
        "{NIX}[[blocker]]\nid = \"a\"\nnote = \"n\"\n\
         [[blocker.all]]\nstate = \"github:o/r#1\"\n\
         [[blocker.all]]\n[[blocker.all.any]]\nreached = \"github:nixos/NIXPKGS#412345\"\nin = {{ branch = \"master\" }}\n"
    );
    assert_eq!(
        found(&manifest, &[("a.nix", PIN)]),
        Vec::<String>::new(),
        "refs compare without case, as forges do"
    );
}

#[test]
fn catches_pull_requests_merge_requests_and_codeberg_links() {
    let text = "\
https://github.com/o/r/pull/7\n\
see https://gitlab.com/g/sub/p/-/merge_requests/8 and https://gitlab.com/g/p/-/issues/9\n\
http://codeberg.org/o/r/pulls/10, https://codeberg.org/o/r/issues/11\n";
    assert_eq!(
        found(NIX, &[("a.nix", text)]),
        [
            "a.nix:1 github:o/r#7 (github)",
            "a.nix:2 gitlab:g/sub/p!8 (gitlab-mr)",
            "a.nix:2 gitlab:g/p#9 (gitlab-issue)",
            "a.nix:3 codeberg:o/r#10 (codeberg)",
            "a.nix:3 codeberg:o/r#11 (codeberg)",
        ]
    );
}

#[test]
fn skips_a_line_marked_ignore_excluded_files_and_the_manifest() {
    let manifest = "[scan]\npaths = [\"**/*\"]\nexclude = [\"tasks/**\"]\n\
                    # https://github.com/o/r/issues/1\n";
    let files = [
        (
            "a.nix",
            "# https://github.com/o/r/issues/2 canibump:ignore\n",
        ),
        ("tasks/t.typ", "https://github.com/o/r/issues/3\n"),
        ("b.nix", "https://github.com/o/r/issues/4\n"),
    ];
    assert_eq!(found(manifest, &files), ["b.nix:1 github:o/r#4 (github)"]);
}

#[test]
fn reports_each_line_a_link_sits_on_once() {
    let text = "https://github.com/o/r/issues/1\n\n\
                https://github.com/o/r/issues/1 https://github.com/o/r/issues/1\n";
    assert_eq!(
        found(NIX, &[("a.nix", text)]),
        [
            "a.nix:1 github:o/r#1 (github)",
            "a.nix:3 github:o/r#1 (github)"
        ]
    );
}

#[test]
fn a_file_that_is_not_utf8_is_skipped() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("bin.nix"), [0xff_u8, 0xfe, b'\n']).expect("write");
    std::fs::write(dir.path().join("a.nix"), PIN).expect("write");
    let manifest_path = dir.path().join("canibump.toml");
    let manifest = Manifest::parse(NIX).expect("valid");
    let hits = unlisted(&manifest, &manifest_path);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits.first().map(|hit| hit.file.clone()),
        Some(Path::new("a.nix").to_owned())
    );
}

#[test]
fn a_custom_catcher_fills_its_ref_from_named_groups() {
    let manifest = format!(
        "{NIX}[[scan.catcher]]\nid = \"jira\"\n\
         pattern = 'https://jira\\.example\\.com/browse/(?<key>[A-Z]+-\\d+)'\n\
         ref = \"jira:jira.example.com#${{key}}\"\n"
    );
    assert_eq!(
        found(
            &manifest,
            &[("a.nix", "# https://jira.example.com/browse/CORE-42\n")]
        ),
        ["a.nix:1 jira:jira.example.com#CORE-42 (jira)"]
    );
}

#[test]
fn a_custom_ref_is_written_the_way_the_manifest_prints_it() {
    let manifest = format!(
        "{NIX}[[scan.catcher]]\nid = \"self-hosted\"\n\
         pattern = 'gl/(?<n>\\d+)'\nref = \"gitlab:gitlab.com/g/p#${{n}}\"\n\
         [[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"gitlab:g/p#5\"\n"
    );
    assert_eq!(
        found(&manifest, &[("a.nix", "gl/5 gl/6\n")]),
        ["a.nix:1 gitlab:g/p#6 (self-hosted)"]
    );
}

#[test]
fn a_built_in_turns_off_or_gives_way_to_a_catcher_of_its_id() {
    let off = format!("{NIX}[[scan.catcher]]\nid = \"github\"\nenabled = false\n");
    assert_eq!(found(&off, &[("a.nix", PIN)]), Vec::<String>::new());
    let replaced = format!(
        "{NIX}[[scan.catcher]]\nid = \"github\"\npattern = 'gh#(?<n>\\d+)'\nref = \"github:o/r#${{n}}\"\n"
    );
    assert_eq!(
        found(
            &replaced,
            &[("a.nix", "gh#3\nhttps://github.com/o/r/issues/4\n")]
        ),
        ["a.nix:1 github:o/r#3 (github)"]
    );
}

#[test]
fn prints_one_unlisted_line_per_hit() {
    let hits = [Hit {
        file: "hosts/gpu.nix".into(),
        line: 41,
        reference: "github:NixOS/nixpkgs#412345".to_owned(),
        catcher: "github".to_owned(),
    }];
    let plain = anstream::adapter::strip_str(&lines(&hits)).to_string();
    assert_eq!(
        plain,
        "unlisted  hosts/gpu.nix:41  github:NixOS/nixpkgs#412345 has no blocker\n"
    );
}

#[test]
fn refuses_a_scan_with_no_paths_or_a_bad_glob() {
    assert!(first_error("[scan]\n").contains("missing field `paths`"));
    assert_eq!(
        first_error("[scan]\npaths = []\n"),
        "scan.paths: list at least one glob of files to read"
    );
    assert!(first_error("[scan]\npaths = [\"[\"]\n").starts_with("scan.paths: `[` is not a glob"));
    assert!(
        first_error("[scan]\npaths = [\"*\"]\nexclude = [\"a[\"]\n")
            .starts_with("scan.exclude: `a[` is not a glob")
    );
}

#[test]
fn refuses_a_catcher_that_cannot_work() {
    let catcher = |body: &str| first_error(&format!("{NIX}[[scan.catcher]]\n{body}"));
    assert_eq!(
        catcher("id = \"mine\"\n"),
        "scan.catcher `mine`: not a built-in, so give it a `pattern` and a `ref`"
    );
    assert_eq!(
        catcher("id = \"mine\"\npattern = 'x'\n"),
        "scan.catcher `mine`: a `pattern` needs a `ref`"
    );
    assert_eq!(
        catcher("id = \"github\"\nref = \"x\"\n"),
        "scan.catcher `github`: a `ref` needs a `pattern`"
    );
    assert_eq!(
        catcher("id = \"mine\"\npattern = '(\\d+)'\nref = \"github:o/r#${2}\"\n"),
        "scan.catcher `mine`: `ref` names `${2}`, which `pattern` has no group for"
    );
    assert!(
        catcher("id = \"mine\"\npattern = '('\nref = \"x\"\n")
            .starts_with("scan.catcher `mine`: bad `pattern`:")
    );
    assert_eq!(
        catcher("id = \"mine\"\npattern = '(?<n>\\d+)'\nref = \"github:o/r#${num}\"\n"),
        "scan.catcher `mine`: `ref` names `${num}`, which `pattern` has no group for"
    );
    assert_eq!(
        catcher(
            "id = \"a\"\npattern = 'x'\nref = \"x\"\n[[scan.catcher]]\nid = \"a\"\nenabled = false\n"
        ),
        "scan.catcher `a`: listed twice"
    );
}

#[test]
fn a_ref_may_name_a_numbered_group() {
    let manifest = format!(
        "{NIX}[[scan.catcher]]\nid = \"n\"\npattern = 'gh#(\\d+)'\nref = \"github:o/r#${{1}}\"\n"
    );
    assert_eq!(
        found(&manifest, &[("a.nix", "gh#2\n")]),
        ["a.nix:1 github:o/r#2 (n)"]
    );
}

#[test]
fn reads_a_linked_file_but_never_enters_a_linked_directory_or_git() {
    let dir = TempDir::new().expect("tempdir");
    let outside = TempDir::new().expect("tempdir");
    std::fs::write(outside.path().join("store.nix"), PIN).expect("write");
    std::os::unix::fs::symlink(outside.path(), dir.path().join("result")).expect("link dir");
    std::os::unix::fs::symlink(
        outside.path().join("store.nix"),
        dir.path().join("linked.nix"),
    )
    .expect("link file");
    std::fs::create_dir(dir.path().join(".git")).expect("mkdir");
    std::fs::write(dir.path().join(".git/hook.nix"), PIN).expect("write");
    let manifest_path = dir.path().join("canibump.toml");
    let manifest = Manifest::parse(NIX).expect("valid");
    let files: Vec<PathBuf> = unlisted(&manifest, &manifest_path)
        .into_iter()
        .map(|hit| hit.file)
        .collect();
    assert_eq!(files, [PathBuf::from("linked.nix")]);
}

#[test]
fn an_exclude_ending_in_globstar_prunes_its_directory() {
    let scan =
        Manifest::parse("[scan]\npaths = [\"**/*\"]\nexclude = [\"var/**\", \"tasks/*.typ\"]\n")
            .expect("valid")
            .scan
            .expect("scan");
    assert!(scan.prunes(Path::new("var")));
    assert!(!scan.prunes(Path::new("variable")));
    assert!(
        !scan.prunes(Path::new("tasks")),
        "tasks/sub/x.typ is not excluded"
    );
}
