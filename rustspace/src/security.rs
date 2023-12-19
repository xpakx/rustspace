// TODO: add JWT
pub fn get_token(username: &Option<String>) -> String {
    match &username {
        Some(username) => username.clone(),
        None => String::from("")
    }
}
