use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::{Row, PgPool, Postgres};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_new_user, clear_posts, clear_comments}, security::get_token, UserModel, BlogCommentModel};

// adding comment

#[tokio::test]
#[serial]
async fn test_making_comment_request_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/1/comments")
            .body(Body::from("content=content"))
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
