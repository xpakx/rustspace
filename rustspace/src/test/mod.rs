use std::sync::Arc;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_core::OsRng;
use sqlx::{PgPool, Postgres};

use crate::{get_router, AppState, db::get_db, UserModel};

mod test_routes;
mod test_auth;
mod test_user;
mod test_profile;
mod test_community;
mod test_friendships;
mod test_post;

async fn clear_db(db: &PgPool) {
    _ = sqlx::query("DELETE FROM users")
        .execute(db)
        .await;
}

async fn clear_profiles(db: &PgPool) {
    _ = sqlx::query("DELETE FROM profiles")
        .execute(db)
        .await;
}

async fn clear_friendships(db: &PgPool) {
    _ = sqlx::query("DELETE FROM friendships")
        .execute(db)
        .await;
}

async fn prepare_server() -> axum::Router {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;
    clear_db(&db).await;
    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

async fn prepare_server_with_user(hash_password: bool) -> axum::Router {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;
    clear_db(&db).await;
    insert_default_user(hash_password, &db).await;
    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

async fn prepare_db() -> PgPool {
    let db = get_db("postgresql://root:password@localhost:5432/rustspacetest").await;
    clear_db(&db).await;
    db
}

async fn prepare_server_with_db(db: PgPool) -> axum::Router {
    let app = get_router()
        .with_state(Arc::new(AppState{db}));
    app
}

async fn insert_default_user(hash_password: bool, db: &PgPool) {
    insert_user("Test", "test@email.com", hash_password, db).await;
}

async fn insert_user(username: &str, email: &str, hash_password: bool, db: &PgPool) {
    let password = match hash_password {
        true => {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
            .hash_password("password".as_bytes(), &salt)
            .map(|hash| hash.to_string()).unwrap()
        },
        false => String::from("password")
    };

    _ = sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
        .bind(username)
        .bind(email)
        .bind(password)
        .execute(db)
        .await;
}

async fn insert_new_user(username: &str, password: &str, db: &PgPool) {
    insert_user(username, password, false, db).await;
}

async fn insert_users(amount: i32, username_prefix: &str, db: &PgPool) {
    _ = sqlx::query("INSERT INTO users (screen_name, email, password) 
                    SELECT concat($2, a), concat($2, concat(a, '@mail.com')), 'password' 
                    FROM generate_series(1, $1) as s(a);
                    ")
        .bind(amount)
        .bind(username_prefix)
        .execute(db)
        .await;
}

async fn insert_requests(accepted: bool, rejected: bool, db: &PgPool) {
    let user = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind("Test")
        .fetch_optional(db)
        .await;
    let Ok(Some(user)) = user else {
        panic!("No user in db!");
    };
    let Some(user_id) = user.id else {
        panic!("No user id!");
    };
    _ = sqlx::query("INSERT INTO friendships (user_id, friend_id, accepted, rejected) 
                    SELECT u.id, $1, $2, $3 FROM users u WHERE screen_name <> $4")
        .bind(user_id)
        .bind(accepted)
        .bind(rejected)
        .bind("Test")
        .execute(db)
        .await;
}
