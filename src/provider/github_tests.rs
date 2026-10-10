use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use serde_json::json;

use super::*;
use crate::provider::{CodeHost, GitRef};
use crate::refs::Source;

/// A client holding the token the `issue` mocks insist on.
fn authed(server: &MockServer) -> GitHub {
    GitHub::new(Some(&server.base_url()), Some("t0ken".to_owned())).expect("client")
}

async fn state_of(server: &MockServer, number: u64) -> Result<ItemState, CheckError> {
    let item: ItemRef = format!("github:o/r#{number}").parse().expect("valid ref");
    authed(server).item_state(&item).await
}

const MERGED_AT: &str = "2025-11-02T10:30:00Z";

async fn issue(server: &MockServer, number: u64, body: serde_json::Value) -> Mock<'_> {
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/repos/o/r/issues/{number}"))
                .header("authorization", "Bearer t0ken");
            then.status(200).json_body(body);
        })
        .await
}

/// What `/issues/N` answers for a pull request: the PR rides along as
/// `pull_request`, and its `merged_at` is set once it merged.
async fn pull<'a>(
    server: &'a MockServer,
    number: u64,
    state: &str,
    merged_at: Option<&str>,
) -> Mock<'a> {
    let url = format!("https://api.github.com/repos/o/r/pulls/{number}");
    let body = json!({
        "state": state,
        "pull_request": {"url": url, "merged_at": merged_at},
    });
    issue(server, number, body).await
}

/// What `/pulls/N` answers: the merge commit lives only here.
async fn pull_detail(server: &MockServer, number: u64, body: serde_json::Value) -> Mock<'_> {
    server
        .mock_async(|when, then| {
            when.method(GET).path(format!("/repos/o/r/pulls/{number}"));
            then.status(200).json_body(body);
        })
        .await
}

#[tokio::test]
async fn reads_issue_states() {
    let server = MockServer::start_async().await;
    issue(&server, 1, json!({"state": "open", "state_reason": null})).await;
    issue(
        &server,
        2,
        json!({"state": "closed", "state_reason": "completed"}),
    )
    .await;
    issue(
        &server,
        3,
        json!({"state": "closed", "state_reason": "not_planned"}),
    )
    .await;
    issue(
        &server,
        4,
        json!({"state": "closed", "state_reason": "duplicate"}),
    )
    .await;
    issue(&server, 5, json!({"state": "closed", "state_reason": null})).await;
    issue(
        &server,
        6,
        json!({"state": "closed", "state_reason": "reopened"}),
    )
    .await;
    let expected = [
        ItemState::Open,
        ItemState::Completed,
        ItemState::NotPlanned,
        ItemState::Duplicate,
        ItemState::ClosedNoReason,
        ItemState::ClosedNoReason,
    ];
    for (number, state) in (1..).zip(expected) {
        assert_eq!(state_of(&server, number).await, Ok(state), "issue {number}");
    }
}

#[tokio::test]
async fn reads_pull_request_states() {
    let server = MockServer::start_async().await;
    pull(&server, 7, "open", None).await;
    pull(&server, 8, "closed", Some(MERGED_AT)).await;
    pull(&server, 9, "closed", None).await;
    assert_eq!(state_of(&server, 7).await, Ok(ItemState::Open));
    assert_eq!(state_of(&server, 8).await, Ok(ItemState::Merged));
    assert_eq!(state_of(&server, 9).await, Ok(ItemState::ClosedUnmerged));
}

#[tokio::test]
async fn a_pull_requests_state_costs_one_call() {
    let server = MockServer::start_async().await;
    let issues = pull(&server, 7, "closed", Some(MERGED_AT)).await;
    let pulls = pull_detail(&server, 7, json!({"merged": true})).await;
    assert_eq!(state_of(&server, 7).await, Ok(ItemState::Merged));
    assert_eq!(issues.calls_async().await, 1);
    assert_eq!(
        pulls.calls_async().await,
        0,
        "the issue already says it merged"
    );
}

#[tokio::test]
async fn a_pull_requests_state_and_merge_commit_cost_two_calls() {
    let server = MockServer::start_async().await;
    let issues = pull(&server, 7, "closed", Some(MERGED_AT)).await;
    let body = json!({"merged": true, "merge_commit_sha": "abc1234"});
    let pulls = pull_detail(&server, 7, body).await;
    let github = authed(&server);
    let item: ItemRef = "github:o/r#7".parse().expect("valid ref");
    assert_eq!(github.item_state(&item).await, Ok(ItemState::Merged));
    assert_eq!(
        github.merge_commit(&item).await,
        Ok(Some("abc1234".to_owned()))
    );
    assert_eq!(issues.calls_async().await, 1);
    assert_eq!(pulls.calls_async().await, 1);
}

