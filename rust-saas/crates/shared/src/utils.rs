use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i64,
    pub exp: usize,
}

pub fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key-change-in-production".to_string())
}

pub fn generate_jwt(user_id: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let expiration = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    
    let claims = Claims { user_id, exp: expiration };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn generate_refresh_token(user_id: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let expiration = (Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
    
    let claims = Claims { user_id, exp: expiration };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

pub fn get_current_time() -> DateTime<Utc> {
    Utc::now()
}

pub fn parse_json<T: for<'a> Deserialize<'a>>(json_str: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json_str)
}

pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}