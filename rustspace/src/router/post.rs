use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Path, Query}, Form, http::{HeaderMap, HeaderValue}};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};
use serde::Deserialize;

use crate::{template::{HtmlTemplate, ErrorsTemplate, UserNotFoundTemplate, PostTemplate, PostsTemplate, PostsResultTemplate, PostFormTemplate}, UserData, AppState, UserModel, validation::validate_non_empty, PostRequest, BlogPostModel};

fn validate_post(request: &PostRequest) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(&request.content) {
        errors.push("Post content cannot be empty!");
    }
    if !validate_non_empty(&request.title) {
        errors.push("Post title cannot be empty!");
    }
    return errors;
}

fn post_action_validate(user: &UserData, request: &PostRequest,) -> Option<ErrorsTemplate> {
    if user.username.is_none() {
        return Some(ErrorsTemplate {errors: vec!["Unauthenticated!"]});
    }
    let errors = validate_post(&request);
    if !errors.is_empty() {
        return Some(ErrorsTemplate {errors});
    }
    return None;
}

pub async fn get_user_by_name(db: &PgPool, username: &String) -> Result<Option<UserModel>, sqlx::Error> {
    debug!("getting user from database");
    return sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(username)
        .fetch_optional(db)
        .await;
}

pub async fn insert_post(db: &PgPool, user_id: &i32, request: &PostRequest) -> Result<i64, String> {
    debug!("saving post in database");
    return sqlx::query_scalar("INSERT INTO posts (user_id, content, title) VALUES ($1, $2, $3) RETURNING id")
        .bind(user_id)
        .bind(&request.content)
        .bind(&request.title)
        .fetch_one(db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());
}

pub async fn add_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<PostRequest>
    ) -> impl IntoResponse {
    info!("adding blogpost requested");
    let error = post_action_validate(&user, &request);
    if let Some(template) = error {
        return HtmlTemplate(template).into_response()
    }

    let username = user.username.unwrap();
    let Ok(Some(user)) = get_user_by_name(&state.db, &username).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let user_id = user.id.unwrap();
    let query_result = insert_post(&state.db, &user_id, &request).await;

    let Ok(id) = query_result else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    info!("post succesfully created.");

    let mut headers = HeaderMap::new();
    headers.insert("HX-redirect", HeaderValue::from_str(&format!("/blog/{}", id)).unwrap());
    (headers, "Success").into_response()
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
    let username = user.username.unwrap();
    let Ok(Some(user)) = get_user_by_name(&state.db, &username).await else {
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
    let error = post_action_validate(&user, &request);
    if let Some(template) = error {
        return HtmlTemplate(template).into_response()
    }

    debug!("getting user from database");
    let username = user.username.unwrap();
    let Ok(Some(user)) = get_user_by_name(&state.db, &username).await else {
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

    let Ok(Some(user_db)) = get_user_by_name(&state.db, &username).await else {
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
        Ok((posts, records)) => {
            let pages = records_to_count(records);
            let template = PostsTemplate {posts, user, pages, username, path: "/posts"};
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
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>
    ) -> impl IntoResponse {
    info!("blogpost requested");

    let Ok(Some(user_db)) = get_user_by_name(&state.db, &username).await else {
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
        Ok((posts, results)) => {
            let pages = records_to_count(results);
            let template = PostsResultTemplate {posts, pages, username, page: query.page};
            return HtmlTemplate(template).into_response()
        }
    };
}

fn records_to_count(records: Option<i64>) -> i32 {
    match records {
        None => 0,
        Some(count) => {
            let count = (count as f64)/25.0;
            let count = count.ceil() as i32;
            count
        }
    }
}

pub async fn post_form(user: UserData) -> impl IntoResponse {
    info!("register form requested");
    let template = PostFormTemplate {path: "register", user};
    return HtmlTemplate(template)
}
