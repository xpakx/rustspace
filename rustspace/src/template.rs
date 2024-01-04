use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;

use crate::{UserData, UserModel, ProfileModel};

#[derive(Template)]
#[template(path = "index.html")]
pub struct RootTemplate {
    pub path: &'static str,
    pub user: UserData
}

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub path: &'static str,
    pub user: UserData
}

#[derive(Template)]
#[template(path = "help.html")]
pub struct HelpTemplate {
    pub path: &'static str,
    pub user: UserData
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub redir: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub redir: Option<String>,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub user_db: UserModel
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

#[derive(Template)]
#[template(path = "unauthorized.html")]
pub struct UnauthorizedTemplate {
    pub message: &'static str,
    pub redir: Option<String>,
}

#[derive(Template)]
#[template(path = "email-form.html")]
pub struct EmailFormTemplate {
}

#[derive(Template)]
#[template(path = "email-field.html")]
pub struct EmailFieldTemplate {
    pub email: String,
}

#[derive(Template)]
#[template(path = "password-form.html")]
pub struct PasswordFormTemplate {
}

#[derive(Template)]
#[template(path = "password-field.html")]
pub struct PasswordFieldTemplate {
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub username: String,
    pub profile: Option<ProfileModel>,
    pub owner: bool
}

#[derive(Template)]
#[template(path = "profile_error.html")]
pub struct UserNotFoundTemplate {
}

#[derive(Template)]
#[template(path = "profile-form.html")]
pub struct ProfileFormTemplate {
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
