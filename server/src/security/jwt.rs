use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

const JWT_SECRET: &[u8] = b"SUPER_SECRET_CHANGE_ME"; // später aus ENV!

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,
}

pub fn create_jwt(
    user_id: i64,
    email: &str,
    is_admin: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        is_admin,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}