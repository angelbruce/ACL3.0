
use tracing_subscriber;
use std::env;

mod state;
mod routes;
mod tools_handler;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8088".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let app = routes::create_router();

    tracing::info!("MCP Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
