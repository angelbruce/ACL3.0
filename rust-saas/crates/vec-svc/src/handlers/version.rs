
use axum::{
    extract::{Path, Query, State, Json as AxumJson},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::version_control::{DocumentVersionSummary, DiffResult, DocumentVersion};

#[derive(Debug, Deserialize)]
pub struct CreateVersionRequest {
    pub content: String,
    pub change_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    pub version_a: i64,
    pub version_b: i64,
}

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub version_id: i64,
}

#[derive(Debug, Serialize)]
pub struct VersionListResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<Vec<DocumentVersionSummary>>,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<DocumentVersion>,
}

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<DiffResult>,
}

#[derive(Debug, Serialize)]
pub struct CreateVersionResponse {
    pub code: i32,
    pub message: String,
    pub version_id: Option<i64>,
}

pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Query(query): Query<VersionListQuery>,
) -> Json<VersionListResponse> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    match state.version_service.list_versions(document_id, limit, offset).await {
        Ok(versions) => Json(VersionListResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(versions),
        }),
        Err(e) => Json(VersionListResponse {
            code: 500,
            message: format!("Failed to list versions: {}", e),
            data: None,
        }),
    }
}

pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Path(version_id): Path<i64>,
) -> Json<VersionResponse> {
    match state.version_service.get_version(version_id).await {
        Ok(version) => Json(VersionResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(version),
        }),
        Err(e) => Json(VersionResponse {
            code: 500,
            message: format!("Failed to get version: {}", e),
            data: None,
        }),
    }
}

pub async fn create_version(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    AxumJson(req): AxumJson<CreateVersionRequest>,
) -> Json<CreateVersionResponse> {
    match state
        .version_service
        .create_version(document_id, &req.content, req.change_note.as_deref(), None)
        .await
    {
        Ok(version_id) => Json(CreateVersionResponse {
            code: 0,
            message: "success".to_string(),
            version_id: Some(version_id),
        }),
        Err(e) => Json(CreateVersionResponse {
            code: 500,
            message: format!("Failed to create version: {}", e),
            version_id: None,
        }),
    }
}

pub async fn compare_versions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CompareQuery>,
) -> Json<DiffResponse> {
    match state
        .version_service
        .compare_versions(query.version_a, query.version_b)
        .await
    {
        Ok(diff) => Json(DiffResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(diff),
        }),
        Err(e) => Json(DiffResponse {
            code: 500,
            message: format!("Failed to compare versions: {}", e),
            data: None,
        }),
    }
}

pub async fn rollback_version(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    AxumJson(req): AxumJson<RollbackRequest>,
) -> Json<CreateVersionResponse> {
    match state
        .version_service
        .rollback_to_version(document_id, req.version_id, None)
        .await
    {
        Ok(version_id) => Json(CreateVersionResponse {
            code: 0,
            message: "success".to_string(),
            version_id: Some(version_id),
        }),
        Err(e) => Json(CreateVersionResponse {
            code: 500,
            message: format!("Failed to rollback: {}", e),
            version_id: None,
        }),
    }
}
