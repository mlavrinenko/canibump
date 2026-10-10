use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

#[tokio::test]
async fn maps_status_names_and_is_closed() {
    let server = MockServer::start_async().await;
    let cases = [
        (
            1,
            json!({"name": "New", "is_closed": false}),
            ItemState::Open,
        ),
        (
            2,
            json!({"name": "Closed", "is_closed": true}),
            ItemState::Completed,
        ),
        (3, json!({"name": "Resolved"}), ItemState::Completed),
        (
            4,
            json!({"name": "Rejected", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            5,
            json!({"name": "Won't fix", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            6,
            json!({"name": "Duplicate", "is_closed": true}),
            ItemState::Duplicate,
        ),
        (
            7,
            json!({"name": "Archived", "is_closed": true}),
            ItemState::Completed,
        ),
        (
            8,
            json!({"name": "Abandoned", "is_closed": true}),
            ItemState::NotPlanned,
        ),
        (
            9,
            json!({"name": "Undone", "is_closed": false}),
            ItemState::Open,
        ),
    ];
    for (number, status, _) in &cases {
        let body = json!({"issue": {"status": status}});
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/issues/{number}.json"))
                    .header("x-redmine-api-key", "k3y");
                then.status(200).json_body(body);
            })
            .await;
    }
    let keys = HashMap::from([("redmine.example.org".to_owned(), "k3y".to_owned())]);
    let redmine = Redmine::new(Some(&server.base_url()), keys).expect("client");
    for (number, _, want) in cases {
        let item: ItemRef = format!("redmine:redmine.example.org#{number}")
            .parse()
            .expect("ref");
        assert_eq!(redmine.item_state(&item).await, Ok(want), "#{number}");
    }
    let other: ItemRef = "github:o/r#1".parse().expect("ref");
    assert!(matches!(
        redmine.item_state(&other).await,
        Err(CheckError::Unsupported(_))
    ));
}
