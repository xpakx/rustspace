use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode};
use sqlx::{Postgres, PgPool, Row};
use tower::ServiceExt;
use serial_test::serial;

use crate::{test::{prepare_server_with_user, prepare_db, prepare_server_with_db, insert_new_user, clear_friendships, insert_users, insert_requests}, security::get_token, UserModel, FriendshipModel};

#[tokio::test]
#[serial]
async fn test_making_friendship_request_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Unauthenticated"));
}

#[tokio::test]
#[serial]
async fn test_making_friendship_request_with_no_name() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Username"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_making_friendship_request_with_empty_name() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username="))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Username"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_user_trying_to_befriend_themself() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=Test"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("befriend yourself"));
}

#[tokio::test]
#[serial]
async fn test_user_trying_to_befriend_nonexistent_user() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("not found"));
}

async fn insert_friendship(username: &str, username2: &str, accepted: bool, rejected: bool, db: &PgPool) {
    let user_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username)
        .fetch_optional(db)
        .await;
    let Ok(Some(user)) = user_db else {
        panic!("No such user!");
    };
    let friend_db = sqlx::query_as::<Postgres, UserModel>("SELECT * FROM users WHERE screen_name = $1")
        .bind(&username2)
        .fetch_optional(db)
        .await;
    let Ok(Some(friend)) = friend_db else {
        panic!("No such user!");
    };

    _ = sqlx::query("INSERT INTO friendships (user_id, friend_id, accepted, rejected) VALUES ($1, $2, $3, $4)")
        .bind(&user.id)
        .bind(&friend.id)
        .bind(&accepted)
        .bind(&rejected)
        .execute(db)
        .await;
}


async fn insert_friendship_request(username: &str, username2: &str, db: &PgPool) {
    insert_friendship(username, username2, false, false, db).await;
}

#[tokio::test]
#[serial]
async fn test_making_request_while_its_already_created() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_friendship_request("Test", "User", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("already created"));
    clear_friendships(&db).await;
}

#[tokio::test]
#[serial]
async fn test_making_request_while_reverse_one_is_already_created() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_friendship_request("User", "Test", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("already created"));
    clear_friendships(&db).await;
}

#[tokio::test]
#[serial]
async fn test_making_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Cookie", format!("Token={};", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .uri("/friendships")
            .body(Body::from("username=User"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result = sqlx::query("SELECT COUNT(*) FROM friendships")
        .fetch_one(&db)
        .await;
    clear_friendships(&db).await;

    assert!(result.is_ok());
    if let Ok(result) = result {
        assert_eq!(result.get::<i64, _>(0), 1);
    }

}

// friendship requests view

#[tokio::test]
#[serial]
async fn test_getting_friendship_requests_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/friends/requests")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Authentication Error"));
}

#[tokio::test]
#[serial]
async fn test_if_friendship_requests_endpoint_exists() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Cookie", format!("Token={};", token))
            .uri("/friends/requests")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Requests"));
}

#[tokio::test]
#[serial]
async fn test_getting_friendship_requests() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;

    insert_new_user("User", "user1@mail.com", &db).await;
    insert_friendship_request("User", "Test", &db).await;
    insert_new_user("User2", "user2@mail.com", &db).await;
    insert_friendship_request("User2", "Test", &db).await;

    // accepted
    insert_new_user("User3", "user3@mail.com", &db).await;
    insert_friendship("User3", "Test", true, false, &db).await;
    // rejected
    insert_new_user("User4", "user4@mail.com", &db).await;
    insert_friendship("User4", "Test", false, true, &db).await;
    // reverse
    insert_new_user("User5", "user5@mail.com", &db).await;
    insert_friendship_request("Test", "User5", &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Cookie", format!("Token={};", token))
            .uri("/friends/requests")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;

    let body = to_bytes(response.into_body(), 9000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("User"));
    assert!(content.contains("User2"));
    assert!(!content.contains("User3"));
    assert!(!content.contains("User4"));
    assert!(!content.contains("User5"));
}

#[tokio::test]
#[serial]
async fn test_pagination_on_friend_requests_view() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_users(300, "user", &db).await;
    insert_requests(false, false, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/friends/requests")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 20000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    println!("{}", content);
    assert!(content.contains("300 requests found"));
    assert!(content.contains("\"current\">0<"));
    assert!(content.contains("page=1\""));
    assert!(!content.contains("page=2\""));
    assert!(!content.contains("page=10"));
    assert!(content.contains("page=11"));
}

// friends view

#[tokio::test]
#[serial]
async fn test_getting_friends_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/friends")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Authentication Error"));
}

#[tokio::test]
#[serial]
async fn test_if_friends_endpoint_exists() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Cookie", format!("Token={};", token))
            .uri("/friends")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Friends"));
}

#[tokio::test]
#[serial]
async fn test_getting_friends() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;

    insert_new_user("User", "user1@mail.com", &db).await;
    insert_friendship("User", "Test", true, false, &db).await;
    insert_new_user("User2", "user2@mail.com", &db).await;
    insert_friendship("User2", "Test", true, false, &db).await;

    // not accepted
    insert_new_user("User3", "user3@mail.com", &db).await;
    insert_friendship_request("User3", "Test", &db).await;
    // rejected
    insert_new_user("User4", "user4@mail.com", &db).await;
    insert_friendship("User4", "Test", false, true, &db).await;
    // reverse
    insert_new_user("User5", "user5@mail.com", &db).await;
    insert_friendship("Test", "User5", true, false, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .header("Cookie", format!("Token={};", token))
            .uri("/friends")
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;

    let body = to_bytes(response.into_body(), 9000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("User"));
    assert!(content.contains("User2"));
    assert!(!content.contains("User3"));
    assert!(!content.contains("User4"));
    assert!(content.contains("User5"));
}

