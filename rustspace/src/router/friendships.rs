use std::sync::Arc;

use axum::{response::IntoResponse, extract::State, Form};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};

use crate::{template::{HtmlTemplate, ErrorsTemplate}, UserData, AppState, UserModel, FriendshipModel, FriendshipRequest, validation::validate_non_empty, FriendshipDetails};

pub async fn send_friend_request(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<FriendshipRequest>
    ) -> impl IntoResponse {
    info!("sending friend request requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    if !validate_non_empty(&request.username) {
        let template = ErrorsTemplate {errors: vec!["Username cannot be empty!"]};
        return HtmlTemplate(template).into_response()
    }

    let username = request.username.unwrap();

    debug!("getting user from database");
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&user.username)
        .fetch_optional(&state.db)
        .await;
    let Ok(Some(user)) = user_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let friend_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username)
        .fetch_optional(&state.db)
        .await;
    let Ok(friend) = friend_db else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(friend) = friend else {
        let template = ErrorsTemplate {errors: vec!["User not found!"]};
        return HtmlTemplate(template).into_response()
    };

    let friendship = sqlx::query_as::<Postgres, FriendshipModel>(
        "SELECT * FROM friendships WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
        )
        .bind(&friend.id)
        .bind(&user.id)
        .fetch_optional(&state.db)
        .await;
    let Ok(friendship) = friendship else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    match friendship {
        Some(accepted) if accepted.accepted =>  {
            let template = ErrorsTemplate {errors: vec!["You're already friends!"]};
            return HtmlTemplate(template).into_response()
        },
        Some(rejected) if rejected.rejected => {
            let template = ErrorsTemplate {errors: vec!["User already have rejected your request!"]};
            return HtmlTemplate(template).into_response()
        },
        Some(_) => {
            let template = ErrorsTemplate {errors: vec!["Request already created!"]};
            return HtmlTemplate(template).into_response()
        },
        None => {
            let query_result =
                sqlx::query("INSERT INTO friendships (user_id, friend_id) VALUES ($1, $2)")
                .bind(&user.id)
                .bind(&friend.id)
                .execute(&state.db)
                .await
                .map_err(|err: sqlx::Error| err.to_string());

            if let Err(_) = query_result {
                let template = ErrorsTemplate {errors: vec!["Couldn't send request!"]};
                return HtmlTemplate(template).into_response()
            }
            info!("request succesfully created.");
            let template = ErrorsTemplate {errors: vec!["TODO"]};
            return HtmlTemplate(template).into_response()
        }
    }
}

#[allow(dead_code)]
async fn get_friend_requests(db: &PgPool, user_id: i32, page: i32, accepted: bool, rejected: bool, get_count: bool) -> Result<(Vec<FriendshipDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, FriendshipDetails>(
        "SELECT f.id, u.screen_name, f.accepted, f.rejected, f.created_at
        FROM users u
        LEFT JOIN friendships f ON u.id = f.friend_id
        WHERE f.user_id = $3 AND f.accepted = $4 AND f.rejected = $5
        ORDER BY f.created_at
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(user_id)
        .bind(accepted)
        .bind(rejected)
        .fetch_all(db)
        .await?;
    if !get_count {
        return Ok((users, None));
    }

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE f.user_id = $1 AND f.accepted = $2 AND f.rejected = $3")
        .bind(user_id)
        .bind(accepted)
        .bind(rejected)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

#[allow(dead_code)]
async fn get_friends(db: &PgPool, user_id: i32, page: i32, get_count: bool) -> Result<(Vec<FriendshipDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, FriendshipDetails>(
        "SELECT f.id, u.screen_name, f.accepted, f.rejected, f.created_at
        FROM users u
        LEFT JOIN friendships f ON u.id = f.friend_id
        WHERE (f.user_id = $3 OR f.friend_id = $3) AND f.accepted = true
        ORDER BY f.created_at
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(user_id)
        .fetch_all(db)
        .await?;
    if !get_count {
        return Ok((users, None));
    }

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE (f.user_id = $1 OR f.friend_id = $1) AND f.accepted = true")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}
