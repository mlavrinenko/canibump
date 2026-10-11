use super::*;
use crate::condition::{
    Bound, Condition, ReachedCondition, StateCondition, Step, VersionCondition, Within,
};
use crate::package::PackageRef;
use crate::refs::{ItemRef, Reachable, Source};

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
                kind: crate::refs::ItemKind::Issue,
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
fn reads_a_ttl_in_days_and_weeks() {
    for (text, hours) in [("1d", 24), ("2w", 336), ("1d12h", 36), ("1w 1d", 192)] {
        let manifest = Manifest::parse(&format!("[settings]\nttl = \"{text}\"\n"))
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(
            manifest.settings.ttl,
            SignedDuration::from_hours(hours),
            "{text}"
        );
    }
}

#[test]
fn a_ttl_in_months_names_the_units_it_takes() {
    let error = first_error("[settings]\nttl = \"1mo\"\n");
    assert!(error.contains("settings.ttl"), "{error}");
    assert!(error.contains("weeks"), "{error}");
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

fn error_line(text: &str) -> Option<usize> {
    Manifest::parse(text).expect_err("expected an error").line()
}

#[test]
fn reports_the_line_a_bad_blocker_starts_on() {
    let good = "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\n";
    let bad_key = format!("{good}\n[[blocker]]\nid = \"b\"\nnote = \"n\"\nstat = 1\n");
    assert_eq!(error_line(&bad_key), Some(6));
    let no_id = format!("{good}[[blocker]]\nnote = \"n\"\nstate = \"github:o/r#1\"\n");
    assert_eq!(error_line(&no_id), Some(5));
    assert_eq!(error_line(&format!("{good}{good}")), Some(5));
    let bad_step = format!(
        "{good}[[blocker]]\nid = \"b\"\nnote = \"n\"\n[[blocker.all]]\nstate = \"github:o/r#1\"\nextra = 1\n"
    );
    assert_eq!(error_line(&bad_step), Some(5));
}

#[test]
fn leaves_syntax_and_settings_errors_without_a_line() {
    assert_eq!(error_line("[[blocker]]\nid = x\n"), None);
    assert_eq!(error_line("[settings]\nttl = \"soon\"\n"), None);
}

#[test]
fn load_names_the_file_and_the_blocker_line_once() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deps.toml");
    std::fs::write(&path, "\n[[blocker]]\nid = \"a\"\nnote = \"n\"\nstat = 1\n").expect("write");
    let error = Manifest::load(&path).expect_err("bad blocker").to_string();
    assert_eq!(
        error,
        format!(
            "{}:2: blocker `a`: needs one condition: state, reached, version, all, any",
            path.display()
        ),
    );
}

#[test]
fn load_names_the_file_of_a_syntax_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deps.toml");
    std::fs::write(&path, "id = x\n").expect("write");
    let error = Manifest::load(&path).expect_err("bad toml").to_string();
    let prefix = format!("{}: TOML parse error at line 1", path.display());
    assert!(error.starts_with(&prefix), "{error}");
    assert_eq!(error.matches("TOML parse error").count(), 1, "{error}");
}

#[test]
fn load_names_the_missing_file() {
    let error = Manifest::load(Path::new("/nonexistent/canibump.toml"))
        .expect_err("missing file")
        .to_string();
    assert!(error.contains("/nonexistent/canibump.toml"), "{error}");
}

fn reached(within: &str) -> Result<Condition, ManifestError> {
    let text = format!(
        "[[blocker]]\nid = \"a\"\nnote = \"n\"\nreached = \"github:o/r#1\"\nin = {within}\n"
    );
    Manifest::parse(&text).map(|manifest| {
        manifest
            .blockers
            .into_iter()
            .next()
            .map(|blocker| blocker.condition)
            .expect("one blocker")
    })
}

#[test]
fn parses_reached_targets() {
    let item: Reachable = "github:o/r#1".parse().expect("valid");
    let expect = |within| {
        Condition::Reached(ReachedCondition {
            target: item.clone(),
            within,
        })
    };
    assert_eq!(
        reached("{ branch = \"main\" }").expect("branch"),
        expect(Within::Branch("main".to_owned()))
    );
    assert_eq!(
        reached("{ tag = \"v1\" }").expect("tag"),
        expect(Within::Tag("v1".to_owned()))
    );
    assert_eq!(
        reached("{ tags = \"v*\", since = \"0.5.0\" }").expect("tags"),
        expect(Within::Tags {
            pattern: glob::Pattern::new("v*").expect("glob"),
            since: Some("0.5.0".to_owned()),
        })
    );
}

