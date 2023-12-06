use axum::{routing::get, Router};
use axum::response::Html;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/index", get(root))
        .route("/about", get(about))
        .route("/help", get(help));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root() -> Html<String> {
    Html(format!("<h1>Homepage</h1>"))
}

async fn about() -> Html<String> {
    Html(format!("<h1>About us</h1>"))
}

async fn help() -> Html<String> {
    Html(format!("<h1>Help</h1>"))
}