#[tokio::test]
#[serial]
async fn test_pagination_on_friend_view() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_users(300, "user", &db).await;
    insert_requests(true, false, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/friends")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 20000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    println!("{}", content);
    assert!(content.contains("300 requests found"));
    assert!(content.contains("\"current\">0<"));
    assert!(content.contains("page=1\""));
    assert!(!content.contains("page=2\""));
    assert!(!content.contains("page=10"));
    assert!(content.contains("page=11"));
}

// changing request's state

#[tokio::test]
#[serial]
async fn test_accepting_request_by_unauthenticated_user() {
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri("/friends/requests/1")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from("state=accepted"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Unauthenticated"));
}

#[tokio::test]
#[serial]
async fn test_changing_request_state_without_specyfying_state() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri("/friends/requests/1")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("State"));
    assert!(content.contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_changing_request_state_with_wrong_state() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri("/friends/requests/1")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::from("state=state"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("Unsupported state"));
}

#[tokio::test]
#[serial]
async fn test_accepting_nonexistent_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let response = prepare_server_with_user(false)
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri("/friends/requests/1")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::from("state=accepted"))
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("error"));
    assert!(content.contains("No such request"));
}

#[tokio::test]
#[serial]
async fn test_accepting_users_own_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;

    insert_new_user("User", "user1@mail.com", &db).await;
    insert_friendship("Test", "User", false, false, &db).await;

    let request_db = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request)) = request_db else {
        panic!("No request in db!");
    };
    let Some(request_id) = request.id else {
        panic!("No request id!");
    };

    _ = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri(format!("/friends/requests/{}", request_id))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::from("state=accepted"))
            .unwrap()
            )
        .await
        .unwrap();

    let request_updated = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request_updated)) = request_updated else {
        panic!("No request in db!");
    };

    clear_friendships(&db).await;
    assert!(!request_updated.accepted);
}

#[tokio::test]
#[serial]
async fn test_accepting_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_friendship("User", "Test", true, false, &db).await;

    let request_db = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request)) = request_db else {
        panic!("No request in db!");
    };
    let Some(request_id) = request.id else {
        panic!("No request id!");
    };

    _ = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri(format!("/friends/requests/{}", request_id))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::from("state=accepted"))
            .unwrap()
            )
        .await
        .unwrap();

    let request_updated = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request_updated)) = request_updated else {
        panic!("No request in db!");
    };

    clear_friendships(&db).await;
    assert!(request_updated.accepted);
    assert!(!request_updated.rejected);
}

#[tokio::test]
#[serial]
async fn test_rejecting_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "Test@mail.com", &db).await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_friendship("User", "Test", true, false, &db).await;

    let request_db = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request)) = request_db else {
        panic!("No request in db!");
    };
    let Some(request_id) = request.id else {
        panic!("No request id!");
    };

    _ = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("POST")
            .uri(format!("/friends/requests/{}", request_id))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::from("state=rejected"))
            .unwrap()
            )
        .await
        .unwrap();

    let request_updated = sqlx::query_as::<Postgres, FriendshipModel>("SELECT * FROM friendships LIMIT 1")
        .fetch_optional(&db)
        .await;
    let Ok(Some(request_updated)) = request_updated else {
        panic!("No request in db!");
    };

    clear_friendships(&db).await;
    assert!(!request_updated.accepted);
    assert!(request_updated.rejected);
}

// friendship state on the profile page

#[tokio::test]
#[serial]
async fn test_if_friendship_button_is_present() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/profile/User")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Send friend request"));
}

#[tokio::test]
#[serial]
async fn test_friendship_button_if_there_is_new_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_friendship("Test", "User", false, false, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/profile/User")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Invited to friends"));
}

#[tokio::test]
#[serial]
async fn test_friendship_button_if_there_is_accepted_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_friendship("Test", "User", true, false, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/profile/User")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("Friend"));
}

#[tokio::test]
#[serial]
async fn test_friendship_button_if_there_is_rejected_request() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("User", "user1@mail.com", &db).await;
    insert_new_user("Test", "test@mail.com", &db).await;
    insert_friendship("Test", "User", false, true, &db).await;

    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/profile/User")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("User rejected"));
}

#[tokio::test]
#[serial]
async fn test_friendship_button_presence_while_visiting_self_profile() {
    let (token, _) = get_token(&Some(String::from("Test")));
    let db = prepare_db().await;
    insert_new_user("Test", "test@mail.com", &db).await;
    let response = prepare_server_with_db(db.clone())
        .await
        .oneshot(
            Request::builder()
            .method("GET")
            .uri("/profile/Test")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("Token={};", token))
            .body(Body::empty())
            .unwrap()
            )
        .await
        .unwrap();

    clear_friendships(&db).await;
    let body = to_bytes(response.into_body(), 2000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(!content.contains("friend-btn"));
}
