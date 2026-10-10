use super::*;

#[test]
fn parses_a_github_item() {
    let item: ItemRef = "github:ggml-org/llama.cpp#26487".parse().expect("valid");
    assert_eq!(
        item,
        ItemRef {
            source: Source::GitHub {
                owner: "ggml-org".to_owned(),
                repo: "llama.cpp".to_owned(),
            },
            number: 26487,
        }
    );
    assert_eq!(item.to_string(), "github:ggml-org/llama.cpp#26487");
    assert_eq!(item.source.kind(), SourceKind::GitHub);
}

#[test]
fn rejects_malformed_items() {
    for text in [
        "ggml-org/llama.cpp#1",
        "github:llama.cpp#1",
        "github:o/r",
        "github:o/r#x",
        "github:o/r/x#1",
        "github:/r#1",
    ] {
        let error = text.parse::<ItemRef>().expect_err(text).to_string();
        assert!(error.contains(text), "{text}: {error}");
    }
}

#[test]
fn names_the_known_schemes_for_an_unknown_one() {
    let error = "svn:o/r#1"
        .parse::<ItemRef>()
        .expect_err("unknown")
        .to_string();
    assert!(error.contains("svn"), "{error}");
    assert!(error.contains("github"), "{error}");
}
