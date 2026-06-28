
use axum::{
    extract::{Multipart, Query, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::import_export::{ImportDocument, ImportResult, ExportFormat, KnowledgeGraphExport};
use crate::task_queue::{NewTask, TaskType};
use shared::errors::{ServiceError, ServiceResult};

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub project_id: Option<i64>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<ImportResult>,
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub code: i32,
    pub message: String,
    pub content: Option<String>,
    pub format: Option<String>,
    pub document_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct GraphExportResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<KnowledgeGraphExport>,
}

pub async fn import_documents(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ServiceResult<Json<ImportResponse>> {
    let mut project_id: Option<i64> = None;
    let mut documents = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| ServiceError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "project_id" => {
                if let Ok(text) = field.text().await {
                    if let Ok(pid) = text.parse() {
                        project_id = Some(pid);
                    }
                }
            }
            "file" | "files" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let bytes = field.bytes().await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
                let content = String::from_utf8(bytes.to_vec())
                    .map_err(|e| ServiceError::BadRequest(format!("Invalid UTF-8: {}", e)))?;

                documents.push(ImportDocument {
                    title: filename.clone(),
                    content,
                    source: Some(filename),
                    metadata: std::collections::HashMap::new(),
                });
            }
            _ => {}
        }
    }

    if documents.is_empty() {
        return Err(ServiceError::BadRequest("No valid files uploaded".to_string()));
    }

    match state
        .import_export_service
        .import_documents(project_id, documents)
        .await
    {
        Ok(result) => {
            for document_id in &result.document_ids {
                let task = NewTask {
                    task_type: TaskType::DocumentProcess,
                    payload: serde_json::json!({ "document_id": document_id }),
                };
                let _ = state.task_queue.enqueue(task).await;
            }
            Ok(Json(ImportResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(result),
            }))
        }
        Err(e) => Ok(Json(ImportResponse {
            code: 500,
            message: format!("Import failed: {}", e),
            data: None,
        })),
    }
}

pub async fn export_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
) -> impl IntoResponse {
    let format = match query.format.as_deref() {
        Some("markdown") | Some("md") => ExportFormat::Markdown,
        Some("csv") => ExportFormat::Csv,
        _ => ExportFormat::Json,
    };

    match state
        .import_export_service
        .export_documents(query.project_id, None, format)
        .await
    {
        Ok(result) => {
            let content_type = match result.format {
                ExportFormat::Json => "application/json",
                ExportFormat::Markdown => "text/markdown",
                ExportFormat::Csv => "text/csv",
            };

            (
                StatusCode::OK,
                [
                    ("Content-Type", content_type),
                    ("Content-Disposition", "attachment; filename=\"export\""),
                ],
                result.content,
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [
                ("Content-Type", "text/plain"),
                ("Content-Disposition", "attachment; filename=\"internal_error\""),
            ],
            format!("Export failed: {}", e),
        ),
    }
}

pub async fn export_knowledge_graph(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
) -> Json<GraphExportResponse> {
    match state
        .import_export_service
        .export_knowledge_graph(query.project_id)
        .await
    {
        Ok(data) => Json(GraphExportResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }),
        Err(e) => Json(GraphExportResponse {
            code: 500,
            message: format!("Export failed: {}", e),
            data: None,
        }),
    }
}
