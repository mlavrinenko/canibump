use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

#[tokio::test]
async fn maps_status_category_and_resolution() {
    let server = MockServer::start_async().await;
    let cases = [
        (1, "indeterminate", None, ItemState::Open),
        (2, "done", Some("Fixed"), ItemState::Completed),
        (3, "done", Some("Done"), ItemState::Completed),
        (4, "done", Some("Won't Fix"), ItemState::NotPlanned),
        (5, "done", Some("Cannot Reproduce"), ItemState::NotPlanned),
        (6, "done", Some("Duplicate"), ItemState::Duplicate),
        (7, "done", None, ItemState::ClosedNoReason),
        (8, "done", Some("Abandoned"), ItemState::NotPlanned),
        (9, "done", Some("Invalid"), ItemState::NotPlanned),
        (10, "done", Some("Not A Bug"), ItemState::NotPlanned),
        (11, "done", Some("Implemented"), ItemState::Completed),
        (12, "done", Some("Undone"), ItemState::ClosedNoReason),
    ];
    for (number, category, resolution, _) in &cases {
        let resolution = resolution.map(|name| json!({"name": name}));
        let body = json!({"fields": {
            "status": {"statusCategory": {"key": category}},
            "resolution": resolution,
        }});
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/rest/api/2/issue/CORE-{number}"))
                    .query_param("fields", "status,resolution")
                    .header("authorization", "Bearer t0ken");
                then.status(200).json_body(body);
            })
            .await;
    }
    let auth = HashMap::from([(
        "jira.example.org".to_owned(),
        JiraAuth::Bearer("t0ken".to_owned()),
    )]);
    let jira = Jira::new(Some(&server.base_url()), auth).expect("client");
    for (number, _, _, want) in cases {
        let item: ItemRef = format!("jira:jira.example.org#CORE-{number}")
            .parse()
            .expect("ref");
        assert_eq!(jira.item_state(&item).await, Ok(want), "CORE-{number}");
    }
    let other: ItemRef = "github:o/r#1".parse().expect("ref");
    assert!(matches!(
        jira.item_state(&other).await,
        Err(CheckError::Unsupported(_))
    ));
}

#[tokio::test]
async fn sends_basic_auth() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/rest/api/2/issue/CORE-1")
                .header("authorization", "Basic bWU6c2VjcmV0");
            then.status(200)
                .json_body(json!({"fields": {"status": {"statusCategory": {"key": "new"}}}}));
        })
        .await;
    let auth = HashMap::from([(
        "jira.example.org".to_owned(),
        JiraAuth::Basic("me".to_owned(), "secret".to_owned()),
    )]);
    let jira = Jira::new(Some(&server.base_url()), auth).expect("client");
    let item: ItemRef = "jira:jira.example.org#CORE-1".parse().expect("ref");
    assert_eq!(jira.item_state(&item).await, Ok(ItemState::Open));
}
