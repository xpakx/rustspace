use core::fmt;
use std::sync::Arc;
use serde::Deserialize;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use axum::{extract::{State, Query}, Form, http::HeaderMap, response::IntoResponse};
use rand_core::OsRng;
use sqlx::Postgres;
use tracing::{info, debug, error};

use crate::{AppState, template::{ErrorsTemplate, RegisterTemplate, UserTemplate, HtmlTemplate, FieldTemplate, UnauthorizedTemplate, LoginTemplate, EmailFormTemplate, PasswordFormTemplate, EmailFieldTemplate, PasswordFieldTemplate, AvatarFormTemplate}, UserRequest, validation::{validate_user, validate_password, validate_username, validate_email, validate_repeated_password, validate_login}, UserData, security::get_token, LoginRequest, UserModel, EmailRequest, PasswordRequest};

#[derive(Deserialize)]
pub struct FriendlyRedirect {
    path: Option<String>
}

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("register form sent");
    debug!("validating...");

    let mut errors = validate_user(&user);
    if errors.len() > 0 {
        debug!("user input is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    debug!("hashing password...");
    let password = hash_password(&user.psw.unwrap());
    if let Err(error) = password {
        error!("there was an error during hashing a password!");
        let template = ErrorsTemplate { errors: vec![error.message]};
        return HtmlTemplate(template).into_response()
    }

    debug!("trying to add user to db...");
    let query_result =
        sqlx::query("INSERT INTO users (screen_name, email, password) VALUES ($1, $2, $3)")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&password.unwrap())
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        debug!("cannot add user to db!");
        if err.contains("duplicate key") && err.contains("screen_name") {
            errors.push("Username must be unique!");
        }
        if err.contains("duplicate key") && err.contains("email") {
            errors.push("Email must be unique!");
        }
        if !err.contains("duplicate key") {
            errors.push("Couldn't add to db!");
        }
        debug!(err);
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }
    info!("user succesfully created.");

    let mut path = String::from("/user");
    if let Some(friendly_path) = user.redir {
        path = friendly_path
    }

    let mut headers = HeaderMap::new();
    headers.insert("HX-redirect", path.parse().unwrap());
    let (token, _) = get_token(&user.username);
    let cookie = format!("Token={}; SameSite=Lax", token);
    headers.insert("Set-Cookie", cookie.parse().unwrap());
    (headers, "Success").into_response()
}

pub async fn register_form(user: UserData, query: Query<FriendlyRedirect>) -> impl IntoResponse {
    info!("register form requested");
    let template = RegisterTemplate {path: "register", user, redir: query.path.to_owned()};
    return HtmlTemplate(template)
}

