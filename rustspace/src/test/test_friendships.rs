use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::{Postgres, PgPool};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_new_user, clear_friendships}, security::get_token, UserModel};

#[tokio::test]
#[serial]
async fn test_making_friendship_request_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Authentication Error"));
}

#[tokio::test]
#[serial]
async fn test_making_friendship_request_with_no_name() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
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
    assert!(content.contains("username"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_making_friendship_request_with_empty_name() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username="))
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
    assert!(content.contains("username"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_user_trying_to_befriend_themself() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=Test"))
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
    assert!(content.contains("befriend yourself"));
}

#[tokio::test]
#[serial]
async fn test_user_trying_to_befriend_nonexistent_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
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
    assert!(content.contains("not found"));
}

async fn insert_friendship_request(username: &str, username2: &str, db: &PgPool) {
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username)
        .fetch_optional(db)
        .await;
    let Ok(Some(user)) = user_db else {
        panic!("No such user!");
    };
    let friend_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username2)
        .fetch_optional(db)
        .await;
    let Ok(Some(friend)) = friend_db else {
        panic!("No such user!");
    };

    _ = sqlx::query("INSERT INTO friendships (user_id, friend_id) VALUES ($1, $2)")
        .bind(&user.id)
        .bind(&friend.id)
        .execute(db)
        .await;
}

#[tokio::test]
#[serial]
async fn test_making_request_while_its_already_created() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_friendship_request("Test", "User", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("already created"));
    clear_friendships(&db).await;
}
