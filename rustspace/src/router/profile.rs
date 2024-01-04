use std::sync::Arc;

use axum::{response::IntoResponse, extract::{Path, State}};
use sqlx::Postgres;
use tracing::info;

use crate::{template::{ProfileTemplate, HtmlTemplate, UserNotFoundTemplate}, UserData, UserModel, AppState, ProfileModel};

pub async fn profile(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>
    ) -> impl IntoResponse {
   info!("profile of user {} requested", username);

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&username)
        .fetch_optional(&state.db)
        .await;
    let owner = match &user.username {
        None => false,
        Some(current_user) => current_user == &username
    };

    let Ok(Some(user_db)) = user_db else {
        let template = UserNotFoundTemplate {};
        return HtmlTemplate(template).into_response()
    };
    let Some(user_id) = user_db.id else {
        let template = ProfileTemplate {path: "index", user, username, profile: None, owner};
        return HtmlTemplate(template).into_response()
    };

    let profile = sqlx::query_as::<Postgres, ProfileModel>(
        "SELECT * FROM profiles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;

    let Ok(profile) = profile else {
        let template = ProfileTemplate {path: "index", user, username, profile: None, owner};
        return HtmlTemplate(template).into_response()
    };

   let template = ProfileTemplate {path: "index", user, username, profile, owner};
   return HtmlTemplate(template).into_response()
}
