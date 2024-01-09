use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::{PgPool, Postgres};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server, prepare_server_with_user, prepare_server_with_db, prepare_db, insert_default_user, clear_profiles}, security::get_token, UserModel, ProfileModel};

#[tokio::test]
#[serial]
async fn test_getting_nonexistent_profile() {
    let response = prepare_server()
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/User")
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
    assert!(content.contains("no such user"));
}

#[tokio::test]
#[serial]
async fn test_getting_profile() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/Test")
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
    assert!(content.contains("Test's profile"));
}

#[tokio::test]
#[serial]
async fn test_getting_profile_by_owner() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/profile/Test")
            .header("Cookie", format!("Token={};", token))
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
    assert!(content.contains("Test's profile"));
    assert!(content.contains("Edit"));
}

#[tokio::test]
#[serial]
async fn test_getting_profile_form() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/profile")
            .header("Cookie", format!("Token={};", token))
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
    assert!(content.contains("profile"));
}

#[tokio::test]
#[serial]
async fn test_getting_profile_form_for_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .uri("/forms/profile")
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
    assert!(content.contains("Unauthenticated"));
}

#[tokio::test]
#[serial]
async fn test_changing_profile_while_unauthenticated() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/profile")
            .body(Body::from("gender=&city=London&description=&real_name="))
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
async fn test_changing_profile_without_profile() {
    let db = prepare_db().await;
    insert_default_user(false, &db).await;
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/profile")
            .body(Body::from("gender=&city=London&description=&real_name="))
            .unwrap()
            )
        .await
        .unwrap();
    
    clear_profiles(&db).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("London"));
}

async fn insert_default_profile(db: &PgPool) {
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind("Test")
        .fetch_optional(db)
        .await;
    let user_id = user_db.unwrap().unwrap().id.unwrap();

    _ = sqlx::query("INSERT INTO profiles (gender, city, real_name, description, user_id) VALUES ($1, $2, $3, $4m $5)")
        .bind("male")
        .bind("London")
        .bind("James Bond")
        .bind("007")
        .bind("test@email.com")
        .bind(user_id)
        .execute(db)
        .await;
}

async fn get_profile(db: &PgPool) -> Option<ProfileModel> {
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind("Test")
        .fetch_optional(db)
        .await;
    let user_id = user_db.unwrap().unwrap().id.unwrap();

    let profile = sqlx::query_as::<Postgres, ProfileModel>(
        "SELECT * FROM profiles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(db)
        .await;

    return profile.unwrap();
}

#[tokio::test]
#[serial]
async fn test_changing_profile_with_existing_profile() {
    let db = prepare_db().await;
    insert_default_user(false, &db).await;
    insert_default_profile(&db).await;
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/profile")
            .body(Body::from("gender=&city=Cambridge&description=&real_name="))
            .unwrap()
            )
        .await
        .unwrap();
    
    clear_profiles(&db).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Cambridge"));
}

#[tokio::test]
#[serial]
async fn test_changing_profile_in_db() {
    let db = prepare_db().await;
    insert_default_user(false, &db).await;
    insert_default_profile(&db).await;
    let (token, _) = get_token(&Some(String::from("Test")));
    _ = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("PUT")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .uri("/profile")
            .body(Body::from("gender=&city=Cambridge&description=&real_name="))
            .unwrap()
            )
        .await
        .unwrap();
    let profile = get_profile(&db).await;
    clear_profiles(&db).await;
    
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert!(profile.gender.is_none());
    assert!(profile.city.is_some());
    let city = profile.city.unwrap();
    assert_eq!(city, "Cambridge");
    assert!(profile.description.is_none());
    assert!(profile.real_name.is_none());
}
