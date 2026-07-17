
use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use shared::errors::{ServiceError, ServiceResult};

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
}

/// 
#[derive(Debug, Deserialize)]
pub struct EmbedBatchRequest {
    pub texts: Vec<String>,
}

/// 
#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
    pub dimension: usize,
}

/// 
#[derive(Debug, Serialize)]
pub struct EmbedBatchResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub dimension: usize,
}

/// POST /api/embed
pub async fn embed(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmbedRequest>,
) -> ServiceResult<Json<EmbedResponse>> {
    let embedding = state.embed_text(&req.text).await?;
    let dimension = embedding.len();
    
    Ok(Json(EmbedResponse {
        embedding,
        dimension,
    }))
}

/// 
/// POST /api/embed/batch
pub async fn embed_batch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmbedBatchRequest>,
) -> ServiceResult<Json<EmbedBatchResponse>> {
    let embeddings = state.embed_text_batch(&req.texts).await?;
    let dimension = embeddings.first().map(|e| e.len()).unwrap_or(0);
    
    Ok(Json(EmbedBatchResponse {
        embeddings,
        dimension,
    }))
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub embedding_loaded: bool,
    pub milvus_connected: bool,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> ServiceResult<Json<HealthResponse>> {
    let embedding_loaded = state.is_embedding_loaded();
    let milvus_connected = state.check_milvus_connection().await;
    
    let status = if embedding_loaded && milvus_connected {
        "healthy"
    } else {
        "degraded"
    };
    
    Ok(Json(HealthResponse {
        status: status.to_string(),
        embedding_loaded,
        milvus_connected,
    }))
}
