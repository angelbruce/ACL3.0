use axum::{
    extract::{State, Path, Query, Json},
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use shared::errors::{ServiceResult};

/// 
#[derive(Debug, Serialize, Deserialize)]
pub struct DistillRequest {
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistillPreviewRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgePointResponse {
    pub id: i64,
    pub document_id: i64,
    pub point_type: Option<String>,
    pub point_content: Option<String>,
    pub confidence: Option<f64>,
    pub created_at: String,
}

impl From<crate::model::KnowledgePoint> for KnowledgePointResponse {
    fn from(p: crate::model::KnowledgePoint) -> Self {
        Self {
            id: p.id,
            document_id: p.document_id,
            point_type: p.point_type,
            point_content: p.point_content,
            confidence: p.confidence,
            created_at: p.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DistillPreviewResponse {
    pub summary: Option<String>,
    pub key_phrases: Vec<String>,
    pub qna_pairs: Vec<QnAPairResponse>,
    pub facts: Vec<String>,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct QnAPairResponse {
    pub question: String,
    pub answer: String,
}

/// 
/// POST /api/documents/{id}/distill
pub async fn distill_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    AxumJson(_req): AxumJson<DistillRequest>,
) -> ServiceResult<AxumJson<Vec<KnowledgePointResponse>>> {
    tracing::info!("Distilling document: id={}", id);

    let points = state.distill_document(id).await?;

    let response: Vec<_> = points.into_iter()
        .map(KnowledgePointResponse::from)
        .collect();

    Ok(AxumJson(response))
}

/// 
/// GET /api/documents/{id}/knowledge-points?point_type=xxx
#[derive(Debug, Deserialize)]
pub struct ListKnowledgePointsRequest {
    pub point_type: Option<String>,
}

pub async fn list_knowledge_points(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Query(params): Query<ListKnowledgePointsRequest>,
) -> ServiceResult<AxumJson<Vec<KnowledgePointResponse>>> {
    let points = state.get_knowledge_points(id, params.point_type.as_deref()).await?;

    let response: Vec<_> = points.into_iter()
        .map(KnowledgePointResponse::from)
        .collect();

    Ok(AxumJson(response))
}

/// 
/// POST /api/distill/preview
pub async fn preview_distillation(
    State(state): State<Arc<AppState>>,
    AxumJson(req): AxumJson<DistillPreviewRequest>,
) -> ServiceResult<AxumJson<DistillPreviewResponse>> {
    let result = state.preview_distillation(&req.content).await?;

    let qna_pairs = result.qna_pairs.into_iter()
        .map(|q| QnAPairResponse {
            question: q.question,
            answer: q.answer,
        })
        .collect();

    Ok(AxumJson(DistillPreviewResponse {
        summary: result.summary,
        key_phrases: result.key_phrases,
        qna_pairs,
        facts: result.facts,
        best_practices: result.best_practices,
    }))
}

/// /// DELETE /api/knowledge-points/{id}
pub async fn delete_knowledge_point(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<AxumJson<serde_json::Value>> {
    tracing::info!("Deleting knowledge point: id={}", id);

    state.delete_knowledge_point(id).await?;

    Ok(AxumJson(serde_json::json!({
        "success": true,
        "id": id,
    })))
}
