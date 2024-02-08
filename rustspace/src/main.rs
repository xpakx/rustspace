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

#[cfg(test)]
mod test;

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
    _ = std::fs::create_dir("assets/avatars");

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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct UserModel {
    id: Option<i32>,
    screen_name: String,
    email: String,
    password: String,
    avatar: Option<bool>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct ProfileModel {
    id: Option<i32>,
    user_id: i32,
    gender: Option<String>,
    city: Option<String>,
    description: Option<String>,
    real_name: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[allow(non_snake_case)]
struct UserDetails {
    id: Option<i32>,
    screen_name: String,
    real_name: Option<String>,
    gender: Option<String>,
    city: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct FriendshipModel {
    id: Option<i32>,
    user_id: i32,
    friend_id: i32,
    accepted: bool,
    rejected: bool,
    cancelled: bool,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    accepted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[allow(non_snake_case)]
struct FriendshipDetails {
    id: Option<i32>,
    screen_name: String,
    accepted: bool,
    rejected: bool,
    cancelled: bool,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    username: Option<String>,
    psw: Option<String>,
    psw_repeat: Option<String>,
    email: Option<String>,
    redir: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FriendshipRequest {
    username: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FriendshipStateRequest {
    state: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailRequest {
    email: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordRequest {
    psw: Option<String>,
    new_psw: Option<String>,
    psw_repeat: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    username: Option<String>,
    psw: Option<String>,
    redir: Option<String>,
    remember_me: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileRequest {
    gender: Option<String>,
    city: Option<String>,
    description: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[allow(non_snake_case)]
struct BlogPostModel {
    id: Option<i32>,
    user_id: i32,
    title: Option<String>,
    content: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct PostRequest {
    title: Option<String>,
    content: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[allow(non_snake_case)]
struct BlogPostDetails {
    id: Option<i32>,
    user_id: i32,
    screen_name: String,
    title: Option<String>,
    content: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[allow(non_snake_case)]
struct BlogCommentModel {
    id: Option<i32>,
    user_id: i32,
    post_id: i32,
    content: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct CommentRequest {
    content: Option<String>,
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

        if let Some(token) = token {
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
