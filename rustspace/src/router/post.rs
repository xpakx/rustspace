use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Path, Query}, Form};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};
use serde::Deserialize;

use crate::{template::{HtmlTemplate, ErrorsTemplate, UserNotFoundTemplate, PostTemplate, PostsTemplate}, UserData, AppState, UserModel, validation::validate_non_empty, PostRequest, BlogPostModel};

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

pub async fn get_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>
    ) -> impl IntoResponse {
    info!("blogpost requested");

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

    let template = PostTemplate {post, user, path: "/post"};
    return HtmlTemplate(template).into_response()
}

pub async fn get_users_posts(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>
    ) -> impl IntoResponse {
    info!("blogpost requested");
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&username)
        .fetch_optional(&state.db)
        .await;

    let Ok(Some(user_db)) = user_db else {
        let template = UserNotFoundTemplate {};
        return HtmlTemplate(template).into_response()
    };

    let Some(user_id) = user_db.id else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting posts from database");
    let posts = get_posts(&state.db, user_id, 0).await;
    match posts {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((posts, _)) => {
            let template = PostsTemplate {posts, user, path: "/posts"};
            return HtmlTemplate(template).into_response()
        }
    };
}

async fn get_posts(db: &PgPool, user_id: i32, page: i32) -> Result<(Vec<BlogPostModel>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, BlogPostModel>(
        "SELECT * FROM posts WHERE user_id = $1 
        ORDER BY created_at
        LIMIT $2 OFFSET $3 "
        )
        .bind(&user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(db)
        .await?;

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE f.friend_id = $1 AND f.rejected = true OR f.cancelled = true")
        .bind(&user_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    page: i32,
}

pub async fn posts_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>
    ) -> impl IntoResponse {
    info!("blogpost requested");
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&username)
        .fetch_optional(&state.db)
        .await;

    let Ok(Some(user_db)) = user_db else {
        let template = UserNotFoundTemplate {};
        return HtmlTemplate(template).into_response()
    };

    let Some(user_id) = user_db.id else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting posts from database");
    let posts = get_posts(&state.db, user_id, query.page).await;
    match posts {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((posts, _)) => {
            let template = PostsTemplate {posts, user, path: "/posts"};
            return HtmlTemplate(template).into_response()
        }
    };
}