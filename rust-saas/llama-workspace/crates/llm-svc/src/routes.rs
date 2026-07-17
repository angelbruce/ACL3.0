use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::{get_models, get_model, create_model, update_model, delete_model, chat_stream};
use shared::middleware::auth_middleware;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/models", axum::routing::get(get_models))
        .route("/api/models/:id", axum::routing::get(get_model))
        .route("/api/models", axum::routing::post(create_model))
        .route("/api/models/:id", axum::routing::put(update_model))
        .route("/api/models/:id", axum::routing::delete(delete_model))
        // .route("/api/chat", axum::routing::post(chat))
        .route("/api/chat/stream", axum::routing::post(chat_stream))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(cors)
}