#[tokio::test]
async fn maps_http_errors() {
    let server = MockServer::start_async().await;
    for (number, status, message) in [
        (10, 404, "Not Found"),
        (11, 401, "Bad credentials"),
        (12, 403, "API rate limit exceeded for 1.2.3.4."),
        (13, 403, "Resource not accessible by integration"),
        (14, 429, "Too many requests"),
        (15, 422, "Validation Failed"),
    ] {
        server
            .mock_async(|when, then| {
                when.method(GET).path(format!("/repos/o/r/issues/{number}"));
                then.status(status)
                    .json_body(json!({"message": message, "documentation_url": "x"}));
            })
            .await;
    }
    let item = |number| format!("github:o/r#{number}");
    assert_eq!(
        state_of(&server, 10).await,
        Err(CheckError::NotFound(item(10)))
    );
    assert_eq!(state_of(&server, 11).await, Err(CheckError::Auth(item(11))));
    assert_eq!(
        state_of(&server, 12).await,
        Err(CheckError::RateLimited(item(12)))
    );
    assert_eq!(state_of(&server, 13).await, Err(CheckError::Auth(item(13))));
    assert_eq!(
        state_of(&server, 14).await,
        Err(CheckError::RateLimited(item(14)))
    );
    let Err(CheckError::Network(message)) = state_of(&server, 15).await else {
        panic!("422 should be a network error");
    };
    assert!(message.contains("Validation Failed"), "{message}");
}

#[tokio::test]
async fn an_unreachable_api_is_a_network_error() {
    let github = GitHub::new(Some("http://127.0.0.1:9"), None).expect("client");
    let item: ItemRef = "github:o/r#1".parse().expect("valid ref");
    let result = github.item_state(&item).await;
    assert!(matches!(result, Err(CheckError::Network(_))), "{result:?}");
}

#[test]
fn rejects_a_bad_base_url() {
    assert!(GitHub::new(Some("not a url"), None).is_err());
}

fn github(server: &MockServer) -> GitHub {
    GitHub::new(Some(&server.base_url()), None).expect("client")
}

fn source() -> Source {
    Source::GitHub {
        owner: "o".to_owned(),
        repo: "r".to_owned(),
    }
}

#[tokio::test]
async fn reads_the_merge_commit_of_a_merged_pull() {
    let server = MockServer::start_async().await;
    for (number, body) in [
        (
            1,
            json!({"state": "closed", "merged": true, "merge_commit_sha": "abc1234"}),
        ),
        (
            2,
            json!({"state": "open", "merged": false, "merge_commit_sha": "test-merge"}),
        ),
    ] {
        server
            .mock_async(|when, then| {
                when.method(GET).path(format!("/repos/o/r/pulls/{number}"));
                then.status(200).json_body(body);
            })
            .await;
    }
    let item = |number: u64| -> ItemRef { format!("github:o/r#{number}").parse().expect("ref") };
    let github = github(&server);
    assert_eq!(
        github.merge_commit(&item(1)).await,
        Ok(Some("abc1234".to_owned()))
    );
    assert_eq!(github.merge_commit(&item(2)).await, Ok(None));
}

#[tokio::test]
async fn compares_a_commit_against_a_ref() {
    let server = MockServer::start_async().await;
    for (head, status) in [
        ("main", "ahead"),
        ("v1", "identical"),
        ("old", "behind"),
        ("fork", "diverged"),
    ] {
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/repos/o/r/compare/abc1234...{head}"));
                then.status(200).json_body(json!({"status": status}));
            })
            .await;
    }
    let github = github(&server);
    let contains = |target: GitRef| {
        let github = github.clone();
        async move { github.contains(&source(), "abc1234", &target).await }
    };
    assert_eq!(contains(GitRef::Branch("main".to_owned())).await, Ok(true));
    assert_eq!(contains(GitRef::Tag("v1".to_owned())).await, Ok(true));
    assert_eq!(contains(GitRef::Tag("old".to_owned())).await, Ok(false));
    assert_eq!(contains(GitRef::Branch("fork".to_owned())).await, Ok(false));
    let missing = contains(GitRef::Tag("nope".to_owned())).await;
    assert!(
        matches!(missing, Err(CheckError::NotFound(_))),
        "{missing:?}"
    );
}

#[tokio::test]
async fn lists_tags_by_prefix() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/repos/o/r/git/matching-refs/tags/v");
            then.status(200).json_body(json!([
                {"ref": "refs/tags/v0.1.0"},
                {"ref": "refs/tags/v0.2.0"},
            ]));
        })
        .await;
    let tags = github(&server).tags(&source(), "v").await;
    assert_eq!(tags, Ok(vec!["v0.1.0".to_owned(), "v0.2.0".to_owned()]));
}
