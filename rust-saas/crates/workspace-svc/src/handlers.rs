use axum::{extract::{Path, Extension}, Json, http::{HeaderName, HeaderValue}, response::IntoResponse};
use chrono::{Utc, DateTime};
use mime_guess::from_path;
use serde::{Serialize, Deserialize};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{WorkspaceFile, KanbanBoard, KanbanBoardWithItems, KanbanItem, KanbanSubscription, SubscribedBoard, CreateKanbanBoardRequest, UpdateKanbanBoardRequest, ShareFileRequest};
use shared::utils::Claims;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::repository::WorkspaceRepository;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env::var("WORKSPACE_ROOT").unwrap_or_else(|_| "./workspace_storage".to_string()))
}

fn get_user_workspace_path(user_id: i64) -> PathBuf {
    get_workspace_root().join(user_id.to_string())
}

fn sanitize_path(path: &str) -> ServiceResult<String> {
    let path = path.replace("..", "").replace("//", "/");
    if path.starts_with('/') {
        Ok(path[1..].to_string())
    } else {
        Ok(path)
    }
}

pub async fn list_projects(
    Extension(claims): Extension<Claims>,
) -> ServiceResult<Json<Vec<ProjectInfo>>> {
    let user_id = claims.user_id;
    let workspace_path = get_user_workspace_path(user_id);
    
    if !workspace_path.exists() {
        fs::create_dir_all(&workspace_path).map_err(|_| ServiceError::InternalError)?;
        return Ok(Json(Vec::new()));
    }
    
    let mut projects: Vec<ProjectInfo> = Vec::new();
    
    for entry in fs::read_dir(&workspace_path).map_err(|_| ServiceError::InternalError)? {
        let entry = entry.map_err(|_| ServiceError::InternalError)?;
        let path = entry.path();
        
        if path.is_dir() {
            let project_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            projects.push(ProjectInfo {
                name: project_name.clone(),
                path: project_name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
    }
    
    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    
    Ok(Json(projects))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

pub async fn create_project(
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateProjectRequest>,
) -> ServiceResult<Json<ProjectInfo>> {
    let user_id = claims.user_id;
    let project_name = sanitize_path(&req.name)?;
    
    if project_name.is_empty() {
        return Err(ServiceError::InvalidInput("Project name cannot be empty".to_string()));
    }
    
    let project_path = get_user_workspace_path(user_id).join(&project_name);
    
    if project_path.exists() {
        return Err(ServiceError::Conflict("Project already exists".to_string()));
    }
    
    fs::create_dir_all(&project_path).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
    
    Ok(Json(ProjectInfo {
        name: project_name.clone(),
        path: project_name,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }))
}

pub async fn delete_project(
    Extension(claims): Extension<Claims>,
    Path(project_name): Path<String>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;
    let sanitized_name = sanitize_path(&project_name)?;
    
    let project_path = get_user_workspace_path(user_id).join(&sanitized_name);
    
    if !project_path.exists() {
        return Err(ServiceError::NotFound);
    }
    
    fs::remove_dir_all(&project_path).map_err(|_| ServiceError::InternalError)?;
    
    Ok(Json(HashMap::from([("message".to_string(), "Project deleted successfully".to_string())])))
}

pub async fn list_project_files(
    Extension(claims): Extension<Claims>,
    Path(project_name): Path<String>,
) -> ServiceResult<Json<Vec<WorkspaceFile>>> {
    let user_id = claims.user_id;
    let sanitized_name = sanitize_path(&project_name)?;
    
    let project_path = get_user_workspace_path(user_id).join(&sanitized_name);
    
    if !project_path.exists() {
        return Err(ServiceError::NotFound);
    }
    
    let mut files: Vec<WorkspaceFile> = Vec::new();
    
    for entry in fs::read_dir(&project_path).map_err(|_| ServiceError::InternalError)? {
        let entry = entry.map_err(|_| ServiceError::InternalError)?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|_| ServiceError::InternalError)?;
        
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let relative_path = format!("{}/{}", sanitized_name, file_name);
        
        files.push(WorkspaceFile {
            id: 0,
            user_id,
            file_path: relative_path,
            file_name: file_name.clone(),
            file_size: metadata.len() as i64,
            is_directory: metadata.is_dir(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        });
    }
    
    files.sort_by(|a, b| {
        if a.is_directory != b.is_directory {
            a.is_directory.cmp(&b.is_directory).reverse()
        } else {
            a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
        }
    });
    
    Ok(Json(files))
}

pub async fn download_file(
    Extension(_claims): Extension<Claims>,
    Path(file_path): Path<String>,
) -> ServiceResult<impl IntoResponse> {
    let sanitized_path = sanitize_path(&file_path)?;
    let full_path = get_workspace_root().join(&sanitized_path);
    
    if !full_path.exists() {
        return Err(ServiceError::NotFound);
    }
    
    let content = fs::read(&full_path).map_err(|_| ServiceError::InternalError)?;
    let file_name = full_path.file_name().unwrap().to_string_lossy().to_string();
    let mime_type = from_path(&full_path).first_or_octet_stream().to_string();
    
    let headers = [
        (HeaderName::from_static("content-type"), HeaderValue::from_str(&mime_type).unwrap()),
        (HeaderName::from_static("content-disposition"), HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_name)).unwrap()),
    ];
    
    Ok((headers, content))
}

pub async fn delete_file(
    Extension(claims): Extension<Claims>,
    Path(file_path): Path<String>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;
    let sanitized_path = sanitize_path(&file_path)?;
    
    let full_path = get_user_workspace_path(user_id).join(&sanitized_path);
    
    if !full_path.exists() {
        return Err(ServiceError::NotFound);
    }
    
    if full_path.is_dir() {
        fs::remove_dir_all(&full_path).map_err(|_| ServiceError::InternalError)?;
    } else {
        fs::remove_file(&full_path).map_err(|_| ServiceError::InternalError)?;
    }
    
    Ok(Json(HashMap::from([("message".to_string(), "File deleted successfully".to_string())])))
}

pub async fn get_public_kanban_boards(
    Extension(_claims): Extension<Claims>,
) -> ServiceResult<Json<Vec<KanbanBoard>>> {
    let repo = WorkspaceRepository::new();
    let boards = repo.get_public_kanban_boards().await?;
    Ok(Json(boards))
}

pub async fn create_kanban_board(
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateKanbanBoardRequest>,
) -> ServiceResult<Json<KanbanBoard>> {
    let repo = WorkspaceRepository::new();
    let board = repo.create_kanban_board(claims.user_id, req).await?;
    Ok(Json(board))
}

pub async fn update_kanban_board(
    Extension(claims): Extension<Claims>,
    Path(board_id): Path<i64>,
    Json(req): Json<UpdateKanbanBoardRequest>,
) -> ServiceResult<Json<KanbanBoard>> {
    let repo = WorkspaceRepository::new();
    let board = repo.update_kanban_board(board_id, claims.user_id, req).await?;
    Ok(Json(board))
}

pub async fn get_kanban_board(
    Extension(_claims): Extension<Claims>,
    Path(board_id): Path<i64>,
) -> ServiceResult<Json<KanbanBoardWithItems>> {
    let repo = WorkspaceRepository::new();
    let board_option = repo.get_kanban_board_by_id(board_id).await?;
    let board = board_option.ok_or(ServiceError::NotFound)?;
    let items = repo.get_kanban_items(board_id).await?;
    let subscriber_count = 0;
    Ok(Json(KanbanBoardWithItems {
        board,
        items,
        subscriber_count,
    }))
}

pub async fn delete_kanban_board(
    Extension(claims): Extension<Claims>,
    Path(board_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let repo = WorkspaceRepository::new();
    repo.delete_kanban_board(board_id, claims.user_id).await?;
    Ok(Json(HashMap::from([("message".to_string(), "Board deleted successfully".to_string())])))
}

pub async fn share_file_to_board(
    Extension(claims): Extension<Claims>,
    Path(board_id): Path<i64>,
    Json(req): Json<ShareFileRequest>,
) -> ServiceResult<Json<KanbanItem>> {
    let repo = WorkspaceRepository::new();
    let item = repo.add_kanban_item(board_id, claims.user_id, req.file_path, "".to_string()).await?;
    Ok(Json(item))
}

pub async fn remove_file_from_board(
    Extension(claims): Extension<Claims>,
    Path(item_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let repo = WorkspaceRepository::new();
    repo.remove_kanban_item(item_id, claims.user_id).await?;
    Ok(Json(HashMap::from([("message".to_string(), "File removed successfully".to_string())])))
}

pub async fn subscribe_board(
    Extension(claims): Extension<Claims>,
    Path(board_id): Path<i64>,
) -> ServiceResult<Json<KanbanSubscription>> {
    let repo = WorkspaceRepository::new();
    let subscription = repo.subscribe_board(board_id, claims.user_id).await?;
    Ok(Json(subscription))
}

pub async fn unsubscribe_board(
    Extension(claims): Extension<Claims>,
    Path(board_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let repo = WorkspaceRepository::new();
    repo.unsubscribe_board(board_id, claims.user_id).await?;
    Ok(Json(HashMap::from([("message".to_string(), "Unsubscribed successfully".to_string())])))
}

pub async fn get_subscribed_boards(
    Extension(_claims): Extension<Claims>,
) -> ServiceResult<Json<Vec<SubscribedBoard>>> {
    Ok(Json(Vec::new()))
}

pub async fn download_shared_file(
    Extension(_claims): Extension<Claims>,
    Path((board_id, file_path)): Path<(i64, String)>,
) -> ServiceResult<impl IntoResponse> {
    let content = b"File not found".to_vec();
    let headers = [
        (HeaderName::from_static("content-type"), HeaderValue::from_static("text/plain")),
        (HeaderName::from_static("content-disposition"), HeaderValue::from_static("attachment; filename=\"error.txt\"")),
    ];
    Ok((headers, content))
}
