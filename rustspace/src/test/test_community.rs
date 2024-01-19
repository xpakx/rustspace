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

#[tokio::test]
#[serial]
async fn test_getting_users_catalogue_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=0&search=t&update_count=false&pages=1")
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
    println!("{}", content);
    assert!(content.contains("unauthorized"));
}

#[tokio::test]
#[serial]
async fn test_getting_users_catalogue() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_new_user("Test2", "test2@mail.com", &db).await;
    insert_new_user("Aaa", "aaa@mail.com", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=0&search=t&update_count=false&pages=1")
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
    assert!(content.contains("Test"));
    assert!(content.contains("Test2"));
    assert!(!content.contains("Aaa"));
}

#[tokio::test]
#[serial]
async fn test_getting_users_catalogue_with_two_pages() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(30, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=0&search=u&update_count=true")
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
    let re = Regex::new(r"user[0-9]").unwrap();
    let count = re.find_iter(content).count();
    assert_eq!(count, 25*2); // link and username
    assert!(content.contains("30 users found"));
    assert!(content.contains("page=1"));
    assert!(!content.contains("page=2"));
}

#[tokio::test]
#[serial]
async fn test_getting_second_page_of_catalogue() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=1&search=u&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(content.contains("\"current\">1<"));
    assert!(content.contains("page=2"));
    assert!(!content.contains("page=3"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_third_page_of_catalogue() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=2&search=u&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(content.contains("page=1"));
    assert!(!content.contains("page=2"));
    assert!(content.contains("\"current\">2<"));
    assert!(content.contains("page=3"));
    assert!(!content.contains("page=4"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_page_in_the_middle() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=7&search=u&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(!content.contains("page=5"));
    assert!(content.contains("page=6"));
    assert!(!content.contains("page=7"));
    assert!(content.contains("\"current\">7<"));
    assert!(content.contains("page=8"));
    assert!(!content.contains("page=9"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_last_page() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=11&search=u&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(content.contains("page=10"));
    assert!(!content.contains("page=11"));
    assert!(content.contains("\"current\">11<"));
}

#[tokio::test]
#[serial]
async fn test_getting_search_page_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users")
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
async fn test_getting_search_page() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users")
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
    assert!(content.contains("Search for user"));
}

#[tokio::test]
#[serial]
async fn test_searching_users() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("user1", "user1@mail.com", &db).await;
    insert_new_user("auser", "user2@mail.com", &db).await;
    insert_new_user("aaauser55", "user3@mail.com", &db).await;
    insert_new_user("User", "user4@mail.com", &db).await;
    insert_new_user("aaaaa", "user5@mail.com", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=0&search=user&update_count=true")
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
    assert!(content.contains("4 users found"));
    assert!(!content.contains("Test"));
    assert!(content.contains("user1"));
    assert!(content.contains("auser"));
    assert!(content.contains("aaauser55"));
    assert!(content.contains("User"));
    assert!(!content.contains("aaaaa"));
}

#[tokio::test]
#[serial]
async fn test_getting_users_search_results_with_two_pages() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(30, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=0&search=user&update_count=true")
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
    let re = Regex::new(r"user[0-9]").unwrap();
    let count = re.find_iter(content).count();
    assert_eq!(count, 25*2); // link and username
    assert!(content.contains("30 users found"));
    assert!(content.contains("page=1"));
    assert!(!content.contains("page=2"));
}

#[tokio::test]
#[serial]
async fn test_getting_second_page_of_user_search_results() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=1&search=user&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(content.contains("\"current\">1<"));
    assert!(content.contains("page=2"));
    assert!(!content.contains("page=3"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_third_page_of_user_search_results() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=2&search=user&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(content.contains("page=1"));
    assert!(!content.contains("page=2"));
    assert!(content.contains("\"current\">2<"));
    assert!(content.contains("page=3"));
    assert!(!content.contains("page=4"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_page_of_user_search_results_in_the_middle() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=7&search=user&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(!content.contains("page=5"));
    assert!(content.contains("page=6"));
    assert!(!content.contains("page=7"));
    assert!(content.contains("\"current\">7<"));
    assert!(content.contains("page=8"));
    assert!(!content.contains("page=9"));
    assert!(content.contains("page=11"));
}

#[tokio::test]
#[serial]
async fn test_getting_last_page_of_user_search_result() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_users(300, "user", &db).await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=11&search=user&update_count=true")
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
    assert!(content.contains("300 users found"));
    assert!(content.contains("page=0"));
    assert!(!content.contains("page=1&"));
    assert!(content.contains("page=10"));
    assert!(!content.contains("page=11"));
    assert!(content.contains("\"current\">11<"));
}

#[tokio::test]
#[serial]
async fn test_requesting_negative_page_of_catalogue() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=-1&search=u&update_count=false")
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
    assert!(content.contains("error"));
    assert!(content.contains("negative"));
}

#[tokio::test]
#[serial]
async fn test_requesting_negative_page_of_user_search_results() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/users/search?page=-1&search=user&update_count=false")
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
    assert!(content.contains("error"));
    assert!(content.contains("negative"));
}

#[tokio::test]
#[serial]
async fn test_requesting_too_long_search_query_in_catalogue() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .uri("/community/search?page=0&search=user&update_count=false")
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
    assert!(content.contains("error"));
    assert!(content.contains("single letter"));
}