pub async fn user_page(user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("user index requested");
    if user.username.is_none() {
        let template = UnauthorizedTemplate {message: "You're unauthorized!", redir: Some(String::from("/user"))};
        return HtmlTemplate(template).into_response()
    }

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username)
        .fetch_optional(&state.db)
        .await;

    if let Ok(user_db) = user_db {
        if let Some(user_db) = user_db {
            // TODO
            let template = UserTemplate {path: "user", user, user_db};
            return HtmlTemplate(template).into_response()
        } else {
            debug!("no such user");
            let template = ErrorsTemplate {errors: vec!["No such user!"]};
            return HtmlTemplate(template).into_response()
        }
    } else {
        debug!("login unsuccessful due to db error");
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

pub async fn check_password(Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("request to validate password");
    debug!("validating...");

    let password = user.psw;
    let errors = validate_password(&password);
    let error = errors.len() > 0;
    let value = match password {
        Some(password) => password,
        None => String::from("")
    };
    let template = FieldTemplate {value, error, name: "psw", placeholder: "Enter Password", form_type: "password", text: "Password"};
    return HtmlTemplate(template).into_response()
}

pub async fn check_username(Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("request to validate username");
    debug!("validating...");

    let username = user.username;
    let errors = validate_username(&username);
    let error = errors.len() > 0;
    let value = match username {
        Some(username) => username,
        None => String::from("")
    };
    let template = FieldTemplate {value, error, name: "username", placeholder: "Enter Username", form_type: "text", text: "Username"};
    return HtmlTemplate(template).into_response()
}

pub async fn check_email(Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("request to validate email");
    debug!("validating...");

    let email = user.email;
    let errors = validate_email(&email);
    let error = errors.len() > 0;
    let value = match email {
        Some(email) => email,
        None => String::from("")
    };
    let template = FieldTemplate {value, error, name: "email", placeholder: "Enter Email", form_type: "text", text: "Email"};
    return HtmlTemplate(template).into_response()
}

pub async fn check_password_repeat(Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("request to validate password");
    debug!("validating...");

    let password = user.psw;
    let password_re = user.psw_repeat;
    let errors = validate_repeated_password(&password, &password_re);
    let error = errors.len() > 0;
    let value = match password_re {
        Some(password) => password,
        None => String::from("")
    };
    let template = FieldTemplate {value, error, name: "psw_repeat", placeholder: "Repeat Password", form_type: "password", text: "Repeat Password"};
    return HtmlTemplate(template).into_response()
}

fn hash_password(password: &String) -> Result<String, HashError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| {
            return HashError{message: "Couldn't hash password!"}
        })
    .map(|hash| hash.to_string())?;
    Ok(hash_password)
}

