use axum::{routing::get, Router};
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;
use askama::Template;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let assets_path = std::env::current_dir().unwrap();
    let app = Router::new()
        .route("/", get(root))
        .route("/index", get(root))
        .route("/about", get(about))
        .route("/help", get(help))
        .nest_service("/assets", ServeDir::new(format!("{}/assets/", assets_path.to_str().unwrap())));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
   let template = RootTemplate {};
   return HtmlTemplate(template)
}

async fn about() -> impl IntoResponse {
   let template = AboutTemplate {};
   return HtmlTemplate(template)
}

async fn help() -> impl IntoResponse {
   let template = HelpTemplate {};
   return HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct RootTemplate {}

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {}

#[derive(Template)]
#[template(path = "help.html")]
pub struct HelpTemplate {}

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