#[test]
fn rejects_a_bad_reached_target() {
    for (within, message) in [
        ("{ }", "`in` needs one of branch, tag, tags"),
        (
            "{ branch = \"a\", tag = \"b\" }",
            "`in` needs one of branch, tag, tags",
        ),
        (
            "{ branch = \"a\", since = \"1\" }",
            "`since` goes with `tags`",
        ),
        ("{ tags = \"[\" }", "Pattern syntax error"),
        ("{ commit = \"a\" }", "commit"),
    ] {
        let error = reached(within).expect_err(within).to_string();
        assert!(error.contains(message), "{within}: {error}");
    }
    let error = first_error("[[blocker]]\nid = \"a\"\nnote = \"n\"\nreached = \"github:o/r#1\"\n");
    assert!(error.contains("in"), "{error}");
}

#[test]
fn rejects_two_conditions_in_one_blocker() {
    let error = first_error(
        "[[blocker]]\nid = \"a\"\nnote = \"n\"\nstate = \"github:o/r#1\"\nreached = \"github:o/r#1\"\nin = { tag = \"v1\" }\n",
    );
    assert!(
        error.contains("several conditions (state, reached)"),
        "{error}"
    );
}

#[test]
fn parses_a_version_condition() {
    let manifest = Manifest::parse(
        "[[blocker]]\nid = \"a\"\nnote = \"n\"\nversion = \"repology:llama.cpp\"\nrepo = \"nix_unstable\"\nsrcname = \"llama-cpp\"\nat-least = \"0.6.0\"\n",
    )
    .expect("valid");
    let condition = manifest
        .blockers
        .into_iter()
        .next()
        .map(|blocker| blocker.condition);
    assert_eq!(
        condition,
        Some(Condition::Version(VersionCondition {
            package: PackageRef::Repology {
                project: "llama.cpp".to_owned(),
                repo: "nix_unstable".to_owned(),
                srcname: Some("llama-cpp".to_owned()),
            },
            at_least: Bound::Version("0.6.0".to_owned()),
        }))
    );
    let error = first_error(
        "[[blocker]]\nid = \"a\"\nnote = \"n\"\nversion = \"repology:x\"\nat-least = \"1\"\n",
    );
    assert!(error.contains("needs `repo`"), "{error}");
    let error =
        first_error("[[blocker]]\nid = \"a\"\nnote = \"n\"\nversion = \"deps.dev:cargo/x\"\n");
    assert!(error.contains("at-least"), "{error}");
}

const CHAIN: &str = r#"
[[blocker]]
id = "llama"
note = "n"

[[blocker.all]]
id = "merged"
state = "github:o/r#1"

[[blocker.all]]
id = "released"
reached = "github:o/r#1"
in = { tags = "v*" }

[[blocker.all]]
version = "repology:x"
repo = "nix_unstable"
at-least = { step = "released" }
"#;

fn only_condition(text: &str) -> Condition {
    Manifest::parse(text)
        .expect("valid")
        .blockers
        .into_iter()
        .next()
        .map(|blocker| blocker.condition)
        .expect("one blocker")
}

#[test]
fn parses_an_all_of_chain_with_a_step_bound() {
    let Condition::All(steps) = only_condition(CHAIN) else {
        panic!("all-of expected");
    };
    let ids: Vec<Option<&str>> = steps.iter().map(|step| step.id.as_deref()).collect();
    assert_eq!(ids, [Some("merged"), Some("released"), None]);
    let Some(Step {
        condition: Condition::Version(version),
        ..
    }) = steps.get(2)
    else {
        panic!("a version step expected: {steps:?}");
    };
    assert_eq!(version.at_least, Bound::Step("released".to_owned()));
}

#[test]
fn parses_a_nested_any_of() {
    let text = r#"
[[blocker]]
id = "a"
note = "n"

[[blocker.any]]
state = "github:o/r#1"

[[blocker.any]]
[[blocker.any.all]]
state = "github:o/r#2"
[[blocker.any.all]]
state = "github:o/r#3"
"#;
    let Condition::Any(steps) = only_condition(text) else {
        panic!("any-of expected");
    };
    assert!(matches!(
        steps.get(1).map(|step| &step.condition),
        Some(Condition::All(inner)) if inner.len() == 2
    ));
}

