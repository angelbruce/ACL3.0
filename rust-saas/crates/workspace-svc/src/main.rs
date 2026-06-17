use crate::routes::create_router;
use axum::serve;
use dotenv::dotenv;
use std::env;
use tracing_subscriber;
use tokio::net::TcpListener;

mod routes;
mod handlers;
mod repository;
mod voice;
mod container;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8087".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = create_router();
   

    tracing::info!("Workspace service listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
