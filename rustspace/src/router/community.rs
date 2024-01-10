use std::sync::Arc;

use axum::{response::IntoResponse, extract::State};
use sqlx::Postgres;
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

    let users  = sqlx::query_as::<Postgres, UserDetails>(
        "SELECT u.id, u.screen_name, p.real_name, p.gender, p.city
        FROM users u
        LEFT JOIN profiles p ON u.id = p.user_id",
        )
        .fetch_all(&state.db)
        .await;
    match users {
        Err(err) => {
            debug!("{}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok(users) => {
            let template = CommunityTemplate {path: "community", user, users};
            return HtmlTemplate(template).into_response()
        }
    };
}
