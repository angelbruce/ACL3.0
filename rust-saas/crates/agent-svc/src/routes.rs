use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::*;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/agents", axum::routing::get(get_agents))
        .route("/api/agents/:id", axum::routing::get(get_agent))
        .route("/api/agents", axum::routing::post(create_agent))
        .route("/api/agents/:id", axum::routing::put(update_agent))
        .route("/api/agents/:id", axum::routing::delete(delete_agent))
        .layer(cors)
}