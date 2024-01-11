use std::sync::Arc;

use axum::{response::IntoResponse, extract::State};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};

use crate::{template::{CommunityTemplate, HtmlTemplate, ErrorsTemplate}, UserData, AppState, UserDetails};

pub async fn community(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("community page requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let users = get_users(&state.db, 0).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((users, records)) => {
            let template = CommunityTemplate {path: "community", user, users, records};
            return HtmlTemplate(template).into_response()
        }
    };
}

async fn get_users(db: &PgPool, page: i32) -> Result<(Vec<UserDetails>, i64), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, UserDetails>(
        "SELECT u.id, u.screen_name, p.real_name, p.gender, p.city
        FROM users u
        LEFT JOIN profiles p ON u.id = p.user_id
        LIMIT $1 OFFSET $2",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(db)
        .await?;
    let records: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await?;
    
    Ok((users, records))
}
