use crate::routes::create_router;
use crate::repository::McpServerRepository;
use crate::sse_client::MCP_CLIENT_REGISTRY;
use axum::serve;
use dotenv::dotenv;
use std::env;
use tracing_subscriber;
use tokio::net::TcpListener;

mod routes;
mod handlers;
mod tools;
mod repository;
mod sse_client;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8085".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let repo = McpServerRepository::new();
    match repo.get_enabled_servers().await {
        Ok(servers) => {
            for server in servers {
                MCP_CLIENT_REGISTRY.register_server(server).await;
            }
            tracing::info!("Loaded MCP servers from database");
        }
        Err(e) => {
            tracing::error!("Failed to load MCP servers: {}", e);
        }
    }

    let app = create_router();

    tracing::info!("MCP service listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
