use axum::{extract::Path, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{User, AuthResponse, LoginRequest, RegisterRequest};
use shared::utils::{Claims, generate_jwt, generate_refresh_token};
use crate::repository::UserRepository;
use std::env;

pub async fn register(Json(req): Json<RegisterRequest>) -> ServiceResult<Json<AuthResponse>> {
    let repo = UserRepository::new();
    
    if repo.get_user_by_email(&req.email).await?.is_some() {
        return Err(ServiceError::Conflict("Email already registered".to_string()));
    }
    
    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|e| ServiceError::InternalError)?;
    
    let user = repo.create_user(&req.email, &password_hash).await?;
    
    let name = req.email.split('@').next().unwrap_or("用户");
    let _personnel = repo.create_personnel(user.id, name, &req.email).await?;
    
    let access_token = generate_jwt(user.id)
        .map_err(|_| ServiceError::InternalError)?;
    let refresh_token = generate_refresh_token(user.id)
        .map_err(|_| ServiceError::InternalError)?;
    
    let mut redis_conn = get_redis_connection().await?;
    let _: () = redis_conn.set_ex(format!("refresh_token:{}", user.id), &refresh_token, 60 * 60 * 24 * 7).await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

pub async fn login(Json(req): Json<LoginRequest>) -> ServiceResult<Json<AuthResponse>> {
    let repo = UserRepository::new();
    
    let user = repo.get_user_by_email(&req.email).await?
        .ok_or(ServiceError::Unauthorized)?;
    
    if !verify(&req.password, &user.password_hash).map_err(|_| ServiceError::InternalError)? {
        return Err(ServiceError::Unauthorized);
    }
    
    if let Ok(Some(personnel)) = repo.get_personnel_by_user_id(user.id).await {
        let _ = repo.update_last_login_date(personnel.id).await;
    }
    
    let access_token = generate_jwt(user.id)
        .map_err(|_| ServiceError::InternalError)?;
    let refresh_token = generate_refresh_token(user.id)
        .map_err(|_| ServiceError::InternalError)?;
    
    let mut redis_conn = get_redis_connection().await?;
    let _: () = redis_conn.set_ex(format!("refresh_token:{}", user.id), &refresh_token, 60 * 60 * 24 * 7).await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

pub async fn refresh_token(Json(req): Json<RefreshTokenRequest>) -> ServiceResult<Json<AuthResponse>> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    
    let decoded = decode::<Claims>(
        &req.refresh_token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| ServiceError::InvalidToken)?;
    
    let user_id = decoded.claims.user_id;
    
    let mut redis_conn = get_redis_connection().await?;
    let stored_token: Option<String> = redis_conn.get(format!("refresh_token:{}", user_id)).await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    if stored_token != Some(req.refresh_token) {
        return Err(ServiceError::InvalidToken);
    }
    
    let access_token = generate_jwt(user_id)
        .map_err(|_| ServiceError::InternalError)?;
    let new_refresh_token = generate_refresh_token(user_id)
        .map_err(|_| ServiceError::InternalError)?;
    
    redis_conn.set_ex::<_, _, ()>(format!("refresh_token:{}", user_id), &new_refresh_token, 60 * 60 * 24 * 7).await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        user_id,
    }))
}

pub async fn logout(Json(req): Json<LogoutRequest>) -> ServiceResult<Json<()>> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    
    let decoded = decode::<Claims>(
        &req.access_token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| ServiceError::InvalidToken)?;
    
    let mut redis_conn = get_redis_connection().await?;
    let _: () = redis_conn.del(format!("refresh_token:{}", decoded.claims.user_id)).await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    Ok(Json(()))
}

pub async fn get_users() -> ServiceResult<Json<Vec<User>>> {
    let repo = UserRepository::new();
    let users = repo.get_all_users().await?;
    Ok(Json(users))
}

pub async fn get_user(Path(id): Path<i64>) -> ServiceResult<Json<User>> {
    let repo = UserRepository::new();
    let user = repo.get_user(id).await?;
    Ok(Json(user))
}

async fn get_redis_connection() -> ServiceResult<redis::aio::Connection> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());
    let client = redis::Client::open(redis_url).map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    client.get_async_connection().await.map_err(|e| ServiceError::DatabaseError(e.to_string()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub access_token: String,
}