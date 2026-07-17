use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::*;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/auth/register", axum::routing::post(register))
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/refresh", axum::routing::post(refresh_token))
        .route("/api/auth/logout", axum::routing::post(logout))
        .route("/api/users", axum::routing::get(get_users))
        .route("/api/users/:id", axum::routing::get(get_user))
        .layer(cors)
}