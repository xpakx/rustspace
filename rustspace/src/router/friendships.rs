use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Path, Query}, Form};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};
use chrono::Utc;
use serde::Deserialize;

use crate::{template::{HtmlTemplate, ErrorsTemplate, UnauthorizedTemplate, FriendRequestsTemplate, FriendsTemplate, FriendRequestsResultsTemplate, InvitedTemplate, RejectedFriendRequestsTemplate, RejectedRequestsResultsTemplate}, UserData, AppState, UserModel, FriendshipModel, FriendshipRequest, FriendshipStateRequest, validation::validate_non_empty, FriendshipDetails};

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

    if let Some(logged) = &user.username {
       if logged == &username {
            let template = ErrorsTemplate {errors: vec!["You cannot befriend yourself!"]};
            return HtmlTemplate(template).into_response()
        }
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
            let template = InvitedTemplate {};
            return HtmlTemplate(template).into_response()
        }
    }
}

async fn get_friend_requests(db: &PgPool, user_id: i32, page: i32) -> Result<(Vec<FriendshipDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, FriendshipDetails>(
        "SELECT f.id, u.screen_name, f.accepted, f.rejected, f.cancelled, f.created_at
        FROM users u
        LEFT JOIN friendships f ON u.id = f.user_id
        WHERE f.friend_id = $3 AND f.accepted = false AND f.rejected = false AND f.cancelled = false
        ORDER BY f.created_at
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE f.friend_id = $1 AND f.accepted = false AND f.rejected = false AND f.cancelled = false")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

async fn get_friends(db: &PgPool, user_id: i32, page: i32) -> Result<(Vec<FriendshipDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, FriendshipDetails>(
        "SELECT f.id, u.screen_name, f.accepted, f.rejected, f.cancelled, f.created_at
        FROM users u
        LEFT JOIN friendships f ON u.id = f.friend_id
        WHERE f.user_id = $3 AND f.accepted = true AND f.cancelled = false
        UNION
        SELECT fr.id, us.screen_name, fr.accepted, fr.rejected, fr.cancelled, fr.created_at
        FROM users us
        LEFT JOIN friendships fr ON us.id = fr.user_id
        WHERE fr.friend_id = $3 AND fr.accepted = true AND fr.cancelled = false
        ORDER BY created_at
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE (f.user_id = $1 OR f.friend_id = $1) AND f.accepted = true AND f.cancelled = false")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

pub async fn requests(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthenticated!", redir: Some(String::from("/friends/requests"))};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_friend_requests(&state.db, user_id, 0).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((friends, records)) => {
            let pages = records_to_count(records);
            let template = FriendRequestsTemplate {path: "/friends", friends, pages, user, records };
            return HtmlTemplate(template).into_response()
        }
    };
}

#[derive(Deserialize)]
pub struct SearchQuery {
    page: i32,
}

pub async fn requests_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["You're unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_friend_requests(&state.db, user_id, query.page).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((friends, records)) => {
            let pages = records_to_count(records);
            let template = FriendRequestsResultsTemplate {friends, pages, page: query.page};
            return HtmlTemplate(template).into_response()
        }
    };
}

pub async fn friends(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthenticated!", redir: Some(String::from("/friends"))};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_friends(&state.db, user_id, 0).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((friends, records)) => {
            let pages = records_to_count(records);
            let template = FriendsTemplate {path: "/friends", friends, pages, user, records };
            return HtmlTemplate(template).into_response()
        }
    };
}

pub async fn friends_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["You're unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_friends(&state.db, user_id, query.page).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((_, records)) => {
            let _ = records_to_count(records);
            let template = ErrorsTemplate {errors: vec!["TODO"]};
            return HtmlTemplate(template).into_response()
        }
    };
}

pub async fn get_user_id(db: &PgPool, user: &UserData) -> Option<i32> {
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username)
        .fetch_optional(db)
        .await;

    let Ok(Some(user_db)) = user_db else {
        return None;
    };

    let Some(user_id) = user_db.id else {
        return None;
    };
    Some(user_id)
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

