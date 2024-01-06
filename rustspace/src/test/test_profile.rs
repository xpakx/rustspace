use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server, prepare_server_with_user}, security::get_token};

#[tokio::test]
#[serial]
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
#[serial]
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

#[tokio::test]
#[serial]
async fn test_getting_profile_by_owner() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/Test")
            .header("Cookie", format!("Token={};", token))
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
    assert!(content.contains("Edit"));
}

#[tokio::test]
async fn test_getting_profile_form() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/profile")
            .header("Cookie", format!("Token={};", token))
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
    assert!(content.contains("profile"));
}

#[tokio::test]
async fn test_getting_profile_form_for_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/profile")
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
    assert!(content.contains("error"));
    assert!(content.contains("Unauthenticated"));
}
