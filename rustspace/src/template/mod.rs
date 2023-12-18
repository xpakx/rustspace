use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Template)]
#[template(path = "index.html")]
pub struct RootTemplate {
    pub path: &'static str
}

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub path: &'static str
}

#[derive(Template)]
#[template(path = "help.html")]
pub struct HelpTemplate {
    pub path: &'static str
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub path: &'static str,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate {
    pub path: &'static str,
}

#[derive(Template)]
#[template(path = "errors.html")]
pub struct ErrorsTemplate {
    pub errors: Vec<&'static str>
}

#[derive(Template)]
#[template(path = "password-validation.html")]
pub struct FieldTemplate {
    pub value: String,
    pub error: bool,
    pub placeholder: &'static str,
    pub name: &'static str,
    pub text: &'static str,
    pub form_type: &'static str,
}

pub struct HtmlTemplate<T>(pub T);

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
