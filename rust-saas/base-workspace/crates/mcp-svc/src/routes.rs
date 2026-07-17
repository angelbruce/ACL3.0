use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::handlers::*;
use shared::middleware::auth_middleware;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/mcp/tools", axum::routing::get(list_tools))
        .route("/api/mcp/tools/:name", axum::routing::post(call_tool))
        .route("/api/mcp/servers", axum::routing::get(list_mcp_servers))
        .route("/api/mcp/servers", axum::routing::post(create_mcp_server))
        .route("/api/mcp/servers/refresh", axum::routing::post(refresh_mcp_servers))
        .route("/api/mcp/servers/:id", axum::routing::get(get_mcp_server))
        .route("/api/mcp/servers/:id", axum::routing::put(update_mcp_server))
        .route("/api/mcp/servers/:id", axum::routing::delete(delete_mcp_server))
        .route("/api/mcp/servers/:id/tools", axum::routing::get(get_mcp_server_tools))
        .route("/api/mcp/servers/:id/toggle/:enabled", axum::routing::post(toggle_mcp_server))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(cors)
}
