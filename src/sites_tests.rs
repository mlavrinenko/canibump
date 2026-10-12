use tempfile::TempDir;

use crate::manifest::Manifest;
use crate::scan::find;

const PIN: &str = "{\n  \"dependencies\": {\n    \"left-pad\": \"=1.3.0\"\n  }\n}\n";

const LEFT_PAD: &str = r#"[{ file = "package.json", pattern = '"left-pad": "=1\.3\.0"' }]"#;

const SCAN: &str = "[scan]\npaths = [\"*.nix\"]\n";

/// Blocker `a` declaring `sites`.
fn blocker(sites: &str) -> String {
    format!("[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\nsites = {sites}\n")
}

/// What the scan of a temp repo holding `files` and a manifest of `text`
/// finds: each site as `id file:line`, each unmarked blocker as `id: detail`.
fn found(text: &str, files: &[(&str, &str)]) -> (Vec<String>, Vec<String>) {
    let dir = TempDir::new().expect("tempdir");
    for (name, body) in files {
        std::fs::write(dir.path().join(name), body).expect("write");
    }
    let manifest_path = dir.path().join("canibump.toml");
    let findings = find(&Manifest::parse(text).expect("valid"), &manifest_path);
    let sites = findings
        .sites
        .iter()
        .flat_map(|(id, sites)| {
            sites
                .iter()
                .map(move |site| format!("{id} {}:{}", site.file.display(), site.line))
        })
        .collect();
    let unmarked = findings
        .unmarked
        .iter()
        .map(|one| format!("{}: {}", one.id, one.detail))
        .collect();
    (sites, unmarked)
}

fn first_error(text: &str) -> String {
    Manifest::parse(text).expect_err("invalid").to_string()
}

#[test]
fn a_declared_site_is_each_line_its_pattern_matches_with_or_without_scan() {
    let files = [("package.json", PIN)];
    let expected = (vec!["a package.json:3".to_owned()], Vec::<String>::new());
    assert_eq!(found(&blocker(LEFT_PAD), &files), expected);
    assert_eq!(
        found(&format!("{SCAN}{}", blocker(LEFT_PAD)), &files),
        expected,
        "a blocker that declares sites needs no marker"
    );
}

#[test]
fn a_declared_site_that_matches_nothing_or_is_missing_is_unmarked() {
    let gone = PIN.replace("left-pad", "right-pad");
    assert_eq!(
        found(&blocker(LEFT_PAD), &[("package.json", &gone)]).1,
        [r#"a: no line of package.json matches `"left-pad": "=1\.3\.0"`"#]
    );
    assert_eq!(
        found(&blocker(LEFT_PAD), &[]).1,
        ["a: package.json is missing"]
    );
}

#[test]
fn markers_and_declared_sites_list_together_by_file_and_line() {
    let files = [("package.json", PIN), ("a.nix", "x\n# canibump:a\n")];
    assert_eq!(
        found(&format!("{SCAN}{}", blocker(LEFT_PAD)), &files).0,
        ["a a.nix:2", "a package.json:3"]
    );
}

#[test]
fn refuses_sites_that_cannot_work_naming_the_blocker() {
    let error = first_error(&blocker(r#"[{ file = "p", pattern = '(' }]"#));
    assert!(
        error.starts_with("blocker `a`: sites: bad `pattern` `(`:"),
        "{error}"
    );
    let error = first_error(&blocker(r#"[{ file = "p", pattern = "x", line = 3 }]"#));
    assert!(
        error.starts_with("blocker `a`: sites:") && error.contains("unknown field `line`"),
        "{error}"
    );
    let error = first_error(&blocker(r#""p""#));
    assert!(error.starts_with("blocker `a`: sites:"), "{error}");
}

#[test]
fn sites_leave_the_cached_condition_alone() {
    let spec = |text: &str| {
        Manifest::parse(text)
            .expect("valid")
            .blockers
            .first()
            .map(|one| one.spec.clone())
    };
    let bare = "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\n";
    assert_eq!(spec(&blocker(LEFT_PAD)), spec(bare));
}

#[test]
fn a_declared_file_that_cannot_be_read_is_unmarked_with_the_reason() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("package.json"), [0xff_u8, b'\n']).expect("write");
    let manifest = Manifest::parse(&blocker(LEFT_PAD)).expect("valid");
    let findings = find(&manifest, &dir.path().join("canibump.toml"));
    let details: Vec<&str> = findings
        .unmarked
        .iter()
        .map(|one| one.detail.as_str())
        .collect();
    assert_eq!(
        details,
        ["cannot read package.json: stream did not contain valid UTF-8"]
    );
}
