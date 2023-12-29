use std::sync::Arc;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use rand_core::OsRng;
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use crate::{get_router, AppState, db::get_db, security::get_token};
use serial_test::serial;

async fn prepare_server() -> axum::Router {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;

    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

#[tokio::test]
async fn test_index() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/")
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
    assert!(content.contains("Welcome!"));
    assert!(content.contains("Homepage"));
}

#[tokio::test]
async fn test_about() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/about")
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
    assert!(content.contains("About us"));
}

#[tokio::test]
async fn test_help() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/help")
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
    assert!(content.contains("Help"));
}

#[tokio::test]
async fn test_getting_register_form() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/register")
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
    assert!(content.contains("Register"));
}

async fn prepare_server_with_user(hash_password: bool) -> axum::Router {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;

    _ = sqlx::query("DELETE FROM users")
        .execute(&db)
        .await;


    let salt = SaltString::generate(&mut OsRng);
    let password = match hash_password {
        true => Argon2::default()
            .hash_password("password".as_bytes(), &salt)
            .map(|hash| hash.to_string()).unwrap(),
        false => String::from("password")
    };

    _ = sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
        .bind("Test")
        .bind("test@email.com")
        .bind(password)
        .execute(&db)
        .await;

    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

async fn prepare_db() -> PgPool {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;

    _ = sqlx::query("DELETE FROM users")
        .execute(&db)
        .await;
    db
}

async fn prepare_server_with_db(db: PgPool) -> axum::Router {
    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

#[tokio::test]
#[serial]
async fn test_validating_duplicated_username() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/register")
            .body(Body::from("username=Test&email=aaa%40email.com&psw=password&psw_repeat=password"))
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
    assert!(content.contains("Username"));
    assert!(content.contains("unique"));
}

#[tokio::test]
#[serial]
async fn test_validating_duplicated_email() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/register")
            .body(Body::from("username=Username&email=test%40email.com&psw=password&psw_repeat=password"))
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
    assert!(content.contains("Email"));
    assert!(content.contains("unique"));
}

#[tokio::test]
#[serial]
async fn test_redirecting_after_successful_registration() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/register")
            .body(Body::from("username=User&email=user%40email.com&psw=password&psw_repeat=password"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/user");
    }
}

#[tokio::test]
#[serial]
async fn test_adding_user_to_db() {
    let db = prepare_db().await;
    _ = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/register")
            .body(Body::from("username=Test&email=test%40email.com&psw=password&psw_repeat=password"))
            .unwrap()
            )
        .await;

    let result = sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(&db)
        .await;

    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 1);
    }
}

#[tokio::test]
#[serial]
async fn test_authentication_for_nonexistent_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/login")
            .body(Body::from("username=User&psw=password"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("No such user"));
}

#[tokio::test]
#[serial]
async fn test_authentication_with_wrong_password() {
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/login")
            .body(Body::from("username=Test&psw=tst"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("password"));
}

#[tokio::test]
#[serial]
async fn test_authentication() {
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/login")
            .body(Body::from("username=Test&psw=password"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/user");
    }
    let header = response.headers().get("Set-Cookie");
    assert!(header.is_some());
    if let Some(header) = header {
        assert!(header.to_str().unwrap().contains("Token="));
    }
}

#[tokio::test]
#[serial]
async fn test_logging_out() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/logout")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/");
    }
    let header = response.headers().get("Set-Cookie");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "Token=");
    }
}

#[tokio::test]
#[serial]
async fn test_navigation_for_guest() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/")
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
    assert!(content.contains("/register"));
    assert!(content.contains("/login"));
    assert!(!content.contains("/logout"));
    assert!(!content.contains("/user"));
}

#[tokio::test]
#[serial]
async fn test_navigation_for_authorized_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/")
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
    assert!(!content.contains("/register"));
    assert!(!content.contains("/login"));
    assert!(content.contains("/logout"));
    assert!(content.contains("/user"));
}

#[tokio::test]
#[serial]
async fn test_user_redir_for_unauthenticated() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/user")
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
    assert!(content.contains("Unauthorized"));
}

#[tokio::test]
#[serial]
async fn test_user_redir_for_authenticated() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/user")
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
    assert!(!content.contains("Unauthorized"));
}
