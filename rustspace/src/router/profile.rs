use axum::{response::IntoResponse, extract::Path};
use tracing::info;

use crate::{template::{ProfileTemplate, HtmlTemplate}, UserData};

pub async fn profile(
    user: UserData,
    Path(username): Path<String>
    ) -> impl IntoResponse {
   info!("profile of user {} requested", username);
   let template = ProfileTemplate {path: "index", user, username};
   return HtmlTemplate(template)
}
