use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::{Row, PgPool, Postgres};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_new_user, clear_posts}, security::get_token, UserModel};

// adding post

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

    let result = sqlx::query("SELECT COUNT(*) FROM posts")
        .fetch_one(&db)
        .await;
    clear_posts(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 1);
    }
}

// deleting post

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

#[tokio::test]
#[serial]
async fn test_deleting_post_authored_by_different_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::empty())
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
    assert!(content.contains("cannot delete"));
}

#[tokio::test]
#[serial]
async fn test_deleting_post() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let result = sqlx::query("SELECT COUNT(*) FROM posts")
        .fetch_one(&db)
        .await;
    clear_posts(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/user/Test/blog");
    }
    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 0);
    }
}

// editing post

#[tokio::test]
#[serial]
async fn test_editing_post_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/1")
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
async fn test_editing_nonexistent_post() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/1")
            .body(Body::from("title=title&content=content"))
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

#[tokio::test]
#[serial]
async fn test_editing_post_authored_by_different_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::from("title=title&content=content"))
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
    assert!(content.contains("cannot edit"));
}

#[tokio::test]
#[serial]
async fn test_editing_post_with_no_title() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::from("content=content"))
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
    assert!(content.contains("title"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_editing_post_with_empty_title() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::from("title=&content=content"))
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
    assert!(content.contains("title"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_editing_post_with_no_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::from("title=title"))
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
async fn test_editing_post_with_empty_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("Test", "Title", "Content", &db).await;
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/{}", post_id))
            .body(Body::from("title=title&content="))
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
