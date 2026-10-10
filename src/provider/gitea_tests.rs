use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

async fn state(server: &MockServer, text: &str) -> Result<ItemState, CheckError> {
    let tokens = HashMap::from([("codeberg.org".to_owned(), "t0ken".to_owned())]);
    let gitea = Gitea::new(Some(&server.base_url()), tokens).expect("client");
    let item: ItemRef = text.parse().expect("valid ref");
    gitea.item_state(&item).await
}

#[tokio::test]
async fn reads_issue_and_pull_states() {
    let server = MockServer::start_async().await;
    for (number, body) in [
        (1, json!({"state": "open", "pull_request": null})),
        (2, json!({"state": "closed"})),
        (
            3,
            json!({"state": "open", "pull_request": {"merged": false}}),
        ),
        (
            4,
            json!({"state": "closed", "pull_request": {"merged": true}}),
        ),
        (
            5,
            json!({"state": "closed", "pull_request": {"merged": false}}),
        ),
    ] {
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/api/v1/repos/o/r/issues/{number}"))
                    .header("authorization", "token t0ken");
                then.status(200).json_body(body);
            })
            .await;
    }
    let expected = [
        ItemState::Open,
        ItemState::ClosedNoReason,
        ItemState::Open,
        ItemState::Merged,
        ItemState::ClosedUnmerged,
    ];
    for (number, want) in (1..).zip(expected) {
        assert_eq!(
            state(&server, &format!("codeberg:o/r#{number}")).await,
            Ok(want),
            "#{number}"
        );
    }
}

#[tokio::test]
async fn sends_no_token_to_another_host_and_refuses_other_sources() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/v1/repos/o/r/issues/1")
                .header_missing("authorization");
            then.status(200).json_body(json!({"state": "open"}));
        })
        .await;
    assert_eq!(
        state(&server, "gitea:git.example.org/o/r#1").await,
        Ok(ItemState::Open)
    );
    let result = state(&server, "github:o/r#1").await;
    assert!(
        matches!(result, Err(CheckError::Unsupported(_))),
        "{result:?}"
    );
}
