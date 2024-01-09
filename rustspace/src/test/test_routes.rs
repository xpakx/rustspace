use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use tower::ServiceExt;
use serial_test::serial;

use crate::test::prepare_server;

#[tokio::test]
#[serial]
async fn test_index() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Welcome!"));
    assert!(content.contains("Homepage"));
}

#[tokio::test]
#[serial]
async fn test_about() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/about")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("About us"));
}

#[tokio::test]
#[serial]
async fn test_help() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/help")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Help"));
}
