use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::Row;
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_new_user, clear_posts}, security::get_token};

#[tokio::test]
#[serial]
async fn test_making_post_request_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("title=title&content=content"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Unauthenticated"));
}

#[tokio::test]
#[serial]
async fn test_making_post_request_with_no_title() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("content=content"))
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
    assert!(content.contains("title"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_making_post_request_with_empty_title() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("title=&content=content"))
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
    assert!(content.contains("title"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_making_post_request_with_no_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("title=title"))
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
    assert!(content.contains("content"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_making_post_request_with_empty_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("title=title&content="))
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
    assert!(content.contains("content"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_adding_post() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog")
            .body(Body::from("title=title&content=content"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result = sqlx::query("SELECT COUNT(*) FROM posts")
        .fetch_one(&db)
        .await;
    clear_posts(&db).await;

    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 1);
    }
}
// deleting post

#[tokio::test]
#[serial]
async fn test_deleting_post_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/1")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Unauthenticated"));
}

#[tokio::test]
#[serial]
async fn test_deleting_nonexistent_post() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/1")
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
    assert!(content.contains("No such post"));
}
