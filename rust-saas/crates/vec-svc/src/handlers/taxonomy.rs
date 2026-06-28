
use axum::{extract::{State, Path, Query}, http::StatusCode, response::IntoResponse, Json};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::model::{DocumentCategory, NewDocumentCategory, DocumentLevel, NewDocumentLevel, DocumentCategoryMapping, NewDocumentCategoryMapping, DocumentLevelMapping, NewDocumentLevelMapping};
use shared::errors::{ServiceResult};

// ============ 请求/响应结构 ============

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub project_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_type: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

impl From<CreateCategoryRequest> for NewDocumentCategory {
    fn from(req: CreateCategoryRequest) -> Self {
        Self {
            project_id: req.project_id,
            category_name: req.category_name,
            category_type: req.category_type,
            parent_id: req.parent_id,
            level: req.level,
            description: req.description,
            icon: req.icon,
            color: req.color,
            sort_order: req.sort_order,
            is_active: req.is_active,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCategoryRequest {
    pub category_name: Option<String>,
    pub category_type: Option<String>,
    pub parent_id: Option<i64>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignCategoriesRequest {
    pub category_ids: Vec<i64>,
    pub primary_category_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLevelRequest {
    pub project_id: Option<i64>,
    pub level_name: Option<String>,
    pub level_type: Option<String>,
    pub level_value: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

impl From<CreateLevelRequest> for NewDocumentLevel {
    fn from(req: CreateLevelRequest) -> Self {
        Self {
            project_id: req.project_id,
            level_name: req.level_name,
            level_type: req.level_type,
            level_value: req.level_value,
            description: req.description,
            icon: req.icon,
            color: req.color,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLevelRequest {
    pub level_name: Option<String>,
    pub level_type: Option<String>,
    pub level_value: Option<i32>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignLevelsRequest {
    pub level_ids: Vec<i64>,
    pub primary_level_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryQuery {
    pub project_id: Option<i64>,
}

// ============ 分类 API ============

/// 
/// POST /api/categories
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCategoryRequest>,
) -> ServiceResult<Json<DocumentCategory>> {
    let category = state.create_category(req.into()).await?;
    Ok(Json(category))
}

/// 
/// GET /api/categories/{id}
pub async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<Json<DocumentCategory>> {
    let category = state.get_category(id).await?;
    Ok(Json(category))
}

/// /// GET /api/categories
pub async fn list_root_categories(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CategoryQuery>,
) -> ServiceResult<Json<Vec<DocumentCategory>>> {
    let project_id = query.project_id.unwrap_or(0);
    let categories = state.list_root_categories(project_id).await?;
    Ok(Json(categories))
}

/// /// GET /api/categories/{parent_id}/children
pub async fn list_child_categories(
    State(state): State<Arc<AppState>>,
    Path(parent_id): Path<i64>,
) -> ServiceResult<Json<Vec<DocumentCategory>>> {
    let categories = state.list_child_categories(parent_id).await?;
    Ok(Json(categories))
}

/// 
/// PUT /api/categories/{id}
pub async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCategoryRequest>,
) -> ServiceResult<Json<DocumentCategory>> {
    let existing = state.get_category(id).await?;
    
    let new_category = NewDocumentCategory {
        project_id: existing.project_id,
        category_name: req.category_name.or(existing.category_name),
        category_type: req.category_type.or(existing.category_type),
        parent_id: req.parent_id.or(existing.parent_id),
        level: req.level.unwrap_or(existing.level),
        description: req.description.or(existing.description),
        icon: req.icon.or(existing.icon),
        color: req.color.or(existing.color),
        sort_order: req.sort_order.unwrap_or(existing.sort_order),
        is_active: req.is_active.unwrap_or(existing.is_active),
    };
    
    let category = state.update_category(id, new_category).await?;
    Ok(Json(category))
}

/// /// DELETE /api/categories/{id}
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<impl IntoResponse> {
    state.delete_category(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 
/// POST /api/documents/{document_id}/categories
pub async fn assign_document_categories(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Json(req): Json<AssignCategoriesRequest>,
) -> ServiceResult<Json<Vec<DocumentCategoryMapping>>> {
    let mappings: Vec<NewDocumentCategoryMapping> = req.category_ids
        .into_iter()
        .map(|cat_id| NewDocumentCategoryMapping {
            document_id,
            category_id: cat_id,
            confidence: None,
            is_primary: req.primary_category_id == Some(cat_id),
        })
        .collect();
    
    let results = state.assign_document_categories(document_id, mappings).await?;
    Ok(Json(results))
}

/// /// GET /api/documents/{document_id}/categories
pub async fn get_document_categories(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
) -> ServiceResult<Json<Vec<DocumentCategory>>> {
    let categories = state.get_document_categories(document_id).await?;
    Ok(Json(categories))
}

// ============ 分级 API ============

/// 
/// POST /api/levels
pub async fn create_level(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLevelRequest>,
) -> ServiceResult<Json<DocumentLevel>> {
    let level = state.create_level(req.into()).await?;
    Ok(Json(level))
}

/// 
/// GET /api/levels/{id}
pub async fn get_level(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<Json<DocumentLevel>> {
    let level = state.get_level(id).await?;
    Ok(Json(level))
}

/// /// GET /api/levels
pub async fn list_levels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CategoryQuery>,
) -> ServiceResult<Json<Vec<DocumentLevel>>> {
    let project_id = query.project_id.unwrap_or(0);
    let levels = state.list_levels(project_id).await?;
    Ok(Json(levels))
}

/// 
/// PUT /api/levels/{id}
pub async fn update_level(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLevelRequest>,
) -> ServiceResult<Json<DocumentLevel>> {
    let existing = state.get_level(id).await?;
    
    let new_level = NewDocumentLevel {
        project_id: existing.project_id,
        level_name: req.level_name.or(existing.level_name),
        level_type: req.level_type.or(existing.level_type),
        level_value: req.level_value.unwrap_or(existing.level_value),
        description: req.description.or(existing.description),
        icon: req.icon.or(existing.icon),
        color: req.color.or(existing.color),
    };
    
    let level = state.update_level(id, new_level).await?;
    Ok(Json(level))
}

/// 
/// DELETE /api/levels/{id}
pub async fn delete_level(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ServiceResult<impl IntoResponse> {
    state.delete_level(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 
/// POST /api/documents/{document_id}/levels
pub async fn assign_document_levels(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
    Json(req): Json<AssignLevelsRequest>,
) -> ServiceResult<Json<Vec<DocumentLevelMapping>>> {
    let mappings: Vec<NewDocumentLevelMapping> = req.level_ids
        .into_iter()
        .map(|level_id| NewDocumentLevelMapping {
            document_id,
            level_id,
            confidence: None,
            is_primary: req.primary_level_id == Some(level_id),
        })
        .collect();
    
    let results = state.assign_document_levels(document_id, mappings).await?;
    Ok(Json(results))
}

/// /// GET /api/documents/{document_id}/levels
pub async fn get_document_levels(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<i64>,
) -> ServiceResult<Json<Vec<DocumentLevel>>> {
    let levels = state.get_document_levels(document_id).await?;
    Ok(Json(levels))
}