use axum::{routing::{get, post}, Router, Form};
use axum::response::{Html, IntoResponse, Response};
use axum::http::{StatusCode, HeaderMap};
use askama::Template;
use tower_http::services::ServeDir;
use tracing::{info, debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::postgres::{PgPool, PgPoolOptions};
use regex::Regex;
use std::sync::Arc;
use axum::extract::State;
use serde::{Serialize, Deserialize};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_core::OsRng;
use std::fmt;

struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustspace=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let db_url = "postgresql://root:password@localhost:5432/rustspace";

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();
    let state = AppState { db: pool };

    info!("Initializing router...");

    let assets_path = std::env::current_dir().unwrap();
    let app = Router::new()
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
        .route("/user", get(user))
        .with_state(Arc::new(state))
        .nest_service("/assets", ServeDir::new(format!("{}/assets/", assets_path.to_str().unwrap())));

    let host = "0.0.0.0";
    let port = 3000;

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
   info!("index requested");
   let template = RootTemplate {path: "index"};
   return HtmlTemplate(template)
}

async fn about() -> impl IntoResponse {
   info!("about requested");
   let template = AboutTemplate {path: "about"};
   return HtmlTemplate(template)
}

async fn help() -> impl IntoResponse {
   info!("help requested");
   let template = HelpTemplate {path: "help"};
   return HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "index.html")]
#[allow(dead_code)]
pub struct RootTemplate {
    path: &'static str
}

#[derive(Template)]
#[template(path = "about.html")]
#[allow(dead_code)]
pub struct AboutTemplate {
    path: &'static str
}

#[derive(Template)]
#[template(path = "help.html")]
#[allow(dead_code)]
pub struct HelpTemplate {
    path: &'static str
}

#[derive(Template)]
#[template(path = "register.html")]
#[allow(dead_code)]
pub struct RegisterTemplate {
    path: &'static str,
}

#[derive(Template)]
#[template(path = "user.html")]
#[allow(dead_code)]
pub struct UserTemplate {
    path: &'static str,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T> where T: Template, {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(sqlx::FromRow)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct UserModel {
    id: Option<i32>,
    screen_name: String,
    email: String,
    password: String,
    // created_at: Option<chrono::DateTime<chrono::Utc>>,
    // updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn validate_user(user: &UserRequest) -> Vec<&'static str> {
    let mut errors = vec![];
    errors.append(&mut validate_username(&user.username));
    errors.append(&mut validate_email(&user.email));
    errors.append(&mut validate_password(&user.psw));
    errors.append(&mut validate_repeated_password(&user.psw, &user.psw_repeat));
    return errors;
}

fn validate_password(password: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(password) {
        errors.push("Password cannot be empty!");
    }
    if let Some(password) = password {
        if !validate_length(password, 4, 20) {
            errors.push("Password must have length between 4 and 20 characters!");
        }
    }
    errors
}

fn validate_repeated_password(password: &Option<String>, password_re: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(password_re) {
        errors.push("Password cannot be empty!");
    }
    if let (Some(password), Some(password_re)) = (password, password_re) {
        if !validate_same(password.clone(), password_re.clone()) {
            errors.push("Passwords must match!");
        }
    }
    errors
}

fn validate_username(username: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(username) {
        errors.push("Username cannot be empty!");
    }
    if let Some(name) = username {
        if !validate_length(name, 4, 20) {
            errors.push("Username must have length between 4 and 20 characters!");
        }
        if !validate_alphanumeric(name) {
            errors.push("Username must contain only letters, numbers, or the underscore!");
        }
    }
    errors
}

fn validate_email(email: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(email) {
        errors.push("Email cannot be empty!");
    }
    if let Some(email) = email {
        if !validate_length(email, 0, 50) {
            errors.push("Email must be shorter than 50 characters!");
        }
    }
    errors
}

