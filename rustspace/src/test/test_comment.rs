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

async fn insert_post(username: &str, title: &str, content: &str, db: &PgPool) -> i32 {
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username)
        .fetch_optional(db)
        .await;
    let Ok(Some(user)) = user_db else {
        panic!("No such user!");
    };

    let result = sqlx::query_scalar("INSERT INTO posts (user_id, title, content) VALUES ($1, $2, $3) RETURNING id")
        .bind(&user.id)
        .bind(title)
        .bind(content)
        .fetch_one(db)
        .await;

    result.unwrap()
}

#[tokio::test]
#[serial]
async fn test_making_comment_request_with_no_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}/comments", post_id))
            .body(Body::from(""))
            .unwrap()
            )
        .await
        .unwrap();
    clear_posts(&db).await;

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
async fn test_making_comment_request_with_empty_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}/comments", post_id))
            .body(Body::from("content="))
            .unwrap()
            )
        .await
        .unwrap();
    clear_posts(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("content"));
    assert!(content.contains("empty"));
}
