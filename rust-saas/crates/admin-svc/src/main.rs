mod handlers;
mod repository;
mod routes;

use std::env;
use tower_http::cors::{Any, CorsLayer};
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::create_router().layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "3007".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Admin service listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
