
use axum::{extract::{State, Path, Query}, Json};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::knowledge_graph::{EntityExtractionResult};
use crate::model::{KnowledgeEntity, KnowledgeRelation};
use shared::errors::{ServiceResult, ServiceError};

#[derive(Debug, serde::Deserialize)]
pub struct ExtractEntitiesRequest {
    pub document_id: i64,
    pub project_id: Option<i64>,
}

pub async fn extract_entities(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExtractEntitiesRequest>,
) -> ServiceResult<Json<EntityExtractionResult>> {
    let doc = state.get_document(req.document_id).await
        .map_err(|e| ServiceError::InternalError)?;
    
    let content = doc.content.clone().unwrap_or_default();
    
    let result = state.knowledge_graph_service
        .extract_from_document(req.document_id, &content, req.project_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

pub async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<i64>,
) -> ServiceResult<Json<KnowledgeEntity>> {
    let entity = state.knowledge_graph_service
        .get_entity_by_id(entity_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(entity))
}

pub async fn list_entities(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<KnowledgeEntity>>> {
    let entities = state.knowledge_graph_service
        .get_entities_by_project(project_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(entities))
}

pub async fn get_entity_relations(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<i64>,
) -> ServiceResult<Json<Vec<KnowledgeRelation>>> {
    let relations = state.knowledge_graph_service
        .get_relations_by_entity(entity_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(relations))
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchEntitiesRequest {
    pub query: Option<String>,
    pub project_id: Option<i64>,
    pub limit: Option<usize>,
}

pub async fn search_entities(
    State(state): State<Arc<AppState>>,
    Query(req): Query<SearchEntitiesRequest>,
) -> ServiceResult<Json<Vec<KnowledgeEntity>>> {
    let entities = if let Some(ref q) = req.query {
        state.knowledge_graph_service
            .search_entities(q, req.project_id)
            .await
            .map_err(|e| ServiceError::InternalError)?
    } else {
        // 无查询条件时返回所有实体
        if let Some(pid) = req.project_id {
            state.knowledge_graph_service
                .get_entities_by_project(pid)
                .await
                .map_err(|e| ServiceError::InternalError)?
        } else {
            state.knowledge_graph_service
                .search_entities("", None)
                .await
                .map_err(|e| ServiceError::InternalError)?
        }
    };

    // 限制返回数量
    let limit = req.limit.unwrap_or(100);
    let mut result = entities;
    if result.len() > limit {
        result.truncate(limit);
    }

    Ok(Json(result))
}

pub async fn delete_entity(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<i64>,
) -> ServiceResult<Json<()>> {
    state.knowledge_graph_service
        .delete_entity(entity_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(()))
}
