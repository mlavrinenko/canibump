//! The JSON Schemas for `canibump.toml`, `--json` and the state file:
//! generated from the types, committed, and checked against real files.

use std::path::PathBuf;

use assert_cmd::Command;
use jsonschema::Validator;
use serde_json::Value;

use crate::common::waiting_json_run;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn committed() -> String {
    std::fs::read_to_string(root().join("schema/canibump.schema.json"))
        .expect("schema/canibump.schema.json is committed")
}

fn validator() -> Validator {
    let schema: Value = serde_json::from_str(&committed()).expect("the schema is JSON");
    jsonschema::validator_for(&schema).expect("the schema is a valid JSON Schema")
}

fn manifest(text: &str) -> Value {
    toml::from_str(text).expect("the manifest is TOML")
}

#[test]
fn committed_schema_matches_the_generated_one() {
    assert!(
        committed() == canibump::schema::render(),
        "schema/canibump.schema.json is stale: run `just schema`"
    );
}

#[test]
fn schema_subcommand_prints_the_committed_file_without_a_manifest() {
    let dir = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .expect("binary should build")
        .current_dir(dir.path())
        .arg("schema")
        .assert()
        .success()
        .stdout(committed());
}

#[test]
fn the_nix_flake_manifest_validates() {
    let text =
        std::fs::read_to_string(root().join("tests/fixtures/nix-canibump.toml")).expect("fixture");
    let errors: Vec<String> = validator()
        .iter_errors(&manifest(&text))
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn this_repos_own_manifest_validates_and_loads() {
    let path = root().join("canibump.toml");
    let text = std::fs::read_to_string(&path).expect("canibump.toml is committed");
    assert!(
        text.starts_with("#:schema ./schema/canibump.schema.json\n"),
        "the first line wires the committed schema in"
    );
    let errors: Vec<String> = validator()
        .iter_errors(&manifest(&text))
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let loaded = canibump::manifest::Manifest::load(&path).expect("the manifest loads");
    assert!(
        !loaded.blockers.is_empty(),
        "it tracks at least one blocker"
    );
}

#[test]
fn every_condition_shape_validates() {
    let text = r#"
[settings]
ttl = "30m"
cache = "state.json"

[[blocker]]
id = "a"
note = "n"
state = "github:o/r#1"

[[blocker]]
id = "b"
note = "n"
reached = "github:o/r#1"
in = { tags = "v*", since = "1.0.0" }

[[blocker]]
id = "c"
note = "n"
version = "deps.dev:cargo/serde"
at-least = "2.0.0"

[[blocker]]
id = "d"
note = "n"

[[blocker.any]]
state = "github:o/r#1"

[[blocker.any]]
id = "nested"

[[blocker.any.all]]
state = "github:o/r#2"
"#;
    assert!(validator().is_valid(&manifest(text)));
}

#[test]
fn a_misspelled_key_is_rejected() {
    let bad = [
        "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstat = \"github:o/r#1\"\n",
        "[settngs]\nttl = \"6h\"\n",
        "[settings]\nttl = 6\n",
        "[[blocker]]\nid = \"a\"\nstate = \"github:o/r#1\"\n",
    ];
    for text in bad {
        assert!(!validator().is_valid(&manifest(text)), "{text}");
    }
}

#[test]
fn two_conditions_in_one_blocker_are_rejected() {
    let text = "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\nreached = \"github:o/r#1\"\nin = { branch = \"main\" }\n";
    assert!(!validator().is_valid(&manifest(text)));
}

fn committed_at(name: &str) -> String {
    std::fs::read_to_string(root().join("schema").join(name)).expect("the schema is committed")
}

fn validator_of(name: &str) -> Validator {
    let schema: Value = serde_json::from_str(&committed_at(name)).expect("the schema is JSON");
    jsonschema::validator_for(&schema).expect("the schema is a valid JSON Schema")
}

fn errors(validator: &Validator, instance: &Value) -> Vec<String> {
    validator
        .iter_errors(instance)
        .map(|error| error.to_string())
        .collect()
}

#[test]
fn committed_report_and_state_schemas_match_the_generated_ones() {
    assert!(
        committed_at("report.schema.json") == canibump::schema::render_report(),
        "schema/report.schema.json is stale: run `just schema`"
    );
    assert!(
        committed_at("state.schema.json") == canibump::schema::render_state(),
        "schema/state.schema.json is stale: run `just schema`"
    );
}

#[test]
fn schema_subcommand_prints_each_schema_by_name() {
    for (what, file) in [
        ("manifest", "canibump.schema.json"),
        ("report", "report.schema.json"),
        ("state", "state.schema.json"),
    ] {
        let dir = tempfile::tempdir().expect("tempdir");
        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("binary should build")
            .current_dir(dir.path())
            .args(["schema", what])
            .assert()
            .success()
            .stdout(committed_at(file));
    }
}

#[test]
fn a_real_report_and_its_state_file_validate() {
    let (repo, report) = waiting_json_run();
    assert_eq!(
        errors(&validator_of("report.schema.json"), &report),
        Vec::<String>::new()
    );
    let state: Value =
        serde_json::from_str(&std::fs::read_to_string(repo.cache_path()).expect("state file"))
            .expect("json");
    assert_eq!(
        errors(&validator_of("state.schema.json"), &state),
        Vec::<String>::new()
    );
}
