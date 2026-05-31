use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::*;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/sessions", axum::routing::get(get_sessions))
        .route("/api/sessions/:id", axum::routing::get(get_session))
        .route("/api/sessions", axum::routing::post(create_session))
        .route("/api/sessions/:id", axum::routing::put(update_session))
        .route("/api/sessions/:id", axum::routing::delete(delete_session))
        .route("/api/sessions/:id/messages", axum::routing::get(get_session_messages))
        .route("/api/sessions/:id/messages", axum::routing::post(add_message))
        .layer(cors)
}