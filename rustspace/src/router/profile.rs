use std::sync::Arc;

use axum::{response::IntoResponse, extract::{Path, State}, Form};
use sqlx::Postgres;
use tracing::{info, debug};

use crate::{template::{ProfileTemplate, HtmlTemplate, UserNotFoundTemplate, ProfileFormTemplate, ErrorsTemplate, ProfileFieldTemplate, FriendStatus}, UserData, UserModel, AppState, ProfileModel, ProfileRequest, FriendshipModel};

pub async fn profile(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>
    ) -> impl IntoResponse {
   info!("profile of user {} requested", username);

    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&username)
        .fetch_optional(&state.db)
        .await;
    let owner = match &user.username {
        None => false,
        Some(current_user) => current_user == &username
    };

    let Ok(Some(user_db)) = user_db else {
        let template = UserNotFoundTemplate {};
        return HtmlTemplate(template).into_response()
    };

    let avatar = match user_db.avatar {
        Some(true) => true,
        _ => false
    };

    let timestamp: i64 = match &user_db.updated_at {
        Some(time) => time.timestamp(),
        None => 0
    };

    let Some(user_id) = user_db.id else {
        let template = ProfileTemplate {path: "profile", user, username, profile: None, owner, avatar, timestamp, friend: FriendStatus::NotFriend};
        return HtmlTemplate(template).into_response()
    };

    let current_db = match &user.username {
        Some(username) => sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(username)
        .fetch_optional(&state.db)
        .await,
        None => Ok(None)
    };
    let current_id = match current_db {
        Ok(Some(curr)) if curr.id.is_some()=> curr.id,
        _ => None
    };

    let friend = match current_id {
        _ if owner => FriendStatus::User,
        None => FriendStatus::NotFriend,
        Some(current_id) => {
            let friendship = sqlx::query_as::<Postgres, FriendshipModel>(
                "SELECT * FROM friendships WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
                )
                .bind(&current_id)
                .bind(&user_id)
                .fetch_optional(&state.db)
                .await;

            match friendship {
                Ok(Some(accepted)) if accepted.accepted =>  FriendStatus::Friend,
                Ok(Some(rejected)) if rejected.rejected =>  FriendStatus::Rejector,
                Ok(Some(_)) =>  FriendStatus::Invitee,
                Ok(None) => FriendStatus::NotFriend,
                Err(_) => FriendStatus::NotFriend
            }
        }
    };
        
    let profile = sqlx::query_as::<Postgres, ProfileModel>(
        "SELECT * FROM profiles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;
    

    let Ok(profile) = profile else {
        let template = ProfileTemplate {path: "profile", user, username, profile: None, owner, avatar, timestamp, friend};
        return HtmlTemplate(template).into_response()
    };

   let template = ProfileTemplate {path: "profile", user, username, profile, owner, avatar, timestamp, friend};
   return HtmlTemplate(template).into_response()
}

pub async fn edit_profile(user: UserData,
    State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("profile form requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    debug!("getting user from database");
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&user.username.unwrap())
        .fetch_optional(&state.db)
        .await;

    let mut gender = None;
    let mut city = None;
    let mut description = None;
    let mut real_name = None;

    if let Ok(Some(user_db)) = user_db {
        if let Some(user_id) = user_db.id {
            debug!("getting user's profile from db");
            let profile_db = sqlx::query_as::<Postgres, ProfileModel>("SELECT * FROM profiles WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(&state.db)
                .await;
            if let Ok(Some(profile_db)) = profile_db  {
                gender = profile_db.gender;
                city = profile_db.city;
                description = profile_db.description;
                real_name = profile_db.real_name;
            }
        }
    }

    let template = ProfileFormTemplate {gender, city, description, real_name};
    return HtmlTemplate(template).into_response()
}

pub async fn update_profile(
    user: UserData,
    State(state): State<Arc<AppState>>,
    Form(request): Form<ProfileRequest>) -> impl IntoResponse {
    info!("profile update requested");
    if user.username.is_none() {
        let template = ErrorsTemplate {errors: vec!["Unauthenticated!"]};
        return HtmlTemplate(template).into_response()
    }

    debug!("getting user from database");
    let user_db = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM users WHERE screen_name = $1",
        )
        .bind(&user.username.unwrap())
        .fetch_optional(&state.db)
        .await;

    let Ok(Some(user_db)) = user_db else {
        debug!("couldn't get user from database");
        let template = ErrorsTemplate {errors: vec!["There was a problem with database!"]};
        return HtmlTemplate(template).into_response()
    };
    let Some(user_id) = user_db.id else {
        debug!("user has not id!");
        let template = ErrorsTemplate {errors: vec!["There was a problem with database!"]};
        return HtmlTemplate(template).into_response()
    };

    debug!("getting user's profile from db");
    let profile = sqlx::query_as::<Postgres, ProfileModel>(
        "SELECT * FROM profiles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;
    let Ok(profile) = profile else {
        debug!("errors while getting profile");
        let template = ErrorsTemplate {errors: vec!["There was a problem with database!"]};
        return HtmlTemplate(template).into_response()
    };

    let gender = clear_empty(request.gender);
    let city = clear_empty(request.city);
    let description = clear_empty(request.description);
    let name = clear_empty(request.name);
    
    let Some(_) = profile else {
        debug!("profile doesn't exists, creating new one");
        let query_result =
            sqlx::query("INSERT INTO profiles (gender, city, description, real_name, user_id) VALUES ($1, $2, $3, $4, $5)")
            .bind(&gender)
            .bind(&city)
            .bind(&description)
            .bind(&name)
            .bind(user_id)
            .execute(&state.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

        if let Err(_) = query_result {
            let template = ErrorsTemplate {errors: vec!["Couldn't create profile!"]};
            return HtmlTemplate(template).into_response()
        }
        info!("profile succesfully created.");
        let profile_some = gender.is_some() || city.is_some() || description.is_some() || name.is_some();
        let template = ProfileFieldTemplate {profile: profile_some, gender, city, description, real_name: name};
        return HtmlTemplate(template).into_response()

    };

    debug!("profile already exists, updating");
    let result = sqlx::query("UPDATE profiles SET gender = $1, city = $2, description = $3, real_name = $4 WHERE user_id = $5")
        .bind(&gender)
        .bind(&city)
        .bind(&description)
        .bind(&name)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());
    if let Ok(_) = result {
        info!("profile succesfully updated.");
        let profile_some = gender.is_some() || city.is_some() || description.is_some() || name.is_some();
        let template = ProfileFieldTemplate {profile: profile_some, gender, city, description, real_name: name};
        return HtmlTemplate(template).into_response()
    } else {
        debug!("password change unsuccessful due to db error");
        let template = ErrorsTemplate {errors: vec!["Database error, please try again later"]};
        return HtmlTemplate(template).into_response()
    }
}

fn clear_empty(field: Option<String>) -> Option<String> {
    match field {
        None => None,
        Some(field) if field == "" => None,
        Some(field) => Some(field)
    }
}
