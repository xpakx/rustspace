use axum::{routing::get, Router};
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;
use askama::Template;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::postgres::PgPoolOptions;
 

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

    info!("Initializing router...");

    let assets_path = std::env::current_dir().unwrap();
    let app = Router::new()
        .route("/", get(root))
        .route("/index", get(root))
        .route("/about", get(about))
        .route("/help", get(help))
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