pub async fn change_request_state(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<i32>,
    Form(request): Form<FriendshipStateRequest>
    ) -> impl IntoResponse {
    info!("sending friend request requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let (accepted, rejected, error) = match request.state {
        None => (false, false, Some("State cannot be empty!")),
        Some(state) if state == "accepted" => (true, false, None),
        Some(state) if state == "rejected" => (false, true, None),
        Some(_) => (false, false, Some("Unsupported state!"))
    };

    if let Some(error) = error {
        let template = ErrorsTemplate {errors: vec![error]};
        return HtmlTemplate(template).into_response()
    };

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

    let friendship = sqlx::query_as::<Postgres, FriendshipModel>(
        "SELECT * FROM friendships WHERE id = $1",
        )
        .bind(&request_id)
        .fetch_optional(&state.db)
        .await;

    let Ok(friendship) = friendship else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let Some(friendship) = friendship else {
        let template = ErrorsTemplate {errors: vec!["No such request!"]};
        return HtmlTemplate(template).into_response()
    };


    let accepted_at = match &accepted {
        true => Some(Utc::now()),
        false => friendship.accepted_at
    };
    if &friendship.friend_id != &user_id {
        if (friendship.cancelled && rejected) || (!friendship.cancelled && accepted) {
            let template = ErrorsTemplate {errors: vec!["TODO"]};
            return HtmlTemplate(template).into_response()
        }
        let result = sqlx::query("UPDATE friendships SET cancelled = $1, accepted_at = $2 WHERE id = $3")
            .bind(&rejected)
            .bind(&accepted_at)
            .bind(&request_id)
            .execute(&state.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());
        if let Ok(_) = result {
            let template = ErrorsTemplate {errors: vec!["TODO"]};
            return HtmlTemplate(template).into_response()
        } else {
            let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
            return HtmlTemplate(template).into_response()
        }
    }
    
    if (friendship.rejected && rejected) || (friendship.accepted && accepted) {
        let template = ErrorsTemplate {errors: vec!["TODO"]};
        return HtmlTemplate(template).into_response()
    }

    let result = sqlx::query("UPDATE friendships SET accepted = $1, rejected = $2, accepted_at = $3 WHERE id = $4")
        .bind(&accepted)
        .bind(&rejected)
        .bind(&accepted_at)
        .bind(&request_id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());
    if let Ok(_) = result {
        let template = ErrorsTemplate {errors: vec!["TODO"]};
        return HtmlTemplate(template).into_response()
    } else {
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

async fn get_rejected_requests(db: &PgPool, user_id: i32, page: i32) -> Result<(Vec<FriendshipDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, FriendshipDetails>(
        "SELECT f.id, u.screen_name, f.accepted, f.rejected, f.cancelled, f.created_at
        FROM users u
        LEFT JOIN friendships f ON u.id = f.user_id
        WHERE f.friend_id = $3 AND f.rejected = true OR f.cancelled = true
        ORDER BY f.created_at
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friendships f
        WHERE f.friend_id = $1 AND f.rejected = true OR f.cancelled = true")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

pub async fn rejected_requests(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthenticated!", redir: Some(String::from("/friends/requests/rejected"))};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_rejected_requests(&state.db, user_id, 0).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((friends, records)) => {
            let pages = records_to_count(records);
            let template = RejectedFriendRequestsTemplate {path: "/friends", friends, pages, user, records };
            return HtmlTemplate(template).into_response()
        }
    };
}

pub async fn rejected_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["You're unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    let Some(user_id) = get_user_id(&state.db, &user).await else {
        let template = ErrorsTemplate {errors: vec!["Db error!"]};
        return HtmlTemplate(template).into_response()
    };

    let users = get_rejected_requests(&state.db, user_id, query.page).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((friends, records)) => {
            let pages = records_to_count(records);
            let template = RejectedRequestsResultsTemplate {friends, pages, page: query.page};
            return HtmlTemplate(template).into_response()
        }
    };
}
