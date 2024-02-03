use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Path}, Form};
use sqlx::Postgres;
use tracing::{info, debug};

use crate::{template::{HtmlTemplate, ErrorsTemplate}, UserData, AppState, UserModel, validation::validate_non_empty, PostRequest, BlogPostModel};

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

pub async fn delete_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>
    ) -> impl IntoResponse {
    info!("deleting blogpost requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
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
    let Some(user_id) = user.id else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting post from database");
    let post_db = sqlx::query_as::<Postgres, BlogPostModel>("SELECT * FROM posts WHERE id = $1")
        .bind(&post_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(post) = post_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(post) = post else {
        let template = ErrorsTemplate {errors: vec!["No such post!"]};
        return HtmlTemplate(template).into_response()
    };

    if &post.user_id != &user_id {
        let template = ErrorsTemplate {errors: vec!["You cannot delete this post!"]};
        return HtmlTemplate(template).into_response()
    }

    let query_result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(&post.id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(_) = query_result {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    }

    info!("post succesfully deleted.");
    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}

pub async fn edit_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>,
    Form(request): Form<PostRequest>
    ) -> impl IntoResponse {
    info!("updating blogpost requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
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
    let Some(user_id) = user.id else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting post from database");
    let post_db = sqlx::query_as::<Postgres, BlogPostModel>("SELECT * FROM posts WHERE id = $1")
        .bind(&post_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(post) = post_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(post) = post else {
        let template = ErrorsTemplate {errors: vec!["No such post!"]};
        return HtmlTemplate(template).into_response()
    };

    if &post.user_id != &user_id {
        let template = ErrorsTemplate {errors: vec!["You cannot edit this post!"]};
        return HtmlTemplate(template).into_response()
    }

    let query_result = sqlx::query("UPDATE posts  SET content = $1, title = $2 WHERE id = $3)")
        .bind(&request.content)
        .bind(&request.title)
        .bind(&post_id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(_) = query_result {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    }
    info!("post succesfully updated.");
    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}
