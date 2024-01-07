use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::Row;
use tower::ServiceExt;

use crate::security::get_token;
use serial_test::serial;

use crate::test::{prepare_server, prepare_db, prepare_server_with_db, prepare_server_with_user};

#[tokio::test]
#[serial]
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
    assert!(content.contains("/to_login?path=/user"));
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
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(!content.contains("Unauthorized"));
}

#[tokio::test]
#[serial]
async fn test_login_page_add_friendly_redirect_to_form() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/login?path=/home")
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
    assert!(content.contains("name=\"redir\""));
    assert!(content.contains("value=\"/home\""));
}

#[tokio::test]
#[serial]
async fn test_friendly_redir_after_authentication() {
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/login")
            .body(Body::from("username=Test&psw=password&redir=/home"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/home");
    }
}

#[tokio::test]
#[serial]
async fn test_friendly_redir_after_registration() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/register")
            .body(Body::from("username=User&email=user%40email.com&psw=password&psw_repeat=password&redir=/home"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("HX-redirect");
    assert!(header.is_some());
    if let Some(header) = header {
        assert_eq!(header.to_str().unwrap(), "/home");
    }
}

#[tokio::test]
#[serial]
async fn test_remembering() {
    let response = prepare_server_with_user(true)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/login")
            .body(Body::from("username=Test&psw=password&remember_me=true"))
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
        assert!(header.to_str().unwrap().contains("Max-Age=3600"));
    }
}
