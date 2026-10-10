use super::*;
use crate::refs::{ItemRef, Source};

fn first_error(text: &str) -> String {
    match Manifest::parse(text) {
        Ok(manifest) => panic!("expected an error, parsed {manifest:?}"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn parses_a_state_blocker() {
    let manifest = Manifest::parse(
        r#"
        [[blocker]]
        id = "serde-pin"
        note = "Cargo.toml pins serde until the fix lands"
        state = "github:serde-rs/serde#2590"
        "#,
    )
    .expect("valid manifest");
    let [blocker] = manifest.blockers.as_slice() else {
        panic!("one blocker expected, got {:?}", manifest.blockers);
    };
    assert_eq!(blocker.id, "serde-pin");
    assert_eq!(blocker.note, "Cargo.toml pins serde until the fix lands");
    assert_eq!(
        blocker.condition,
        Condition::State(StateCondition {
            item: ItemRef {
                source: Source::GitHub {
                    owner: "serde-rs".to_owned(),
                    repo: "serde".to_owned(),
                },
                number: 2590,
            },
        })
    );
}

#[test]
fn defaults_the_ttl_to_six_hours() {
    let manifest = Manifest::parse("").expect("empty manifest is valid");
    assert_eq!(manifest.settings.ttl, SignedDuration::from_hours(6));
    assert!(manifest.settings.cache.is_none());
    assert!(manifest.blockers.is_empty());
}

#[test]
fn reads_ttl_and_cache_settings() {
    let manifest = Manifest::parse(
        r#"
        [settings]
        ttl = "30m"
        cache = "state.json"
        "#,
    )
    .expect("valid settings");
    assert_eq!(manifest.settings.ttl, SignedDuration::from_mins(30));
    assert_eq!(manifest.settings.cache, Some(PathBuf::from("state.json")));
}

#[test]
fn rejects_a_bad_ttl() {
    let error = first_error("[settings]\nttl = \"soon\"\n");
    assert!(error.contains("settings.ttl"), "{error}");
}

#[test]
fn rejects_a_blocker_without_a_condition() {
    let error = first_error("[[blocker]]\nid = \"a\"\nnote = \"n\"\n");
    assert!(error.contains("blocker `a`"), "{error}");
    assert!(error.contains("needs one condition"), "{error}");
}

#[test]
fn rejects_an_unknown_key_naming_the_blocker() {
    let error =
        first_error("[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\nstat = 1\n");
    assert!(error.contains("blocker `a`"), "{error}");
    assert!(error.contains("stat"), "{error}");
}

#[test]
fn rejects_a_missing_id_and_note() {
    let error = first_error("[[blocker]]\nnote = \"n\"\nstate = \"github:o/r#1\"\n");
    assert!(error.contains("blocker #1"), "{error}");
    assert!(error.contains("`id`"), "{error}");
    let error = first_error("[[blocker]]\nid = \"a\"\nstate = \"github:o/r#1\"\n");
    assert!(error.contains("`note`"), "{error}");
}

#[test]
fn rejects_a_duplicate_id() {
    let one = "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\n";
    let error = first_error(&format!("{one}{one}"));
    assert!(error.contains("duplicate blocker id `a`"), "{error}");
}

#[test]
fn rejects_a_malformed_reference_naming_the_blocker() {
    let error = first_error("[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o#1\"\n");
    assert!(error.contains("blocker `a`"), "{error}");
    assert!(error.contains("github:o#1"), "{error}");
}

#[test]
fn keeps_a_spec_that_changes_with_the_condition() {
    let parse = |state: &str| {
        let text = format!("[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"{state}\"\n");
        Manifest::parse(&text)
            .expect("valid")
            .blockers
            .into_iter()
            .map(|blocker| blocker.spec)
            .collect::<Vec<_>>()
    };
    assert_eq!(parse("github:o/r#1"), parse("github:o/r#1"));
    assert_ne!(parse("github:o/r#1"), parse("github:o/r#2"));
}

#[test]
fn load_names_the_missing_file() {
    let error = Manifest::load(Path::new("/nonexistent/canibump.toml"))
        .expect_err("missing file")
        .to_string();
    assert!(error.contains("/nonexistent/canibump.toml"), "{error}");
}
