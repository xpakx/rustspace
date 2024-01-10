use std::sync::Arc;

use axum::{response::IntoResponse, extract::State};
use tracing::info;

use crate::{template::{CommunityTemplate, HtmlTemplate, ErrorsTemplate}, UserData, AppState};

pub async fn community(
    user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
   info!("community page requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }
    let template = CommunityTemplate {path: "/community", user};
    return HtmlTemplate(template).into_response()
}
