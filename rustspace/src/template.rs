use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;

use crate::{UserData, UserModel, ProfileModel, UserDetails, FriendshipDetails, BlogPostModel, BlogPostDetails, BlogCommentModel, BlogCommentDetails};

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
    pub user_db: UserModel,
    pub timestamp: i64,
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
    pub owner: bool,
    pub avatar: bool,
    pub timestamp: i64,
    pub friend: FriendStatus,
    pub friend_id: Option<i32>,
}

#[derive(PartialEq,Eq,PartialOrd,Ord)]
pub enum FriendStatus {
    User,
    Friend,
    Invitee,
    Rejector,
    NotFriend,
    Cancelled
}

#[derive(Template)]
#[template(path = "profile_error.html")]
pub struct UserNotFoundTemplate {
}

#[derive(Template)]
#[template(path = "profile-form.html")]
pub struct ProfileFormTemplate {
    pub gender: Option<String>,
    pub city: Option<String>,
    pub description: Option<String>,
    pub real_name: Option<String>,
}

#[derive(Template)]
#[template(path = "profile_field.html")]
pub struct ProfileFieldTemplate {
    pub profile: bool,
    pub gender: Option<String>,
    pub city: Option<String>,
    pub description: Option<String>,
    pub real_name: Option<String>,
}

#[derive(Template)]
#[template(path = "community.html")]
pub struct CommunityTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub users: Vec<UserDetails>,
    pub records: Option<i64>,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "community_result.html")]
pub struct CommunityResultsTemplate {
    pub users: Vec<UserDetails>,
    pub records: Option<i64>,
    pub page: i32,
    pub pages: i32,
    pub query: String,
    pub search_path: &'static str,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub path: &'static str,
    pub user: UserData,
}

#[derive(Template)]
#[template(path = "avatar-form.html")]
pub struct AvatarFormTemplate {
}

#[derive(Template)]
#[template(path = "avatar-result.html")]
pub struct AvatarResultTemplate {
    pub avatar: bool,
    pub username: String,
    pub timestamp: i64,
}

#[derive(Template)]
#[template(path = "friend-requests.html")]
pub struct FriendRequestsTemplate {
    pub path: &'static str,
    pub user: UserData,

    pub friends: Vec<FriendshipDetails>,
    pub records: Option<i64>,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "friends.html")]
pub struct FriendsTemplate {
    pub path: &'static str,
    pub user: UserData,

    pub friends: Vec<FriendshipDetails>,
    pub records: Option<i64>,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "friend-requests-result.html")]
pub struct FriendRequestsResultsTemplate {
    pub friends: Vec<FriendshipDetails>,
    pub page: i32,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "invited-button.html")]
pub struct InvitedTemplate {}

#[derive(Template)]
#[template(path = "rejected-requests.html")]
pub struct RejectedFriendRequestsTemplate {
    pub path: &'static str,
    pub user: UserData,

    pub friends: Vec<FriendshipDetails>,
    pub records: Option<i64>,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "rejected-requests-result.html")]
pub struct RejectedRequestsResultsTemplate {
    pub friends: Vec<FriendshipDetails>,
    pub page: i32,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub post: BlogPostDetails,
    pub owner: bool,
}

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostsTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub username: String,
    pub posts: Vec<BlogPostModel>,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "posts-result.html")]
pub struct PostsResultTemplate {
    pub posts: Vec<BlogPostModel>,
    pub username: String,
    pub pages: i32,
    pub page: i32,
}

#[derive(Template)]
#[template(path = "post-form.html")]
pub struct PostFormTemplate {
    pub path: &'static str,
    pub user: UserData,
}

#[derive(Template)]
#[template(path = "post-error.html")]
pub struct PostNotFoundTemplate {
}

#[derive(Template)]
#[template(path = "db-error.html")]
pub struct DbErrorTemplate {
}

#[derive(Template)]
#[template(path = "new-posts.html")]
pub struct NewPostsTemplate {
    pub posts: Vec<BlogPostModel>,
    pub username: String,
}

#[derive(Template)]
#[template(path = "comments-result.html")]
pub struct CommentsTemplate {
    pub comments: Vec<BlogCommentDetails>,
    pub post_id: i32,
    pub pages: i32,
    pub page: i32,
}

#[derive(Template)]
#[template(path = "post-edit-form.html")]
pub struct UpdatePostFormTemplate {
    pub path: &'static str,
    pub user: UserData,
    pub post_id: i32,
    pub post: BlogPostModel,
}

#[derive(Template)]
#[template(path = "comment-form.html")]
pub struct CommentFormTemplate {
    pub comment_id: i32,
    pub comment: BlogCommentModel,
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
