use axum::{extract::{State, Path, Json}, http::StatusCode, response::IntoResponse};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::model::{DocumentBoundary, DocumentShare, NewDocumentShare};
use shared::errors::{ServiceResult};

// ============ 请求/响应结构 ============

#[derive(Debug, Serialize, Deserialize)]
pub struct SetVisibilityRequest {
    pub visibility: String,
    pub owner_id: Option<i64>,
    pub project_id: Option<i64>,
    pub team_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckAccessRequest {
    pub user_id: i64,
    pub user_projects: Vec<i64>,
    pub user_teams: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessResult {
    pub document_id: i64,
    pub has_access: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShareRequest {
    pub document_id: i64,
    pub share_type: Option<String>,
    pub target_type: String,
    pub target_id: i64,
    pub granted_by: Option<i64>,
    pub expire_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCreateSharesRequest {
    pub document_id: i64,
    pub share_type: Option<String>,
    pub target_type: String,
    pub target_ids: Vec<i64>,
    pub granted_by: Option<i64>,
    pub expire_at: Option<String>,
}

// ============ 边界 API ============

/// /// POST /api/documents/{document_id}/visibility
pub async fn set_document_visibility(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Json(req): Json<SetVisibilityRequest>,
) -> ServiceResult<Json<DocumentBoundary>> {
    let boundary = state.set_document_visibility(document_id, &req.visibility, req.owner_id, req.project_id, req.team_id).await?;
    Ok(Json(boundary))
}

/// 
/// GET /api/documents/{document_id}/boundary
pub async fn get_document_boundary(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
) -> ServiceResult<Json<Option<DocumentBoundary>>> {
    let boundary = state.get_document_boundary(document_id).await?;
    Ok(Json(boundary))
}

/// /// POST /api/documents/{document_id}/check-access
pub async fn check_document_access(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Json(req): Json<CheckAccessRequest>,
) -> ServiceResult<Json<AccessResult>> {
    let has_access = state.check_document_access(document_id, req.user_id, req.user_projects, req.user_teams).await?;
    Ok(Json(AccessResult { document_id, has_access }))
}

///  ID 列表
/// POST /api/documents/accessible
pub async fn get_accessible_document_ids(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckAccessRequest>,
) -> ServiceResult<Json<Vec<i64>>> {
    let doc_ids = state.get_accessible_document_ids(req.user_id, req.user_projects, req.user_teams).await?;
    Ok(Json(doc_ids))
}

// ============ 共享 API ============

/// 
/// POST /api/shares
pub async fn create_document_share(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateShareRequest>,
) -> ServiceResult<Json<DocumentShare>> {
    let expire_at = req.expire_at.map(|s| chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.naive_utc()).ok()).flatten();
    
    let share = NewDocumentShare {
        document_id: req.document_id,
        share_type: req.share_type,
        target_type: Some(req.target_type),
        target_id: Some(req.target_id),
        granted_by: req.granted_by,
        expire_at,
    };
    
    let result = state.create_document_share(share).await?;
    Ok(Json(result))
}

/// /// GET /api/documents/{document_id}/shares
pub async fn get_document_shares(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
) -> ServiceResult<Json<Vec<DocumentShare>>> {
    let shares = state.get_document_shares(document_id).await?;
    Ok(Json(shares))
}

/// 
/// DELETE /api/shares/{id}
pub async fn delete_document_share(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<impl IntoResponse> {
    state.delete_document_share(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 
/// POST /api/shares/batch
pub async fn batch_create_shares(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchCreateSharesRequest>,
) -> ServiceResult<Json<Vec<DocumentShare>>> {
    let expire_at = req.expire_at.map(|s| chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.naive_utc()).ok()).flatten();
    
    let shares: Vec<NewDocumentShare> = req.target_ids
        .into_iter()
        .map(|target_id| NewDocumentShare {
            document_id: req.document_id,
            share_type: req.share_type.clone(),
            target_type: Some(req.target_type.clone()),
            target_id: Some(target_id),
            granted_by: req.granted_by,
            expire_at: expire_at.clone(),
        })
        .collect();
    
    let results = state.batch_create_shares(shares).await?;
    Ok(Json(results))
}

/// 
/// POST /api/documents/{document_id}/check-share
pub async fn check_share_access(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Json(req): Json<CheckAccessRequest>,
) -> ServiceResult<Json<AccessResult>> {
    let has_access = state.check_share_access(document_id, req.user_id).await?;
    Ok(Json(AccessResult { document_id, has_access }))
}