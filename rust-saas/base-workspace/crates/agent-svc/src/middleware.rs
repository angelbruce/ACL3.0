use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use shared::errors::ServiceError;
use shared::utils::Claims;
use std::env;

pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, ServiceError> {
    let auth_header = req.headers().get("Authorization");
    
    let token = match auth_header {
        Some(h) => h.to_str().map_err(|_| ServiceError::Unauthorized)?,
        None => return Err(ServiceError::Unauthorized),
    };

    let token = token.strip_prefix("Bearer ").ok_or(ServiceError::InvalidToken)?;

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| ServiceError::InvalidToken)?;

    req.extensions_mut().insert(decoded.claims);
    
    Ok(next.run(req).await)
}

pub async fn cors_middleware(req: Request<Body>, next: Next) -> Result<Response, ServiceError> {
    let mut response = next.run(req).await;
    
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("*"),
    );
    
    Ok(response)
}