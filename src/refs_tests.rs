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

#[test]
fn parses_a_github_commit_or_item_as_reachable() {
    let commit: Reachable = "github:o/r@6ee6c33".parse().expect("valid");
    assert_eq!(
        commit,
        Reachable::Commit(CommitRef {
            source: Source::GitHub {
                owner: "o".to_owned(),
                repo: "r".to_owned(),
            },
            sha: "6ee6c33".to_owned(),
        })
    );
    assert_eq!(commit.to_string(), "github:o/r@6ee6c33");
    let item: Reachable = "github:o/r#5".parse().expect("valid");
    assert_eq!(item.to_string(), "github:o/r#5");
    assert_eq!(item.source().kind(), SourceKind::GitHub);
}

#[test]
fn rejects_a_short_or_non_hex_sha() {
    for text in ["github:o/r@abc", "github:o/r@zzzzzzz", "github:o/r"] {
        let error = text.parse::<Reachable>().expect_err(text).to_string();
        assert!(error.contains(text), "{error}");
    }
}

#[test]
fn parses_a_git_remote_commit() {
    let commit: Reachable = "git:https://example.org/a/b.git@abc1234"
        .parse()
        .expect("valid");
    assert_eq!(
        commit,
        Reachable::Commit(CommitRef {
            source: Source::Git {
                url: "https://example.org/a/b.git".to_owned(),
            },
            sha: "abc1234".to_owned(),
        })
    );
    assert_eq!(
        commit.to_string(),
        "git:https://example.org/a/b.git@abc1234"
    );
    assert_eq!(commit.source().kind(), SourceKind::Git);
    let error = "git:https://example.org/a/b.git#1"
        .parse::<Reachable>()
        .expect_err("no items")
        .to_string();
    assert!(
        error.contains("git:https://example.org/a/b.git#1"),
        "{error}"
    );
}

#[test]
fn parses_package_refs() {
    let nix = PackageRef::parse("repology:llama.cpp", Some("nix_unstable".to_owned()), None)
        .expect("valid");
    assert_eq!(
        nix,
        PackageRef::Repology {
            project: "llama.cpp".to_owned(),
            repo: "nix_unstable".to_owned(),
            srcname: None,
        }
    );
    assert_eq!(nix.to_string(), "repology:llama.cpp in nix_unstable");
    assert_eq!(nix.index(), IndexKind::Repology);
    let crate_ref = PackageRef::parse("deps.dev:cargo/serde", None, None).expect("valid");
    assert_eq!(
        crate_ref,
        PackageRef::DepsDev {
            system: "cargo".to_owned(),
            name: "serde".to_owned(),
        }
    );
    assert_eq!(crate_ref.to_string(), "deps.dev:cargo/serde");
    assert_eq!(crate_ref.index(), IndexKind::DepsDev);
    let go = PackageRef::parse("deps.dev:go/github.com/a/b", None, None).expect("valid");
    assert_eq!(go.to_string(), "deps.dev:go/github.com/a/b");
}

#[test]
fn rejects_bad_package_refs() {
    for (text, repo, srcname, message) in [
        ("repology:llama.cpp", None, None, "needs `repo`"),
        ("deps.dev:cargo/serde", Some("x"), None, "takes no `repo`"),
        (
            "deps.dev:cargo/serde",
            None,
            Some("x"),
            "takes no `srcname`",
        ),
        ("deps.dev:serde", None, None, "deps.dev:<system>/<name>"),
        ("deps.dev:cobol/x", None, None, "cobol"),
        ("repology:", Some("r"), None, "repology:<project>"),
        ("pypi:x", None, None, "unknown scheme"),
    ] {
        let error = PackageRef::parse(text, repo.map(str::to_owned), srcname.map(str::to_owned))
            .expect_err(text)
            .to_string();
        assert!(error.contains(message), "{text}: {error}");
    }
}
