use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use tower::ServiceExt;

use crate::test::prepare_server;

#[tokio::test]
async fn test_getting_email_form() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/email")
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
    assert!(content.contains("<form"));
    assert!(content.contains("email"));
}

#[tokio::test]
async fn test_getting_password_form() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/password")
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
    assert!(content.contains("<form"));
    assert!(content.contains("password"));
}
