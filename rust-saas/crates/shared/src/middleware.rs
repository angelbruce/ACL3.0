use axum::{
    extract::FromRequestParts,
    body::Body,
    http::{request::Parts, Request, Response},
    middleware::Next,
};
use jsonwebtoken::errors::Error as JwtError;
use crate::errors::ServiceError;
use crate::utils::{get_jwt_secret, Claims};

pub async fn auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, ServiceError> {
    if req.method() == http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }
    
    let auth_header = req.headers().get("Authorization");
    
    let token = match auth_header {
        Some(header) => {
            let header_str = header.to_str().map_err(|_| ServiceError::Unauthorized)?;
            header_str.strip_prefix("Bearer ").ok_or(ServiceError::Unauthorized)?
        }
        None => return Err(ServiceError::Unauthorized),
    };

    let claims = validate_jwt(token).map_err(|_| ServiceError::Unauthorized)?;
    
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}

fn validate_jwt(token: &str) -> Result<Claims, JwtError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    
    let secret = get_jwt_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ServiceError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or(ServiceError::Unauthorized)?
            .clone();
        Ok(claims)
    }
}
