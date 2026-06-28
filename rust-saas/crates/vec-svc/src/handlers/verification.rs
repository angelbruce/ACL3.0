
use axum::{extract::{State, Path, Query}, Json};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::verification::{FactVerificationResult, GraphConsistencyResult};
use crate::model::{VerificationConflict};
use shared::errors::{ServiceResult, ServiceError};

#[derive(Debug, serde::Deserialize)]
pub struct VerifyFactRequest {
    pub query_text: String,
    pub llm_summary: String,
    pub project_id: Option<i64>,
}

pub async fn verify_facts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyFactRequest>,
) -> ServiceResult<Json<FactVerificationResult>> {
    let result = state.verification_service
        .verify_facts(&req.query_text, &req.llm_summary, req.project_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

pub async fn verify_graph_consistency(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyFactRequest>,
) -> ServiceResult<Json<GraphConsistencyResult>> {
    let result = state.verification_service
        .verify_graph_consistency(&req.query_text, &req.llm_summary, req.project_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListConflictsRequest {
    pub project_id: Option<i64>,
    pub resolved: Option<bool>,
    pub limit: Option<usize>,
}

pub async fn list_conflicts(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ListConflictsRequest>,
) -> ServiceResult<Json<Vec<VerificationConflict>>> {
    let limit = req.limit.unwrap_or(20);
    let conflicts = state.verification_service
        .list_conflicts(req.project_id, req.resolved, limit)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(conflicts))
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: String,
}

pub async fn resolve_conflict(
    State(state): State<Arc<AppState>>,
    Path(conflict_id): Path<i64>,
    Json(req): Json<ResolveConflictRequest>,
) -> ServiceResult<Json<()>> {
    state.verification_service
        .resolve_conflict(conflict_id, &req.resolution)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(()))
}
