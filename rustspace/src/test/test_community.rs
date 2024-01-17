use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use regex::Regex;
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_users, insert_new_user}, security::get_token};

#[tokio::test]
#[serial]
async fn test_getting_community_page_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/community")
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
    assert!(content.contains("Authentication Error"));
}

#[tokio::test]
#[serial]
async fn test_getting_community_page() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/community")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 9000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Community"));
}

#[tokio::test]
#[serial]
async fn test_getting_community_page_with_correct_users() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("Aaa", "aaa@mail.com", &db).await;
    insert_new_user("Aaa2", "aaa2@mail.com", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 9000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Aaa"));
    assert!(content.contains("Aaa2"));
    assert!(content.contains("2 users found"));
    assert!(!content.contains("Test"));
}

#[tokio::test]
#[serial]
async fn test_getting_community_page_with_two_pages() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(30, "aaa", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 9000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    println!("{}", content);
    let re = Regex::new(r"aaa").unwrap();
    let count = re.find_iter(content).count();
    assert_eq!(count, 25*2); // link and username
    assert!(content.contains("30 users found"));
}
