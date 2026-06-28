
use axum::{
    extract::{State, Path, Query, Multipart},
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::model::NewDocument;
use crate::task_queue::{NewTask, TaskType};
use shared::errors::{ServiceError, ServiceResult};

/// 
#[derive(Debug, Deserialize)]
pub struct TextDocumentRequest {
    /// 
    pub content: String,
    /// /标题
    pub topic: Option<String>,
    ///  ID
    pub project_id: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    /// 
    pub enable_distillation: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: i64,
    pub topic: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub project_id: Option<i64>,
    pub source_type: Option<String>,
    pub file_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub chunk_count: i32,
    pub word_count: Option<i32>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
    pub indexed_at: Option<String>,
}

impl From<crate::model::Document> for DocumentResponse {
    fn from(doc: crate::model::Document) -> Self {
        Self {
            id: doc.id,
            topic: doc.topic.clone(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            project_id: doc.project_id,
            source_type: doc.source_type.clone(),
            file_type: doc.file_type.clone(),
            status: doc.status.clone(),
            visibility: doc.visibility.clone(),
            chunk_count: doc.chunk_count,
            word_count: doc.word_count,
            version: doc.version,
            created_at: doc.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            updated_at: doc.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            indexed_at: doc.indexed_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
        }
    }
}

/// 
/// POST /api/documents/text
pub async fn add_text_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TextDocumentRequest>,
) -> ServiceResult<Json<DocumentResponse>> {
    tracing::info!("Adding text document: topic='{:?}', content_len={}",
        req.topic, req.content.len());

    let mut conn = state.db_pool.get().map_err(|_| ServiceError::InternalError)?;

    let new_doc = NewDocument {
        project_id: req.project_id,
        title: req.topic.clone(),
        topic: req.topic.clone(),
        content: Some(req.content.clone()),
        content_hash: None,
        source_type: Some("text".to_string()),
        source_url: None,
        file_path: None,
        file_type: None,
        status: Some("processing".to_string()),
        visibility: Some("public".to_string()),
        boundary_level: None,
        token_count: Some(req.content.len() as i32),
        version: 1,
        word_count: Some(req.content.len() as i32),
        chunk_count: 0,
        metadata: req.metadata.clone(),
    };

    let doc: crate::model::Document = diesel::insert_into(crate::schema::documents::table)
        .values(&new_doc)
        .get_result(&mut conn)
        .map_err(|_| ServiceError::InternalError)?;

    let task = NewTask {
        task_type: TaskType::DocumentProcess,
        payload: serde_json::json!({
            "document_id": doc.id,
            "topic": req.topic,
            "content": req.content,
            "project_id": req.project_id,
            "source_type": "text",
        }),
    };
    state.task_queue.enqueue(task).await.map_err(|_| ServiceError::InternalError)?;

    Ok(Json(DocumentResponse::from(doc)))
}

/// 
/// POST /api/documents/file
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ServiceResult<Json<DocumentResponse>> {
    let mut topic: Option<String> = None;
    let mut project_id: Option<i64> = None;
    let mut content: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut file_ext: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| ServiceError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "topic" => {
                topic = field.text().await.ok();
            }
            "project_id" => {
                if let Ok(text) = field.text().await.map_err(|e| ServiceError::BadRequest(e.to_string())) {
                    if let Ok(pid) = text.parse() {
                        project_id = Some(pid);
                    }
                }
            }
            "file" => {
                let fname = field.file_name().unwrap_or("unknown").to_string();
                filename = Some(fname.clone());
                file_ext = fname.rsplit('.').next().map(|s| s.to_lowercase());
                let bytes = field.bytes().await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;

                content = Some(extract_text_from_file(&fname, &bytes).await?);
            }
            _ => {}
        }
    }

    let content = content.ok_or_else(|| ServiceError::BadRequest("Missing file content".to_string()))?;

    tracing::info!("Uploading file: topic='{:?}', filename='{:?}', content_len={}",
        topic, filename, content.len());

    let mut conn = state.db_pool.get().map_err(|_| ServiceError::InternalError)?;

    let new_doc = NewDocument {
        project_id,
        title: topic.clone().or_else(|| filename.clone()),
        topic: topic.clone().or_else(|| filename.clone()),
        content: Some(content.clone()),
        content_hash: None,
        source_type: Some("file".to_string()),
        source_url: None,
        file_path: filename.clone(),
        file_type: file_ext.clone(),
        status: Some("processing".to_string()),
        visibility: Some("public".to_string()),
        boundary_level: None,
        token_count: Some(content.len() as i32),
        version: 1,
        word_count: Some(content.len() as i32),
        chunk_count: 0,
        metadata: None,
    };

    let doc: crate::model::Document = diesel::insert_into(crate::schema::documents::table)
        .values(&new_doc)
        .get_result(&mut conn)
        .map_err(|_| ServiceError::InternalError)?;

    let task = NewTask {
        task_type: TaskType::DocumentProcess,
        payload: serde_json::json!({
            "document_id": doc.id,
            "topic": topic.or_else(|| filename.clone()),
            "content": content,
            "project_id": project_id,
            "source_type": "file",
            "file_path": filename,
            "file_type": file_ext,
        }),
    };
    state.task_queue.enqueue(task).await.map_err(|_| ServiceError::InternalError)?;

    Ok(Json(DocumentResponse::from(doc)))
}

/// 
async fn extract_text_from_file(filename: &str, bytes: &[u8]) -> Result<String, ServiceError> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "json" | "xml" | "csv" | "log" => {
            String::from_utf8(bytes.to_vec())
                .map_err(|e| ServiceError::BadRequest(format!("Invalid UTF-8: {}", e)))
        }
        "pdf" => {
            Err(ServiceError::BadRequest("PDF parsing not implemented".to_string()))
        }
        "docx" => {
            Err(ServiceError::BadRequest("DOCX parsing not implemented".to_string()))
        }
        _ => {
            Err(ServiceError::BadRequest(format!("Unsupported file type: {}", ext)))
        }
    }
}

/// 
/// GET /api/documents/{id}
pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<Json<DocumentResponse>> {
    let doc = state.get_document(id).await?;
    Ok(Json(DocumentResponse::from(doc)))
}

/// 
/// DELETE /api/documents/{id}
pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<Json<serde_json::Value>> {
    tracing::info!("Deleting document: id={}", id);

    state.delete_document(id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "id": id,
    })))
}

/// 
/// POST /api/documents/{id}/reindex
pub async fn reindex_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<Json<serde_json::Value>> {
    tracing::info!("Reindexing document: id={}", id);

    let task = NewTask {
        task_type: TaskType::DocumentReindex,
        payload: serde_json::json!({ "document_id": id }),
    };
    state.task_queue.enqueue(task).await.map_err(|_| ServiceError::InternalError)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "id": id,
        "message": "Reindex task created",
    })))
}

/// 
/// GET /api/documents?project_id=xxx&page=1&page_size=20
#[derive(Debug, Deserialize)]
pub struct ListDocumentsRequest {
    pub project_id: Option<i64>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDocumentsRequest>,
) -> ServiceResult<Json<ListDocumentsResponse>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let docs = state.list_documents(params.project_id, page, page_size).await?;
    let total = state.count_documents(params.project_id).await?;

    let documents: Vec<_> = docs.into_iter().map(DocumentResponse::from).collect();

    Ok(Json(ListDocumentsResponse {
        documents,
        total,
        page,
        page_size,
    }))
}
