use axum::response::IntoResponse;
use tracing::info;

use crate::{template::{RootTemplate, AboutTemplate, HelpTemplate, HtmlTemplate}, UserData};

pub async fn root(user: UserData) -> impl IntoResponse {
   info!("index requested");
   let template = RootTemplate {path: "index", user};
   return HtmlTemplate(template)
}

pub async fn about(user: UserData) -> impl IntoResponse {
   info!("about requested");
   let template = AboutTemplate {path: "about", user};
   return HtmlTemplate(template)
}

pub async fn help(user: UserData) -> impl IntoResponse {
   info!("help requested");
   let template = HelpTemplate {path: "help", user};
   return HtmlTemplate(template)
}