#[derive(Debug, Clone)]
struct HashError {
    message: &'static str
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hashing error")
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Form(user): Form<LoginRequest>) -> impl IntoResponse {
    info!("request to login");
    let errors = validate_login(&user.username, &user.psw);
    if errors.len() > 0 {
        debug!("login input is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username)
        .fetch_optional(&state.db)
        .await;

    if let Ok(user_db) = user_db {
        if let Some(user_db) = user_db {
            let is_valid = match PasswordHash::new(&user_db.password) {
                Ok(parsed_hash) => Argon2::default()
                    .verify_password(user.psw.unwrap().as_bytes(), &parsed_hash)
                    .map_or(false, |_| true),
                Err(_) => false,
            };
            if !is_valid {
                debug!("login unsuccessful due to wrong password");
                let template = ErrorsTemplate {errors: vec!["Wrong password!"]};
                return HtmlTemplate(template).into_response()
            }
            let mut path = String::from("/user");
            if let Some(friendly_path) = user.redir {
                path = friendly_path
            }
            let mut headers = HeaderMap::new();
            headers.insert("HX-redirect", path.parse().unwrap());
            let remember = match user.remember_me {
                None | Some(false) => false,
                Some(true) => true
            };
            debug!("remember_me={}", remember);
            let (token, date) = get_token(&user.username);

            let cookie = match remember {
                false => format!("Token={}; SameSite=Lax", token),
                true => format!("Token={}; Max-Age={}; SameSite=Lax", token, date),
            };
            debug!("cookie: {}", cookie);
            headers.insert("Set-Cookie", cookie.parse().unwrap());
            (headers, "Success").into_response()
        } else {
            debug!("no such user");
            let template = ErrorsTemplate {errors: vec!["No such user!"]};
            return HtmlTemplate(template).into_response()
        }
    } else {
        debug!("login unsuccessful due to db error");
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

pub async fn login_form(user: UserData, query: Query<FriendlyRedirect>) -> impl IntoResponse {
    info!("login form requested");
    let template = LoginTemplate {path: "login", user, redir: query.path.to_owned()};
    return HtmlTemplate(template)
}

pub async fn logout() -> impl IntoResponse {
    info!("request to logout");
    let mut headers = HeaderMap::new();
    headers.insert("HX-redirect", "/".parse().unwrap());
    let cookie = String::from("Token=");
    headers.insert("Set-Cookie", cookie.parse().unwrap());
    (headers, "Success").into_response()
}

pub async fn to_login(query: Query<FriendlyRedirect>) -> impl IntoResponse {
    info!("redir to login requested");
    let mut headers = HeaderMap::new();
    let path = format!("/login?path={}", query.path.to_owned().unwrap());
    headers.insert("HX-redirect", path.parse().unwrap());
    (headers, "Success").into_response()
}

pub async fn edit_email() -> impl IntoResponse {
    info!("email form requested");
    let template = EmailFormTemplate {};
    return HtmlTemplate(template)
}

pub async fn edit_password() -> impl IntoResponse {
    info!("password form requested");
    let template = PasswordFormTemplate {};
    return HtmlTemplate(template)
}

pub async fn update_email(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<EmailRequest>) -> impl IntoResponse {
    info!("request to update email");
    let mut errors = validate_email(&request.email);
    if errors.len() > 0 {
        debug!("email is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }
    if user.username.is_none() {
        errors.push("Unauthenticated!");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    let result = sqlx::query("UPDATE users SET email= $1 WHERE screen_name = $2")
        .bind(&request.email)
        .bind(&user.username)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    match result {
        Err(err) => {
            debug!("changing email unsuccessful due to db error");
            if err.contains("duplicate key") && err.contains("email") {
                errors.push("Email must be unique!");
            } else {
                errors.push("Couldn't update due to database error!");
            }
            debug!(err);
            let template = ErrorsTemplate {errors};
            return HtmlTemplate(template).into_response()
        },
        Ok(_) => {
            let template = EmailFieldTemplate {email: request.email.unwrap()};
            return HtmlTemplate(template).into_response()
        }
    }
}

pub async fn update_password(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<PasswordRequest>) -> impl IntoResponse {
    info!("request to update password");
    let mut errors = validate_password(&request.new_psw);
    errors.append(&mut validate_repeated_password(&request.new_psw, &request.psw_repeat));
    if errors.len() > 0 {
        debug!("password input is invalid");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }
    if user.username.is_none() {
        errors.push("Unauthenticated!");
        let template = ErrorsTemplate {errors};
        return HtmlTemplate(template).into_response()
    }

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username)
        .fetch_optional(&state.db)
        .await;

    if let Ok(user_db) = user_db {
        if let Some(user_db) = user_db {
            let is_valid = match PasswordHash::new(&user_db.password) {
                Ok(parsed_hash) => Argon2::default()
                    .verify_password(request.psw.unwrap().as_bytes(), &parsed_hash)
                    .map_or(false, |_| true),
                Err(_) => false,
            };
            if !is_valid {
                debug!("password change unsuccessful due to wrong password");
                let template = ErrorsTemplate {errors: vec!["Old is wrong password!"]};
                return HtmlTemplate(template).into_response()
            }

            debug!("hashing password...");
            let password = hash_password(&request.new_psw.unwrap());
            if let Err(error) = password {
                error!("there was an error during hashing a password!");
                let template = ErrorsTemplate { errors: vec![error.message]};
                return HtmlTemplate(template).into_response()
            }

            let result = sqlx::query("UPDATE users SET password = $1 WHERE screen_name = $2")
                .bind(password.unwrap())
                .bind(&user.username)
                .execute(&state.db)
                .await
                .map_err(|err: sqlx::Error| err.to_string());

            if let Ok(_) = result {
                let template = PasswordFieldTemplate{};
                return HtmlTemplate(template).into_response()
            } else {
                debug!("password change unsuccessful due to db error");
                let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
                return HtmlTemplate(template).into_response()
            }
        } else {
            debug!("no such user");
            let template = ErrorsTemplate {errors: vec!["No such user!"]};
            return HtmlTemplate(template).into_response()
        }
    } else {
        debug!("password change unsuccessful due to db error");
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

pub async fn edit_avatar() -> impl IntoResponse {
    info!("avatar form requested");
    let template = AvatarFormTemplate {};
    return HtmlTemplate(template)
}
