use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use crate::AppState;

use self::{main::{root, about, help}, user::{user_page, register_form, register_user, check_password, check_username, check_email, check_password_repeat}};
mod main;
mod user;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/index", get(root))
        .route("/about", get(about))
        .route("/help", get(help))
        .route("/register", get(register_form))
        .route("/register", post(register_user))
        .route("/validation/psw", post(check_password))
        .route("/validation/username", post(check_username))
        .route("/validation/email", post(check_email))
        .route("/validation/psw_repeat", post(check_password_repeat))
        .route("/user", get(user_page))
}
