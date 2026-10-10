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
            kind: ItemKind::Issue,
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

fn gitlab(host: &str, project: &str) -> Source {
    Source::GitLab {
        host: host.to_owned(),
        project: project.to_owned(),
    }
}

#[test]
fn parses_gitlab_issues_and_merge_requests() {
    let cases = [
        (
            "gitlab:group/sub/proj!12",
            gitlab("gitlab.com", "group/sub/proj"),
            12,
            ItemKind::MergeRequest,
        ),
        (
            "gitlab:gitlab.example.com/group/proj#3",
            gitlab("gitlab.example.com", "group/proj"),
            3,
            ItemKind::Issue,
        ),
        (
            "gitlab:my.group/proj#1",
            gitlab("gitlab.com", "my.group/proj"),
            1,
            ItemKind::Issue,
        ),
    ];
    for (text, source, number, kind) in cases {
        let item: ItemRef = text.parse().expect(text);
        assert_eq!(
            item,
            ItemRef {
                source,
                number,
                kind
            },
            "{text}"
        );
        assert_eq!(item.to_string(), text);
        assert_eq!(item.source.kind(), SourceKind::GitLab);
    }
}

#[test]
fn rejects_a_merge_request_off_gitlab_and_a_bare_project() {
    for text in ["github:o/r!1", "gitlab:proj#1", "gitlab:group/proj!x"] {
        let error = text.parse::<ItemRef>().expect_err(text).to_string();
        assert!(error.contains(text), "{error}");
    }
}
