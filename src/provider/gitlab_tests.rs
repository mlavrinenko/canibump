use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

fn gitlab(server: &MockServer) -> GitLab {
    let tokens = HashMap::from([(server.address().to_string(), "t0ken".to_owned())]);
    GitLab::new(Some(&server.base_url()), tokens).expect("client")
}

async fn mock(server: &MockServer, path: &str, body: serde_json::Value) {
    server
        .mock_async(|when, then| {
            when.method(GET).path(path).header("private-token", "t0ken");
            then.status(200).json_body(body);
        })
        .await;
}

async fn state(server: &MockServer, text: &str) -> Result<ItemState, CheckError> {
    let item: ItemRef = text.parse().expect("valid ref");
    gitlab(server).item_state(&item).await
}

#[tokio::test]
async fn reads_merge_request_states() {
    let server = MockServer::start_async().await;
    let base = "/api/v4/projects/group%2Fproj/merge_requests";
    mock(&server, &format!("{base}/1"), json!({"state": "opened"})).await;
    mock(&server, &format!("{base}/2"), json!({"state": "merged"})).await;
    mock(&server, &format!("{base}/3"), json!({"state": "closed"})).await;
    mock(&server, &format!("{base}/4"), json!({"state": "locked"})).await;
    assert_eq!(
        state(&server, "gitlab:group/proj!1").await,
        Ok(ItemState::Open)
    );
    assert_eq!(
        state(&server, "gitlab:group/proj!2").await,
        Ok(ItemState::Merged)
    );
    assert_eq!(
        state(&server, "gitlab:group/proj!3").await,
        Ok(ItemState::ClosedUnmerged)
    );
    assert_eq!(
        state(&server, "gitlab:group/proj!4").await,
        Ok(ItemState::Open)
    );
}

#[tokio::test]
async fn a_closed_issue_is_completed_only_when_an_mr_closed_it() {
    let server = MockServer::start_async().await;
    let base = "/api/v4/projects/group%2Fproj/issues";
    mock(&server, &format!("{base}/1"), json!({"state": "opened"})).await;
    mock(&server, &format!("{base}/2"), json!({"state": "closed"})).await;
    mock(
        &server,
        &format!("{base}/2/closed_by"),
        json!([{"iid": 7, "state": "merged"}]),
    )
    .await;
    mock(&server, &format!("{base}/3"), json!({"state": "closed"})).await;
    mock(&server, &format!("{base}/3/closed_by"), json!([])).await;
    assert_eq!(
        state(&server, "gitlab:group/proj#1").await,
        Ok(ItemState::Open)
    );
    assert_eq!(
        state(&server, "gitlab:group/proj#2").await,
        Ok(ItemState::Completed)
    );
    assert_eq!(
        state(&server, "gitlab:group/proj#3").await,
        Ok(ItemState::ClosedNoReason)
    );
}

#[tokio::test]
async fn a_closed_issue_counts_only_when_a_closing_mr_merged() {
    let server = MockServer::start_async().await;
    let base = "/api/v4/projects/group%2Fproj/issues";
    let cases = [
        (1, json!([{"iid": 7, "state": "opened"}])),
        (2, json!([{"iid": 7, "state": "closed"}])),
        (
            3,
            json!([{"iid": 7, "state": "closed"}, {"iid": 8, "state": "merged"}]),
        ),
    ];
    for (number, closers) in cases {
        let issue = json!({"state": "closed"});
        mock(&server, &format!("{base}/{number}"), issue).await;
        mock(&server, &format!("{base}/{number}/closed_by"), closers).await;
    }
    assert_eq!(
        state(&server, "gitlab:group/proj#1").await,
        Ok(ItemState::ClosedNoReason),
        "open MR"
    );
    assert_eq!(
        state(&server, "gitlab:group/proj#2").await,
        Ok(ItemState::ClosedNoReason),
        "closed unmerged MR"
    );
    assert_eq!(
        state(&server, "gitlab:group/proj#3").await,
        Ok(ItemState::Completed),
        "one merged MR among others"
    );
}

#[tokio::test]
async fn sends_a_token_only_to_its_own_host() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v4/projects/group%2Fproj/issues/1")
                .header_missing("private-token");
            then.status(200).json_body(json!({"state": "opened"}));
        })
        .await;
    let tokens = HashMap::from([("other.example.org".to_owned(), "t0ken".to_owned())]);
    let gitlab = GitLab::new(Some(&server.base_url()), tokens).expect("client");
    let item: ItemRef = "gitlab:other.example.org/group/proj#1"
        .parse()
        .expect("valid ref");
    assert_eq!(gitlab.item_state(&item).await, Ok(ItemState::Open));
}

#[tokio::test]
async fn refuses_other_sources() {
    let server = MockServer::start_async().await;
    let result = state(&server, "github:o/r#1").await;
    assert!(
        matches!(result, Err(CheckError::Unsupported(_))),
        "{result:?}"
    );
    assert!(GitLab::new(Some("not a url"), HashMap::new()).is_err());
}
