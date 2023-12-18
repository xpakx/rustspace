use axum::response::IntoResponse;
use tracing::info;

use crate::template::{RootTemplate, AboutTemplate, HelpTemplate, HtmlTemplate};

pub async fn root() -> impl IntoResponse {
   info!("index requested");
   let template = RootTemplate {path: "index"};
   return HtmlTemplate(template)
}

pub async fn about() -> impl IntoResponse {
   info!("about requested");
   let template = AboutTemplate {path: "about"};
   return HtmlTemplate(template)
}

pub async fn help() -> impl IntoResponse {
   info!("help requested");
   let template = HelpTemplate {path: "help"};
   return HtmlTemplate(template)
}

