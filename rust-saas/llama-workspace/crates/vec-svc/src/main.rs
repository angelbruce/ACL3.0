use crate::routes::create_router;
use crate::app_state::AppState;
use axum::serve;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tracing_subscriber;
use tokio::net::TcpListener;

mod embedding;
mod tokenizer;
mod loader;
mod milvus;
mod app_state;
mod handlers;
mod routes;
mod schema;
mod model;
mod rdb_repository;
mod distillation;
mod minio;
mod task_queue;
mod knowledge_graph;
mod verification;
mod search_suggestions;
mod cache;
mod rerank;
mod analytics;
mod version_control;
mod import_export;
mod semantic_chunk;
mod ontology;
mod semantic_extractor;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    tracing_subscriber::fmt::init();
    
    // Load application state
    let app_state = match AppState::new().await {
        Ok(state) => Arc::new(state),
        Err(e) => {
            eprintln!("Failed to initialize app state: {}", e);
            eprintln!("Make sure the embedding model file exists and Milvus is running.");
            std::process::exit(1);
        }
    };
    
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let app = create_router(app_state);
    
    tracing::info!("Vec service listening on http://{}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
