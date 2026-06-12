use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::*;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/flows", axum::routing::get(get_flows))
        .route("/api/flows/:id", axum::routing::get(get_flow))
        .route("/api/flows", axum::routing::post(create_flow))
        .route("/api/flows/:id", axum::routing::put(update_flow))
        .route("/api/flows/:id", axum::routing::delete(delete_flow))
        .route("/api/flows/:id/start", axum::routing::post(start_flow))
        .route("/api/flows/:id/stop", axum::routing::post(stop_flow))
        .route("/api/flows/:id/status", axum::routing::get(get_flow_status))
        .route("/api/flows/:id/runtimes", axum::routing::get(get_flow_runtimes))
        .route("/api/flows/:id/runtime", axum::routing::get(get_flow_runtime))
        .route("/api/flows/flow/:id/runtime", axum::routing::get(get_flow_runtime_by_flow_id))
        .route("/api/flow-runtimes/:runtime_id/nodes/:node_id/complete", axum::routing::post(complete_node))
        .layer(cors)
}