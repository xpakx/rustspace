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

#[tokio::test]
#[serial]
async fn test_making_comment_request_to_nonexistent_post() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
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
    assert!(content.contains("No such post"));
}

#[tokio::test]
#[serial]
async fn test_adding_comment() {
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
            .body(Body::from("content=content"))
            .unwrap()
            )
        .await
        .unwrap();

    let result = sqlx::query("SELECT COUNT(*) FROM comments")
        .fetch_one(&db)
        .await;
    clear_posts(&db).await;
    clear_comments(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 1);
    }
}

// deleting comment

#[tokio::test]
#[serial]
async fn test_deleting_comment_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/comment/1")
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
async fn test_deleting_nonexistent_comment() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/comment/1")
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
    assert!(content.contains("No such comment"));
}

async fn insert_comment(username: &str, post_id: &i32, content: &str, db: &PgPool) -> i32 {
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username)
        .fetch_optional(db)
        .await;
    let Ok(Some(user)) = user_db else {
        panic!("No such user!");
    };

    let result = sqlx::query_scalar("INSERT INTO comments (user_id, post_id, content) VALUES ($1, $2, $3) RETURNING id")
        .bind(&user.id)
        .bind(post_id)
        .bind(content)
        .fetch_one(db)
        .await;

    result.unwrap()
}

#[tokio::test]
#[serial]
async fn test_deleting_comment_authored_by_different_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("User", &post_id, "content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();
    clear_posts(&db).await;
    clear_comments(&db).await;

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
async fn test_deleting_comment() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("User", "user@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("Test", &post_id, "content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("DELETE")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let result = sqlx::query("SELECT COUNT(*) FROM comments")
        .fetch_one(&db)
        .await;
    clear_posts(&db).await;
    clear_comments(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 0);
    }
}

// editing comment

#[tokio::test]
#[serial]
async fn test_editing_comment_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/comment/1")
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

#[tokio::test]
#[serial]
async fn test_editing_nonexistent_comment() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/blog/comment/1")
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
    assert!(content.contains("No such comment"));
}

#[tokio::test]
#[serial]
async fn test_editing_comment_authored_by_different_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("User", &post_id, "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::from("content=content"))
            .unwrap()
            )
        .await
        .unwrap();
    clear_posts(&db).await;
    clear_comments(&db).await;

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
async fn test_editing_comment_with_no_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("User", "user@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("Test", &post_id, "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::from(""))
            .unwrap()
            )
        .await
        .unwrap();

    clear_posts(&db).await;
    clear_comments(&db).await;
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
async fn test_editing_comment_with_empty_content() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("User", "user@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("Test", &post_id, "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::from("content="))
            .unwrap()
            )
        .await
        .unwrap();

    clear_posts(&db).await;
    clear_comments(&db).await;
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
async fn test_editing_comment() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("User", "user@mail.com", &db).await;
    let post_id = insert_post("User", "Title", "Content", &db).await;
    let comment_id = insert_comment("Test", &post_id, "Content", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri(format!("/blog/comment/{}", comment_id))
            .body(Body::from("content=new_content"))
            .unwrap()
            )
        .await
        .unwrap();


    let comment = sqlx::query_as::<Postgres, BlogCommentModel>("SELECT * FROM comments WHERE id = $1")
        .bind(&comment_id)
        .fetch_optional(&db)
        .await;
    clear_posts(&db).await;
    clear_comments(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(comment.is_ok());
    if let Ok(post) = comment {
        assert!(post.is_some());
        if let Some(post) = post {
            assert_eq!(post.content.unwrap(), "new_content");
        }
    }
}
