use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Path, Query}, Form};
use sqlx::{PgPool, postgres::PgQueryResult, Postgres};
use tracing::{info, debug};
use serde::Deserialize;

use crate::{template::{HtmlTemplate, ErrorsTemplate, CommentsTemplate, CommentFormTemplate}, UserData, AppState, validation::validate_non_empty, CommentRequest, BlogCommentModel, BlogCommentDetails};

fn validate_comment(request: &CommentRequest) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(&request.content) {
        errors.push("Post content cannot be empty!");
    }
    return errors;
}

fn comment_action_validate(user: &UserData, request: &CommentRequest) -> Option<ErrorsTemplate> {
    if user.username.is_none() {
        return Some(ErrorsTemplate {errors: vec!["Unauthenticated!"]});
    }
    let errors = validate_comment(&request);
    if !errors.is_empty() {
        return Some(ErrorsTemplate {errors});
    }
    return None;
}

pub async fn get_user_by_name(db: &PgPool, username: &String) -> Result<Option<i32>, sqlx::Error> {
    debug!("getting user from database");
    return sqlx::query_scalar("SELECT id FROM users WHERE screen_name = $1")
        .bind(username)
        .fetch_optional(db)
        .await;
}

pub async fn insert_comment(db: &PgPool, user_id: &i32, post_id: &i32, request: &CommentRequest) -> Result<PgQueryResult, String> {
    debug!("saving comment in database");
    return sqlx::query("INSERT INTO comments (user_id, post_id, content) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(post_id)
        .bind(&request.content)
        .execute(db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());
}

pub async fn add_comment(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>,
    Form(request): Form<CommentRequest>,
    ) -> impl IntoResponse {
    info!("adding blog comment requested");
    let error = comment_action_validate(&user, &request);
    if let Some(template) = error {
        return HtmlTemplate(template).into_response()
    }

    let username = user.username.unwrap();
    let Ok(Some(user_id)) = get_user_by_name(&state.db, &username).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let query_result = insert_comment(&state.db, &user_id, &post_id, &request).await;
    if let Err(_) = query_result {
        debug!("Db error: {:?}", query_result);
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    info!("comment succesfully created.");

    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}

pub async fn delete_comment(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i32>
    ) -> impl IntoResponse {
    info!("deleting comment requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    debug!("getting user from database");
    let username = user.username.unwrap();
    let Ok(Some(user_id)) = get_user_by_name(&state.db, &username).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting comment from database");
    let comment_db = sqlx::query_as::<Postgres, BlogCommentModel>("SELECT * FROM comments WHERE id = $1")
        .bind(&comment_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(comment) = comment_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(comment) = comment else {
        let template = ErrorsTemplate {errors: vec!["No such comment!"]};
        return HtmlTemplate(template).into_response()
    };

    if &comment.user_id != &user_id {
        let template = ErrorsTemplate {errors: vec!["You cannot delete this comment!"]};
        return HtmlTemplate(template).into_response()
    }

    let query_result = sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(&comment.id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(_) = query_result {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    }

    info!("comment succesfully deleted.");
    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}

pub async fn edit_comment(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i32>,
    Form(request): Form<CommentRequest>
    ) -> impl IntoResponse {
    info!("updating blog comment requested");
    let error = comment_action_validate(&user, &request);
    if let Some(template) = error {
        return HtmlTemplate(template).into_response()
    }

    debug!("getting user from database");
    let username = user.username.unwrap();
    let Ok(Some(user_id)) = get_user_by_name(&state.db, &username).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting comment from database");
    let comment_db = sqlx::query_as::<Postgres, BlogCommentModel>("SELECT * FROM comments WHERE id = $1")
        .bind(&comment_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(comment) = comment_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(comment) = comment else {
        let template = ErrorsTemplate {errors: vec!["No such comment!"]};
        return HtmlTemplate(template).into_response()
    };

    if &comment.user_id != &user_id {
        let template = ErrorsTemplate {errors: vec!["You cannot edit this comment!"]};
        return HtmlTemplate(template).into_response()
    }

    let query_result = sqlx::query("UPDATE comments SET content = $1 WHERE id = $2")
        .bind(&request.content)
        .bind(&comment_id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(_) = query_result {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    }
    info!("comment succesfully updated.");
    let template = ErrorsTemplate {errors: vec!["TODO"]};
    return HtmlTemplate(template).into_response()
}

pub async fn comments_for_post(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>
    ) -> impl IntoResponse {
    info!("comments requested");

    debug!("getting comments from database");
    let comments = get_comments(&state.db, post_id, 0).await;
    match comments {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((comments, records)) => {
            let pages = records_to_count(records);
            let template = CommentsTemplate {comments, pages, post_id, page: 0, user};
            return HtmlTemplate(template).into_response()
        }
    };
}

async fn get_comments(db: &PgPool, post_id: i32, page: i32) -> Result<(Vec<BlogCommentDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, BlogCommentDetails>(
        "SELECT c.*, u.screen_name 
        FROM comments c 
        LEFT JOIN users u
        ON c.user_id = u.id
        WHERE c.post_id = $1 
        ORDER BY c.created_at
        LIMIT $2 OFFSET $3 "
        )
        .bind(&post_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(db)
        .await?;

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments
        WHERE post_id = $1")
        .bind(&post_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    page: i32,
}

pub async fn comments_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>
    ) -> impl IntoResponse {

    debug!("getting comments from database");
    let comments = get_comments(&state.db, post_id, query.page).await;
    match comments {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((comments, results)) => {
            let pages = records_to_count(results);
            let template = CommentsTemplate {comments, pages, post_id, page: query.page, user };
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

pub async fn comment_form(
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i32>
    ) -> impl IntoResponse {
    let comment_db = sqlx::query_as::<Postgres, BlogCommentModel>(
        "SELECT * FROM comments WHERE id = $1")
        .bind(&comment_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(comment) = comment_db else {
        debug!("Db error: {:?}", comment_db);
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(comment) = comment else {
        let template = ErrorsTemplate {errors: vec!["No such comment!"]};
        return HtmlTemplate(template).into_response()
    };

    let template = CommentFormTemplate {comment, comment_id};
    return HtmlTemplate(template).into_response()
}
