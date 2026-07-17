use axum::{
    Router,
    routing::{get, post, delete, put},
    middleware,
    response::Response,
    http::StatusCode,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use futures::future;

use crate::app_state::AppState;
use crate::handlers::{search, document, embedding, distillation, taxonomy, boundary, storage, task, knowledge_graph, verification, analytics, version, import_export};
use shared::middleware::auth_middleware;

/// 添加 UTF-8 charset 到 JSON 响应
async fn utf8_charset_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let mut response = next.run(req).await;
    if let Some(content_type) = response.headers().get("content-type") {
        let ct = content_type.to_str().unwrap_or("");
        if ct.contains("application/json") && !ct.contains("charset") {
            response.headers_mut().insert(
                "content-type",
                "application/json; charset=utf-8".parse().unwrap(),
            );
        }
    }
    response
}

/// 
pub fn create_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // 
        .route("/api/health", get(embedding::health_check))
        
        // 
        .route("/api/search", post(search::search))
        .route("/api/search/suggest", get(search::suggest))
        .route("/api/search/autocomplete", get(search::autocomplete))
        .route("/api/projects/:project_id/search", get(search::search_by_project))
        
        // 
        .route("/api/documents/text", post(document::add_text_document))
        .route("/api/documents/file", post(document::upload_file))
        .route("/api/documents", get(document::list_documents))
        .route("/api/documents/:id", get(document::get_document))
        .route("/api/documents/:id", delete(document::delete_document))
        .route("/api/documents/:id/reindex", post(document::reindex_document))
        
        // 
        .route("/api/embed", post(embedding::embed))
        .route("/api/embed/batch", post(embedding::embed_batch))
        
        // 
        .route("/api/documents/:id/distill", post(distillation::distill_document))
        .route("/api/documents/:id/knowledge-points", get(distillation::list_knowledge_points))
        .route("/api/distill/preview", post(distillation::preview_distillation))
        .route("/api/knowledge-points/:id", delete(distillation::delete_knowledge_point))
        
        // 
        .route("/api/categories", post(taxonomy::create_category))
        .route("/api/categories", get(taxonomy::list_root_categories))
        .route("/api/categories/:id", get(taxonomy::get_category))
        .route("/api/categories/:id", put(taxonomy::update_category))
        .route("/api/categories/:id", delete(taxonomy::delete_category))
        .route("/api/categories/:parent_id/children", get(taxonomy::list_child_categories))
        
        // 
        .route("/api/levels", post(taxonomy::create_level))
        .route("/api/levels", get(taxonomy::list_levels))
        .route("/api/levels/:id", get(taxonomy::get_level))
        .route("/api/levels/:id", put(taxonomy::update_level))
        .route("/api/levels/:id", delete(taxonomy::delete_level))
        
        // 
        .route("/api/documents/:document_id/categories", post(taxonomy::assign_document_categories))
        .route("/api/documents/:document_id/categories", get(taxonomy::get_document_categories))
        
        // 
        .route("/api/documents/:document_id/levels", post(taxonomy::assign_document_levels))
        .route("/api/documents/:document_id/levels", get(taxonomy::get_document_levels))
        
        // 
        .route("/api/documents/:document_id/visibility", post(boundary::set_document_visibility))
        .route("/api/documents/:document_id/boundary", get(boundary::get_document_boundary))
        .route("/api/documents/:document_id/check-access", post(boundary::check_document_access))
        .route("/api/documents/accessible", post(boundary::get_accessible_document_ids))
        
        // 
        .route("/api/shares", post(boundary::create_document_share))
        .route("/api/shares/batch", post(boundary::batch_create_shares))
        .route("/api/shares/:id", delete(boundary::delete_document_share))
        .route("/api/documents/:document_id/shares", get(boundary::get_document_shares))
        .route("/api/documents/:document_id/check-share", post(boundary::check_share_access))
        
        // 
        .route("/api/storage/upload", post(storage::upload_file))
        .route("/api/storage/download/:object_name", get(storage::download_file))
        .route("/api/storage/delete/:object_name", delete(storage::delete_file))
        .route("/api/storage/presigned/:object_name", get(storage::get_presigned_url))
        .route("/api/storage/projects/:project_id/files", get(storage::list_project_files))
        .route("/api/storage/health", get(storage::check_minio_connection))
        
        // 
        .route("/api/tasks", post(task::create_task))
        .route("/api/tasks", get(task::list_tasks))
        .route("/api/tasks/:task_id", get(task::get_task))
        .route("/api/tasks/:task_id", delete(task::cancel_task))
        .route("/api/tasks/:task_id/progress", get(task::get_task_progress))
        
        // 
        .route("/api/graph/extract", post(knowledge_graph::extract_entities))
        .route("/api/graph/entities", get(knowledge_graph::search_entities))
        .route("/api/graph/entities/:entity_id", get(knowledge_graph::get_entity))
        .route("/api/graph/entities/:entity_id", delete(knowledge_graph::delete_entity))
        .route("/api/graph/entities/:entity_id/relations", get(knowledge_graph::get_entity_relations))
        .route("/api/graph/projects/:project_id/entities", get(knowledge_graph::list_entities))
        
        // 
        .route("/api/verification/facts", post(verification::verify_facts))
        .route("/api/verification/graph", post(verification::verify_graph_consistency))
        .route("/api/verification/conflicts", get(verification::list_conflicts))
        .route("/api/verification/conflicts/:conflict_id", put(verification::resolve_conflict))
        
        // 
        .route("/api/documents/:document_id/versions", get(version::list_versions))
        .route("/api/documents/:document_id/versions", post(version::create_version))
        .route("/api/versions/:version_id", get(version::get_version))
        .route("/api/versions/compare", get(version::compare_versions))
        .route("/api/documents/:document_id/rollback", post(version::rollback_version))
        
        // 
        .route("/api/analytics/summary", get(analytics::get_analytics_summary))
        .route("/api/analytics/document", get(analytics::get_document_stats))
        
        // 
        .route("/api/import/documents", post(import_export::import_documents))
        .route("/api/export/documents", get(import_export::export_documents))
        .route("/api/export/knowledge-graph", get(import_export::export_knowledge_graph))

        .layer(middleware::from_fn(utf8_charset_middleware))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(cors)
        .with_state(app_state)
}