fn validate_non_empty(text: &Option<String>) -> bool {
    match text {
        None => false,
        Some(text) => match &text as &str {
            "" => false,
            _ => true
        }
    }
}

fn validate_length(text: &String, min: usize, max: usize) -> bool {
    if text.len() < min {
        return false;
    }
    if text.len() > max {
        return false;
    }
    true
}

fn validate_alphanumeric(text: &String) -> bool {
    let re = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
    let Some(_) = re.captures(text) else {
        return false;
    };
    true
}

fn validate_same(text: String, text2: String) -> bool {
    text == text2
}

#[derive(Template)]
#[template(path = "errors.html")]
#[allow(dead_code)]
pub struct ErrorsTemplate {
    errors: Vec<&'static str>
}

#[derive(Template)]
#[template(path = "password-validation.html")]
#[allow(dead_code)]
pub struct FieldTemplate {
    value: String,
    error: bool,
    placeholder: &'static str,
    name: &'static str,
    text: &'static str,
    form_type: &'static str,
}

#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    username: Option<String>,
    psw: Option<String>,
    psw_repeat: Option<String>,
    email: Option<String>,
}

async fn register_user(
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
            .bind(user.username.unwrap())
            .bind(user.email.unwrap())
            .bind(password.unwrap())
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

    let mut headers = HeaderMap::new();
    headers.insert("HX-redirect", "/user".parse().unwrap());
    (headers, "Success").into_response()
}

async fn register_form() -> impl IntoResponse {
    info!("register form requested");
    let template = RegisterTemplate {path: "register"};
    return HtmlTemplate(template)
}

async fn user() -> impl IntoResponse {
    info!("user index requested");
    let template = UserTemplate {path: "user"};
    return HtmlTemplate(template)
}

async fn check_password(Form(user): Form<UserRequest>) -> impl IntoResponse {
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

async fn check_username(Form(user): Form<UserRequest>) -> impl IntoResponse {
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

async fn check_email(Form(user): Form<UserRequest>) -> impl IntoResponse {
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

async fn check_password_repeat(Form(user): Form<UserRequest>) -> impl IntoResponse {
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

#[cfg(test)]
mod tests {
    use crate::{validate_username, validate_password, validate_repeated_password, validate_email};

    // Validating username

    #[test]
    fn test_validating_username_that_is_too_long() {
        let username = "username_that_is_too_long";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_too_short() {
        let username = "usr";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_none() {
        let result = validate_username(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_empty() {
        let username = "";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_with_dash() {
        let username = "user-name";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("only letters"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_with_at() {
        let username = "user@name";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("only letters"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_username() {
        let username = "username";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() == 0);
    }

    // Validating password

    #[test]
    fn test_validating_password_that_is_too_long() {
        let password = "password_that_is_too_long";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_too_short() {
        let password = "usr";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_none() {
        let result = validate_password(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_empty() {
        let password = "";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_password() {
        let password = "password";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() == 0);
    }

    // Validating repeated password

    #[test]
    fn test_validating_repeated_password_that_is_none() {
        let result = validate_repeated_password(&None, &None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_repeated_password_that_is_empty() {
        let password = "";
        let result = validate_repeated_password(&None, &Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_unmatched_passwords() {
        let password = "password";
        let password2 = "password2";
        let result = validate_repeated_password(&Some(String::from(password2)), &Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("match"));
        assert!(message)
    }

    #[test]
    fn test_validating_matching_passwords() {
        let password = "password";
        let password2 = "password";
        let result = validate_repeated_password(&Some(String::from(password2)), &Some(String::from(password)));
        assert!(result.len() == 0);
    }

    // Validating email

    #[test]
    fn test_validating_email_that_is_none() {
        let result = validate_email(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_email_that_is_empty() {
        let email = "";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_email_that_is_too_long() {
        let email = "veryveryveryveryveryveryveryveryveryverylong@email.com";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("shorter"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_email() {
        let email = "email@email.com";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() == 0);
    }
}
