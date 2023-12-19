use axum::{extract::FromRequestParts, http::request::Parts, async_trait};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use security::TokenClaims;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::postgres::PgPool;
use std::{sync::Arc, convert::Infallible};
use serde::{Serialize, Deserialize};

mod db;
mod router;
mod template;
mod validation;
mod security;

use crate::router::get_router;

struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustspace=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let db_url = "postgresql://root:password@localhost:5432/rustspace";
    let pool = db::get_db(db_url).await;
    let state = AppState { db: pool };

    let assets_path = std::env::current_dir().unwrap();
    info!("Assets path: {}", assets_path.to_str().unwrap());

    info!("Initializing router...");
    let app = get_router()
        .with_state(Arc::new(state))
        .nest_service("/assets", ServeDir::new(format!("{}/assets/", assets_path.to_str().unwrap())));

    let host = "0.0.0.0";
    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

#[derive(sqlx::FromRow)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct UserModel {
    id: Option<i32>,
    screen_name: String,
    email: String,
    password: String,
    // created_at: Option<chrono::DateTime<chrono::Utc>>,
    // updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    username: Option<String>,
    psw: Option<String>,
    psw_repeat: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    username: Option<String>,
    psw: Option<String>,
}

pub struct UserData {
    username: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserData
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let token = cookie_jar
            .get("Token")
            .map(|cookie| cookie.value().to_string())
            .filter(|value| value != "");

        if let Some(token) = token.clone() {
            let claims = decode::<TokenClaims>(
                &token,
                &DecodingKey::from_secret("secret".as_ref()),
                &Validation::default(),
                );
            if let Ok(claims) = claims {
                return Ok(UserData { username: Some(claims.claims.sub) });
            }
        }
        Ok(UserData { username: None })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
    use sqlx::{PgPool, Row};
    use tower::ServiceExt;

    use crate::{get_router, AppState, db::get_db};
    use serial_test::serial;

    // db
    // TODO: extract to integration tests

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

    async fn prepare_server_with_user() -> axum::Router {
        let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;
        
        _ = sqlx::query("DELETE FROM users")
            .execute(&db)
            .await;
        
        _ = sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
            .bind("Test")
            .bind("test@email.com")
            .bind("password")
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
        let response = prepare_server_with_user()
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
        let response = prepare_server_with_user()
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
        let response = prepare_server_with_user()
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
}
