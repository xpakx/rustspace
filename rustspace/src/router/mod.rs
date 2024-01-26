use std::sync::Arc;

use axum::{Router, routing::{get, post, put, delete}};

use crate::AppState;

use self::{
    main::{root, about, help},
    user::{user_page, register_form, register_user, check_password, check_username, check_email, check_password_repeat, login_form, login, logout, to_login, edit_email, edit_password, update_email, update_password, edit_avatar, upload_avatar, delete_avatar}, 
    profile::{profile, edit_profile, update_profile}, community::{community, get_users_page, search_users, get_search_users_page}, friendships::{send_friend_request, friends, requests, change_request_state}
};
mod main;
mod user;
mod profile;
mod community;
mod friendships;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/index", get(root))
        .route("/about", get(about))
        .route("/help", get(help))
        .route("/register", get(register_form))
        .route("/register", post(register_user))
        .route("/login", get(login_form))
        .route("/login", post(login))
        .route("/to_login", get(to_login))
        .route("/logout", get(logout))
        .route("/validation/psw", post(check_password))
        .route("/validation/username", post(check_username))
        .route("/validation/email", post(check_email))
        .route("/validation/psw_repeat", post(check_password_repeat))
        .route("/user", get(user_page))
        .route("/forms/email", get(edit_email))
        .route("/email", put(update_email))
        .route("/forms/password", get(edit_password))
        .route("/password", put(update_password))
        .route("/profile/:username", get(profile))
        .route("/forms/profile", get(edit_profile))
        .route("/profile", put(update_profile))
        .route("/community", get(community))
        .route("/community/search", get(get_users_page))
        .route("/community/users", get(search_users))
        .route("/community/users/search", get(get_search_users_page))
        .route("/forms/avatar", get(edit_avatar))
        .route("/avatar", post(upload_avatar))
        .route("/avatar", delete(delete_avatar))
        .route("/friendships", post(send_friend_request))
        .route("/friends", get(friends))
        .route("/friends/requests", get(requests))
        .route("/friends/requests/:id", post(change_request_state))
}
