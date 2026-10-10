//! The crate's crates.io metadata, checked before a release finds out.

use std::path::PathBuf;

use toml::Value;

fn package_field(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(path).expect("Cargo.toml");
    let manifest: Value = toml::from_str(&text).expect("Cargo.toml is TOML");
    manifest
        .get("package")
        .and_then(|package| package.get(name))
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()))
}

/// What crates.io accepts: at most 5 keywords, each at most 20 characters,
/// starting with an ASCII letter or digit, then letters, digits, `_`, `-`, `+`.
fn accepted_by_crates_io(keyword: &str) -> bool {
    let mut chars = keyword.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric())
        && chars.all(|next| next.is_ascii_alphanumeric() || matches!(next, '_' | '-' | '+'))
        && keyword.len() <= 20
}

#[test]
fn the_crate_has_keywords_crates_io_accepts() {
    let keywords: Vec<String> = package_field("keywords")
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !keywords.is_empty(),
        "keywords = [] hides the crate from search"
    );
    assert!(
        keywords.len() <= 5,
        "crates.io takes at most five: {keywords:?}"
    );
    for keyword in &keywords {
        assert!(
            accepted_by_crates_io(keyword),
            "crates.io rejects {keyword:?}"
        );
        assert_eq!(
            *keyword,
            keyword.to_lowercase(),
            "{keyword:?} is not lowercase"
        );
    }
}
