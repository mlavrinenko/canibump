use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::{Value, json};

use super::*;

/// Serves each status as issue `#1`, `#2`, ... and checks the state read.
async fn assert_states(cases: Vec<(Value, ItemState)>) {
    let server = MockServer::start_async().await;
    for (index, (status, _)) in cases.iter().enumerate() {
        let body = json!({"issue": {"status": status}});
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/issues/{}.json", index + 1))
                    .header("x-redmine-api-key", "k3y");
                then.status(200).json_body(body);
            })
            .await;
    }
    let keys = HashMap::from([(server.address().to_string(), "k3y".to_owned())]);
    let redmine = Redmine::new(Some(&server.base_url()), keys).expect("client");
    for (index, (status, want)) in cases.into_iter().enumerate() {
        let item: ItemRef = format!("redmine:redmine.example.org#{}", index + 1)
            .parse()
            .expect("ref");
        assert_eq!(redmine.item_state(&item).await, Ok(want), "{status}");
    }
}

#[tokio::test]
async fn maps_status_names_and_is_closed() {
    assert_states(vec![
        (json!({"name": "New", "is_closed": false}), ItemState::Open),
        (
            json!({"name": "Closed", "is_closed": true}),
            ItemState::Completed,
        ),
        (json!({"name": "Resolved"}), ItemState::Completed),
        (
            json!({"name": "Rejected", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            json!({"name": "Won't fix", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            json!({"name": "Duplicate", "is_closed": true}),
            ItemState::Duplicate,
        ),
        (
            json!({"name": "Archived", "is_closed": true}),
            ItemState::Completed,
        ),
        (
            json!({"name": "Abandoned", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            json!({"name": "Undone", "is_closed": false}),
            ItemState::Open,
        ),
        (json!({"name": "Rejected"}), ItemState::NotPlanned),
        (json!({"name": "Duplicate"}), ItemState::Duplicate),
        (json!({"name": "In progress"}), ItemState::Open),
    ])
    .await;
}

#[tokio::test]
async fn is_closed_false_keeps_a_closing_name_open() {
    assert_states(vec![
        (
            json!({"name": "Resolved", "is_closed": false}),
            ItemState::Open,
        ),
        (
            json!({"name": "Fixed", "is_closed": false}),
            ItemState::Open,
        ),
        (
            json!({"name": "Rejected", "is_closed": false}),
            ItemState::Open,
        ),
    ])
    .await;
}

#[tokio::test]
async fn a_negated_name_never_reads_completed() {
    assert_states(vec![
        (
            json!({"name": "Not done", "is_closed": true}),
            ItemState::ClosedNoReason,
        ),
        (
            json!({"name": "Not fixed", "is_closed": true}),
            ItemState::ClosedNoReason,
        ),
        (
            json!({"name": "Won't do", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (json!({"name": "Not done"}), ItemState::Open),
        (json!({"name": "Won't do"}), ItemState::NotPlanned),
    ])
    .await;
}

#[tokio::test]
async fn refuses_other_sources() {
    let redmine = Redmine::new(None, HashMap::new()).expect("client");
    let other: ItemRef = "github:o/r#1".parse().expect("ref");
    assert!(matches!(
        redmine.item_state(&other).await,
        Err(CheckError::Unsupported(_))
    ));
}
