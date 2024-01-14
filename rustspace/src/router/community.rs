use std::sync::Arc;

use axum::{response::IntoResponse, extract::{State, Query}};
use sqlx::{Postgres, PgPool};
use tracing::{info, debug};
use serde::Deserialize;

use crate::{template::{CommunityTemplate, HtmlTemplate, ErrorsTemplate, UnauthorizedTemplate, CommunityResultsTemplate, SearchTemplate}, UserData, AppState, UserDetails, validation::{validate_length, validate_alphanumeric}};

pub async fn community(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("community page requested");
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthorized!", redir: Some(String::from("/community"))};
        return HtmlTemplate(template).into_response()
    }

    let users = get_users(&state.db, "a%", 0, true).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((users, records)) => {
            let template = CommunityTemplate {path: "community", user, users, records};
            return HtmlTemplate(template).into_response()
        }
    };
}

async fn get_users(db: &PgPool, pattern: &str, page: i32, get_count: bool) -> Result<(Vec<UserDetails>, Option<i64>), sqlx::Error> {
    let page_size = 25;
    let offset = page_size * page;
    let users = sqlx::query_as::<Postgres, UserDetails>(
        "SELECT u.id, u.screen_name, p.real_name, p.gender, p.city
        FROM users u
        LEFT JOIN profiles p ON u.id = p.user_id
        WHERE u.screen_name ILIKE $3
        ORDER BY screen_name
        LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .bind(pattern)
        .fetch_all(db)
        .await?;
    if !get_count {
        return Ok((users, None));
    }

    let records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users u
        WHERE u.screen_name ILIKE $1")
        .bind(pattern)
        .fetch_one(db)
        .await?;
    
    Ok((users, Some(records)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    page: i32,
    letter: String,
    update_count: bool,
    pages: Option<i32>,
}

pub async fn get_users_page(
    user: UserData,
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("community results requested, page {}, letter {}", &query.page, &query.letter);
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["You're unauthorized"]};
        return HtmlTemplate(template).into_response()
    }

    let errors = validate_users_query(&query);
    if errors.len() > 0 {
        debug!("user input is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    let letter = format!("{}%", &query.letter);

    let users = get_users(&state.db, letter.as_str(), query.page, query.update_count).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((users, records)) => {
            debug!("Users fetched from db");
            debug!("{:?}", users);
            let pages = match records {
                None => match query.pages {
                    None => 0,
                    Some(p) => p
                },
                Some(count) => {
                    let count = (count as f64)/25.0;
                    let count = count.ceil() as i32;
                    count
                }
            };
            let template = CommunityResultsTemplate {users, records, page: query.page, pages, query: query.letter};
            return HtmlTemplate(template).into_response()
        }
    };
}

pub fn validate_users_query(query: &SearchQuery) -> Vec<&'static str> {
    let mut errors = vec![];
    let single_letter = validate_length(&query.letter, 1, 1);
    let valid_char = validate_alphanumeric(&query.letter);
    if !(single_letter && valid_char) {
        errors.push("Must be single letter or digit!");
    }
    if query.page < 0 {
        errors.push("Page cannot be nagative!");
    }
    return errors;
}

pub async fn search_users(user: UserData) -> impl IntoResponse {
    info!("user search page requested");
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthorized!", redir: Some(String::from("/community/users"))};
        return HtmlTemplate(template).into_response()
    }

    let template = SearchTemplate {path: "community", user};
    return HtmlTemplate(template).into_response()
}

#[derive(Deserialize)]
pub struct SearchUsersQuery {
    page: i32,
    search: String,
    update_count: bool,
    pages: Option<i32>
}

pub fn validate_search_users_query(query: &SearchUsersQuery) -> Vec<&'static str> {
    let mut errors = vec![];
    if query.page < 0 {
        errors.push("Page cannot be nagative!");
    }
    return errors;
}

pub async fn get_search_users_page(
    user: UserData,
    Query(query): Query<SearchUsersQuery>,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("search user results requested, page {}, query {}", &query.page, &query.search);
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["You're unauthorized"]};
        return HtmlTemplate(template).into_response()
    }

    let errors = validate_search_users_query(&query);
    if errors.len() > 0 {
        debug!("user input is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    let search_string = format!("%{}%", &query.search);

    let users = get_users(&state.db, search_string.as_str(), query.page, query.update_count).await;
    match users {
        Err(err) => {
            debug!("Database error: {}", err);
            let template = ErrorsTemplate {errors: vec!["Db error!"]};
            return HtmlTemplate(template).into_response()
        },
        Ok((users, records)) => {
            debug!("Users fetched from db");
            debug!("{:?}", users);
            let pages = match records {
                None => match query.pages {
                    None => 0,
                    Some(p) => p
                },
                Some(count) => {
                    let count = (count as f64)/25.0;
                    let count = count.ceil() as i32;
                    count
                }
            };
            
            let template = CommunityResultsTemplate {users, records, page: query.page, pages, query: query.search};
            return HtmlTemplate(template).into_response()
        }
    };
}
