use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

#[tokio::test]
async fn lists_versions_with_the_name_encoded() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/v3/systems/npm/packages/@types%2Fnode");
            then.status(200).json_body(json!({"versions": [
                {"versionKey": {"system": "NPM", "name": "@types/node", "version": "20.1.0"}},
                {"versionKey": {"system": "NPM", "name": "@types/node", "version": "22.0.0"}},
            ]}));
        })
        .await;
    let depsdev = DepsDev::new(&server.base_url()).expect("client");
    let package = PackageRef::parse("deps.dev:npm/@types/node", None, None).expect("valid");
    assert_eq!(
        depsdev.versions(&package).await,
        Ok(vec!["20.1.0".to_owned(), "22.0.0".to_owned()])
    );
}

#[tokio::test]
async fn maps_http_errors() {
    let server = MockServer::start_async().await;
    for (name, status) in [
        ("gone", 404),
        ("slow", 429),
        ("locked", 403),
        ("broken", 500),
    ] {
        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/v3/systems/cargo/packages/{name}"));
                then.status(status).body("{}");
            })
            .await;
    }
    let depsdev = DepsDev::new(&server.base_url()).expect("client");
    let versions = |name: &str| {
        let package =
            PackageRef::parse(&format!("deps.dev:cargo/{name}"), None, None).expect("valid");
        let depsdev = depsdev.clone();
        async move { depsdev.versions(&package).await }
    };
    assert_eq!(
        versions("gone").await,
        Err(CheckError::NotFound("deps.dev:cargo/gone".to_owned()))
    );
    assert_eq!(
        versions("slow").await,
        Err(CheckError::RateLimited("deps.dev:cargo/slow".to_owned()))
    );
    assert_eq!(
        versions("locked").await,
        Err(CheckError::Auth("deps.dev:cargo/locked".to_owned()))
    );
    assert_eq!(
        versions("broken").await,
        Err(CheckError::Network(
            "deps.dev:cargo/broken: HTTP 500".to_owned()
        ))
    );
    let unreachable = DepsDev::new("http://127.0.0.1:9").expect("client");
    let package = PackageRef::parse("deps.dev:cargo/x", None, None).expect("valid");
    assert!(matches!(
        unreachable.versions(&package).await,
        Err(CheckError::Network(_))
    ));
}

#[tokio::test]
async fn rejects_a_bad_base_and_a_repology_package() {
    assert!(DepsDev::new("not a url").is_err());
    let depsdev = DepsDev::new("http://127.0.0.1:9").expect("client");
    let package = PackageRef::parse("repology:x", Some("r".to_owned()), None).expect("valid");
    assert!(matches!(
        depsdev.versions(&package).await,
        Err(CheckError::Unsupported(_))
    ));
}
