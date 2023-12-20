use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};

pub fn get_token(username: &Option<String>) -> String {
    let username = match &username {
        Some(username) => username.clone(),
        None => String::from("")
    };
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: String::from(&username),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    );

    match token {
        Ok(token) => token,
        Err(_) => String::from("")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}
