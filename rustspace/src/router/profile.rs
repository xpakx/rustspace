use std::sync::Arc;

use axum::{response::IntoResponse, extract::{Path, State}, Form};
use sqlx::Postgres;
use tracing::{info, debug};

use crate::{template::{ProfileTemplate, HtmlTemplate, UserNotFoundTemplate, ProfileFormTemplate, ErrorsTemplate}, UserData, UserModel, AppState, ProfileModel, ProfileRequest};

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

pub async fn edit_profile() -> impl IntoResponse {
    info!("profile form requested");
    let template = ProfileFormTemplate {};
    return HtmlTemplate(template)
}

pub async fn update_profile(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<ProfileRequest>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username.unwrap())
        .fetch_optional(&state.db)
        .await;

    let Ok(Some(user_db)) = user_db else {
        let template = ErrorsTemplate {errors: vec!["There was a problem with database!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(user_id) = user_db.id else {
        let template = ErrorsTemplate {errors: vec!["There was a problem with database!"]};
        return HtmlTemplate(template).into_response()
    };

    let result = sqlx::query("UPDATE profiles SET gender = $1, city = $2, description = $3, real_name = $4 WHERE user_id = $5")
        .bind(request.gender.unwrap())
        .bind(request.city.unwrap())
        .bind(request.description.unwrap())
        .bind(request.name.unwrap())
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Ok(_) = result {
        let template = ProfileFormTemplate {}; //field template
        return HtmlTemplate(template).into_response()
    } else {
        debug!("password change unsuccessful due to db error");
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

