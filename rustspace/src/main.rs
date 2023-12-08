use axum::{routing::{get, post}, Router};
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;
use askama::Template;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::postgres::{PgPool, PgPoolOptions};
use regex::Regex;
use std::sync::Arc;
use axum::extract::State;
 

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
    errors: Vec<&'static str>
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

fn validate_user(name: Option<String>, email: Option<String>, password: Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(name.clone()) {
        errors.push("Username cannot be empty!");
    }
    if let Some(name) = name {
        if !validate_length(name.clone(), 4, 20) {
            errors.push("Username must have length between 4 and 20 characters!");
        }
        if !validate_alphanumeric(name.clone()) {
            errors.push("Username must contain only letters, numbers, or the underscore!");
        }
    }

    if !validate_non_empty(email.clone()) {
        errors.push("Email cannot be empty!");
    }
    if let Some(email) = email {
        if !validate_length(email.clone(), 0, 50) {
            errors.push("Email must be shorter than 50 characters!");
        }
    }

    if !validate_non_empty(password.clone()) {
        errors.push("Password cannot be empty!");
    }
    if let Some(password) = password {
        if !validate_length(password.clone(), 4, 20) {
            errors.push("Password must have length between 4 and 20 characters!");
        }
    }
    return errors;
}

fn validate_non_empty(text: Option<String>) -> bool {
    if text.is_none() {
        return false;
    }
    let text = text.unwrap();
    if text == "" {
        return false;
    }
    true
}

fn validate_length(text: String, min: usize, max: usize) -> bool {
    if text.len() < min {
        return false;
    }
    if text.len() > max {
        return false;
    }
    true
}

fn validate_alphanumeric(text: String) -> bool {
    let re = Regex::new(r"^([A-Za-z0-9_]+$").unwrap();
    let Some(_) = re.captures(&text) else {
        return false;
    };
    true
}


#[allow(dead_code)]
async fn register_user(
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let name = Some(String::from("aaa"));
    let email = Some(String::from("aaa"));
    let password = Some(String::from("aaa"));
    let mut errors = validate_user(name.clone(), email.clone(), password.clone());
    if errors.len() > 0 {
        let template = RegisterTemplate {path: "register", errors};
        return HtmlTemplate(template)
    }
    let query_result =
        sqlx::query(r#"INSERT INTO users (screen_name, email, password) VALUES (?, ?, ?)"#)
            .bind(name.unwrap())
            .bind(email.unwrap())
            .bind(password.unwrap())
            .execute(&state.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") && err.contains("screen_name") {
            errors.push("Username must be unique!");
        }
        if err.contains("Duplicate entry") && err.contains("email") {
            errors.push("Email must be unique!");
        }
        let template = RegisterTemplate {path: "register", errors};
        return HtmlTemplate(template)
    }
   let template = RegisterTemplate {path: "register", errors: vec![]};
   return HtmlTemplate(template)
}

async fn register_form() -> impl IntoResponse {
   info!("register form requested");
   let template = RegisterTemplate {path: "help", errors: vec![]};
   return HtmlTemplate(template)
}
