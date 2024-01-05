use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use tower::ServiceExt;

use crate::test::{prepare_server, prepare_server_with_user};

#[tokio::test]
async fn test_getting_nonexistent_profile() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/User")
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
    assert!(content.contains("no such user"));
}

#[tokio::test]
async fn test_getting_profile() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/Test")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Test's profile"));
}
