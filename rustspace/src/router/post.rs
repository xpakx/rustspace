use std::sync::Arc;

use axum::{response::IntoResponse, extract::State, Form};
use sqlx::Postgres;
use tracing::{info, debug};

use crate::{template::{HtmlTemplate, ErrorsTemplate}, UserData, AppState, UserModel, validation::validate_non_empty, PostRequest};

pub async fn add_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<PostRequest>
    ) -> impl IntoResponse {
    info!("adding blogpost requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    if !validate_non_empty(&request.content) {
        let template = ErrorsTemplate {errors: vec!["Post content cannot be empty!"]};
        return HtmlTemplate(template).into_response()
    }
    debug!("getting user from database");
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&user.username)
        .fetch_optional(&state.db)
        .await;
    let Ok(Some(user)) = user_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let query_result = sqlx::query("INSERT INTO posts (user_id, content, title) VALUES ($1, $2, $3)")
        .bind(&user.id)
        .bind(&request.content)
        .bind(&request.title)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(_) = query_result {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    }
    info!("post succesfully created.");
    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}