#[test]
fn rejects_bad_composites() {
    let blocker = "[[blocker]]\nid = \"a\"\nnote = \"n\"\n";
    for (body, message) in [
        ("all = []\n", "needs at least one step"),
        ("all = [1]\n", "steps are tables"),
        (
            "state = \"github:o/r#1\"\n[[blocker.all]]\nstate = \"github:o/r#1\"\n",
            "several conditions",
        ),
        (
            "[[blocker.all]]\nid = \"x\"\nstate = \"github:o/r#1\"\n[[blocker.all]]\nid = \"x\"\nstate = \"github:o/r#1\"\n",
            "duplicate step id `x`",
        ),
        (
            "[[blocker.all]]\nversion = \"deps.dev:cargo/x\"\nat-least = { step = \"nope\" }\n",
            "no earlier step `nope`",
        ),
        (
            "[[blocker.all]]\nid = \"m\"\nstate = \"github:o/r#1\"\n[[blocker.all]]\nversion = \"deps.dev:cargo/x\"\nat-least = { step = \"m\" }\n",
            "step `m` finds no tag",
        ),
        (
            "version = \"deps.dev:cargo/x\"\nat-least = { step = \"r\" }\n",
            "no earlier step `r`",
        ),
        (
            "[[blocker.all]]\nstate = \"github:o/r#1\"\nextra = 1\n",
            "step 1",
        ),
    ] {
        let error = first_error(&format!("{blocker}{body}"));
        assert!(error.contains(message), "{body}: {error}");
    }
}

const RELEASED: &str = "id = \"released\"\nreached = \"github:o/r#1\"\nin = { tags = \"v*\" }\n";
const AT_LEAST_RELEASED: &str =
    "version = \"deps.dev:cargo/x\"\nat-least = { step = \"released\" }\n";
const WAITING: &str = "state = \"github:o/r#2\"\n";

/// One `[[path]]` table per body.
fn tables(path: &str, bodies: &[&str]) -> String {
    bodies
        .iter()
        .map(|body| format!("[[{path}]]\n{body}"))
        .collect()
}

fn with_blocker(steps: &str) -> String {
    format!("[[blocker]]\nid = \"a\"\nnote = \"n\"\n{steps}")
}

#[test]
fn refuses_a_step_reference_that_may_not_have_held() {
    let may_not_have_held = "`at-least` names step `released`, which sits in an `any` and may not have held; put both in one `all`";
    for steps in [
        // Both steps of one any: the first may not have held.
        tables("blocker.any", &[RELEASED, AT_LEAST_RELEASED]),
        // An earlier any inside an all: it may have resolved through another step.
        tables("blocker.all", &[""])
            + &tables("blocker.all.any", &[RELEASED, WAITING])
            + &tables("blocker.all", &[AT_LEAST_RELEASED]),
        // A sibling branch of an any.
        tables("blocker.any", &[""])
            + &tables("blocker.any.all", &[RELEASED])
            + &tables("blocker.any", &[""])
            + &tables("blocker.any.all", &[AT_LEAST_RELEASED]),
    ] {
        let error = first_error(&with_blocker(&steps));
        assert!(error.starts_with("blocker `a`: "), "{steps}: {error}");
        assert!(error.contains(may_not_have_held), "{steps}: {error}");
    }
}

#[test]
fn accepts_a_step_reference_that_surely_held() {
    for steps in [
        // The all-chain.
        tables("blocker.all", &[RELEASED, AT_LEAST_RELEASED]),
        // A nested all under an any.
        tables("blocker.any", &[WAITING, ""])
            + &tables("blocker.any.all", &[RELEASED, AT_LEAST_RELEASED]),
        // An any under an all sees what the all already ran.
        tables("blocker.all", &[RELEASED, ""])
            + &tables("blocker.all.any", &[AT_LEAST_RELEASED, WAITING]),
        // An earlier nested all hands its ids on.
        tables("blocker.all", &[""])
            + &tables("blocker.all.all", &[RELEASED])
            + &tables("blocker.all", &[AT_LEAST_RELEASED]),
    ] {
        let manifest = Manifest::parse(&with_blocker(&steps));
        assert!(manifest.is_ok(), "{steps}: {:?}", manifest.err());
    }
}

#[test]
fn keeps_step_ids_unique_across_the_whole_blocker() {
    for steps in [
        tables("blocker.any", &[RELEASED, RELEASED]),
        tables("blocker.any", &[RELEASED, ""]) + &tables("blocker.any.all", &[RELEASED]),
        tables("blocker.all", &[""])
            + &tables("blocker.all.any", &[RELEASED])
            + &tables("blocker.all", &[RELEASED]),
    ] {
        let error = first_error(&with_blocker(&steps));
        assert!(
            error.contains("duplicate step id `released`"),
            "{steps}: {error}"
        );
    }
}
