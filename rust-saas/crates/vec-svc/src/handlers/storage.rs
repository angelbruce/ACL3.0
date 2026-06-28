use axum::{extract::{State, Path, Query}, Json};
use std::sync::Arc;
use std::time::Duration;

use crate::app_state::AppState;
use shared::errors::{ServiceResult, ServiceError};

#[derive(Debug, serde::Deserialize)]
pub struct UploadFileRequest {
    pub project_id: Option<i64>,
    pub file_name: String,
    pub content_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UploadFileResponse {
    pub object_name: String,
    pub presigned_url: String,
}

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UploadFileRequest>,
) -> ServiceResult<Json<UploadFileResponse>> {
    let content = vec![0u8; 0];
    
    let object_name = state.minio_service
        .upload_file(req.project_id, &req.file_name, &content, &req.content_type)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    let presigned_url = state.minio_service
        .get_presigned_url(&object_name, Duration::from_secs(24 * 3600).as_secs() as u32)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(UploadFileResponse {
        object_name,
        presigned_url,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct DownloadFileResponse {
    pub content: Vec<u8>,
    pub content_type: String,
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(object_name): Path<String>,
) -> ServiceResult<Json<DownloadFileResponse>> {
    let content = state.minio_service
        .download_file(&object_name)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(DownloadFileResponse {
        content,
        content_type: "application/octet-stream".to_string(),
    }))
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(object_name): Path<String>,
) -> ServiceResult<Json<()>> {
    state.minio_service
        .delete_file(&object_name)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(()))
}

#[derive(Debug, serde::Deserialize)]
pub struct GetPresignedUrlRequest {
    pub expires_hours: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub struct GetPresignedUrlResponse {
    pub url: String,
}

pub async fn get_presigned_url(
    State(state): State<Arc<AppState>>,
    Path(object_name): Path<String>,
    Query(req): Query<GetPresignedUrlRequest>,
) -> ServiceResult<Json<GetPresignedUrlResponse>> {
    let expires_hours = req.expires_hours.unwrap_or(24);
    let duration = Duration::from_secs(expires_hours * 3600);

    let url = state.minio_service
        .get_presigned_url(&object_name, duration.as_secs() as u32)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(GetPresignedUrlResponse { url }))
}

#[derive(Debug, serde::Serialize)]
pub struct ListFilesResponse {
    pub files: Vec<String>,
}

pub async fn list_project_files(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<ListFilesResponse>> {
    let files = state.minio_service
        .list_project_files(project_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(ListFilesResponse { files }))
}

pub async fn check_minio_connection(
    State(state): State<Arc<AppState>>,
) -> ServiceResult<Json<bool>> {
    let result = state.minio_service.check_connection().await;
    Ok(Json(result))
}
