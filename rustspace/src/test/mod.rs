use std::sync::Arc;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_core::OsRng;
use sqlx::PgPool;

use crate::{get_router, AppState, db::get_db};

mod test_routes;
mod test_auth;
mod test_user;

async fn prepare_server() -> axum::Router {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;

    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
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

