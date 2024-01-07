use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server, prepare_server_with_user, prepare_server_with_db}, security::get_token, db::get_db};

#[tokio::test]
#[serial]
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
#[serial]
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

#[tokio::test]
#[serial]
async fn test_changing_email_to_wrong_value() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/email")
            .body(Body::from("email=testveryveryververyveryveryververyveryveryververyveryveryververyveryveryververylongemail%40email.com"))
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
}

#[tokio::test]
#[serial]
async fn test_changing_email_to_duplicated_one() {
    let (token, _) = get_token(&Some(String::from("Test")));

    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;

    _ = sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
        .bind("Test")
        .bind("test@email.com")
        .bind("password")
        .execute(&db)
        .await;
    _ = sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
        .bind("Second")
        .bind("in_use@email.com")
        .bind("password")
        .execute(&db)
        .await;
    let response = prepare_server_with_db(db)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/email")
            .body(Body::from("email=in_use%40email.com"))
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
    assert!(content.contains("unique"));
}

#[tokio::test]
#[serial]
async fn test_changing_email() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/email")
            .body(Body::from("email=new%40email.com"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("new@email.com"));
}

#[tokio::test]
#[serial]
async fn test_changing_email_while_unauthenticated() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/email")
            .body(Body::from("email=new%40email.com"))
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

#[tokio::test]
#[serial]
async fn test_changing_password_to_wrong_value() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/password")
            .body(Body::from("psw=password&new_psw=new&psw_repeat=new"))
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
}

#[tokio::test]
#[serial]
async fn test_changing_password() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/password")
            .body(Body::from("psw=password&new_psw=new_password&psw_repeat=new_password"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("*****"));
}

#[tokio::test]
#[serial]
async fn test_changing_password_while_unauthenticated() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/password")
            .body(Body::from("psw=password&new_psw=new_password&psw_repeat=new_password"))
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

#[tokio::test]
#[serial]
async fn test_changing_password_with_wrong_password() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/password")
            .body(Body::from("psw=wrong_password&new_psw=new_password&psw_repeat=new_password"))
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
    assert!(content.contains("wrong"));
}
