use axum::{extract::{Path, Extension}, Json, http::{HeaderName, HeaderValue}, response::IntoResponse,response::Response};
use axum::body::Body;
use chrono::{Utc, DateTime};
use mime_guess::from_path;
use serde::{Serialize, Deserialize};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{
    WorkspaceFile, KanbanBoard, KanbanBoardWithItems, KanbanItem, KanbanSubscription, SubscribedBoard, 
    CreateKanbanBoardRequest, UpdateKanbanBoardRequest, ShareFileRequest,
    Project, ProjectFile, ProjectMessage, ProjectSummary, ProjectWithNames,
    CreateProjectRequest as SharedCreateProjectRequest, UpdateProjectRequest as SharedUpdateProjectRequest,
    CreateProjectFileRequest, UpdateProjectFileRequest, AddProjectMessageRequest,
    CreateOrUpdateProjectSummaryRequest, ProjectContainerConfig
};
use shared::utils::Claims;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use tokio::runtime::Runtime; 
use tokio::task;
use futures::Stream;
use crate::repository::WorkspaceRepository;
use crate::voice::{Article};

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
) -> ServiceResult<Json<Vec<ProjectWithNames>>> {
    let repo = WorkspaceRepository::new();
    let projects = repo.get_projects_by_user(claims.user_id).await?;
    Ok(Json(projects))
}

pub async fn get_project(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<ProjectWithNames>> {
    let repo = WorkspaceRepository::new();
    let project = repo.get_project_by_id(project_id).await?;
    match project {
        Some(p) if p.user_id == claims.user_id => Ok(Json(p)),
        _ => Err(ServiceError::NotFound),
    }
}

pub async fn create_project(
    Extension(claims): Extension<Claims>,
    Json(req): Json<SharedCreateProjectRequest>,
) -> ServiceResult<Json<Project>> {
    let repo = WorkspaceRepository::new();
    let project = repo.create_project(claims.user_id, req).await?;
    Ok(Json(project))
}

pub async fn update_project(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<SharedUpdateProjectRequest>,
) -> ServiceResult<Json<Project>> {
    let repo = WorkspaceRepository::new();
    let project = repo.update_project(project_id, claims.user_id, req).await?;
    Ok(Json(project))
}

pub async fn delete_project(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let repo = WorkspaceRepository::new();
    repo.delete_project(project_id, claims.user_id).await?;
    Ok(Json(HashMap::from([("message".to_string(), "Project deleted successfully".to_string())])))
}

pub async fn list_project_files(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<ProjectFile>>> {
    let repo = WorkspaceRepository::new();
    let data = repo.get_project_files(project_id, claims.user_id).await;
    if data.is_err() {
        return Err(data.err().unwrap());
    }
    let files = data.unwrap_or_default();  
    Ok(Json(files))
}

pub async fn create_project_file(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<CreateProjectFileRequest>,
) -> ServiceResult<Json<ProjectFile>> {
    let repo = WorkspaceRepository::new();
    let file = repo.create_project_file(project_id, claims.user_id, req).await?;   

    Ok(Json(file))
}

pub async fn update_project_file(
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i64>,
    Json(req): Json<UpdateProjectFileRequest>,
) -> ServiceResult<Json<ProjectFile>> {
    let repo = WorkspaceRepository::new();
    let content = &req.content.clone();
    let file = repo.update_project_file(file_id, claims.user_id, req).await?;
    let project = repo.get_project_by_id(file.project_id).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    if project.is_none() {
        return Err(ServiceError::NotFound);
    }

    if !content.is_empty() && project.unwrap().purpose == "article" {
        let article = Article {
            user_id: claims.user_id,
            project_id: file.project_id,
            article_id: file.id,
            content: content.clone(),
            voice_type: String::from("xtts"),
            voice_seed: 1,
            voice_speed: 1.5,
        };

        task::spawn( async move {
            println!("{:?}", article);  
            match Article::create_voice(article.clone()).await {
                Ok(_) => println!("Voice created successfully"),
                Err(e) => println!("Error creating voice: {:?}", e),
            }
        });
    }

    Ok(Json(file))
}

pub async fn get_project_file_voice(
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i64>,
) -> ServiceResult<Body> {
    let repo = WorkspaceRepository::new();
    let file = repo.get_project_file_by_id(file_id, claims.user_id).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let file_path = Article::get_voice_path(claims.user_id, file.project_id, file.id, "xtts".to_string(), 1).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let file_stream = Article::create_file_stream(file_path.clone(), 8192);
    let body: Body = Body::from_stream(file_stream);
    Ok(body)
}

pub async fn get_project_file_voice_link(
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i64>,
) -> ServiceResult<Json<String>> {

    let repo = WorkspaceRepository::new();
    let file = repo.get_project_file_by_id(file_id, claims.user_id).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let file_path = Article::get_voice_link_path(claims.user_id, file.project_id, file.id, "xtts".to_string(), 1).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    Ok(Json(file_path.to_string()))
}


pub async fn delete_project_file(
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let repo = WorkspaceRepository::new();
    repo.delete_project_file(file_id, claims.user_id).await?;
    Ok(Json(HashMap::from([("message".to_string(), "File deleted successfully".to_string())])))
}

pub async fn get_project_messages(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<ProjectMessage>>> {
    let repo = WorkspaceRepository::new();
    let messages = repo.get_project_messages(project_id, claims.user_id).await?;
    Ok(Json(messages))
}

pub async fn add_project_message(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<AddProjectMessageRequest>,
) -> ServiceResult<Json<ProjectMessage>> {
    let repo = WorkspaceRepository::new();
    let message = repo.add_project_message(project_id, claims.user_id, req).await?;
    Ok(Json(message))
}

pub async fn get_project_summaries(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<ProjectSummary>>> {
    let repo = WorkspaceRepository::new();
    let summaries = repo.get_project_summaries(project_id, claims.user_id).await?;
    Ok(Json(summaries))
}

pub async fn create_or_update_project_summary(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<CreateOrUpdateProjectSummaryRequest>,
) -> ServiceResult<Json<ProjectSummary>> {
    let repo = WorkspaceRepository::new();
    let summary = repo.create_or_update_project_summary(project_id, claims.user_id, req).await?;
    Ok(Json(summary))
}

pub async fn list_workspace_files(
    Extension(claims): Extension<Claims>,
) -> ServiceResult<Json<Vec<WorkspaceFile>>> {
    let user_id = claims.user_id;
    let workspace_path = get_user_workspace_path(user_id);
    
    if !workspace_path.exists() {
        fs::create_dir_all(&workspace_path).map_err(|_| ServiceError::InternalError)?;
        return Ok(Json(Vec::new()));
    }
    
    let mut files: Vec<WorkspaceFile> = Vec::new();
    
    collect_files_recursive(&workspace_path, &workspace_path, user_id, &mut files)?;
    
    files.sort_by(|a, b| {
        if a.is_directory != b.is_directory {
            a.is_directory.cmp(&b.is_directory).reverse()
        } else {
            a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
        }
    });
    
    Ok(Json(files))
}

fn collect_files_recursive(base_path: &PathBuf, current_path: &PathBuf, user_id: i64, files: &mut Vec<WorkspaceFile>) -> ServiceResult<()> {
    for entry in fs::read_dir(current_path).map_err(|_| ServiceError::InternalError)? {
        let entry = entry.map_err(|_| ServiceError::InternalError)?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|_| ServiceError::InternalError)?;
        
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let relative_path = path.strip_prefix(base_path).unwrap_or(&path).to_string_lossy().to_string();
        
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
        
        if metadata.is_dir() {
            collect_files_recursive(base_path, &path, user_id, files)?;
        }
    }
    Ok(())
}

pub async fn list_project_workspace_files(
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

pub async fn get_project_container_config(
    Extension(claims): Extension<Claims>,
    Path(project_id):Path<i64>,
) -> ServiceResult<Json<Vec<ProjectContainerConfig>>> {
    let repo = WorkspaceRepository::new();
    println!("{:?}", project_id);
    let config = repo.get_project_container_config(project_id).await;    
    match config {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
           Ok(Json(Vec::new()))
        },
    }
}


pub async fn save_project_container_config(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<Vec<ProjectContainerConfig>>,    
) -> ServiceResult<Json<Vec<ProjectContainerConfig>>> {
    let repo = WorkspaceRepository::new();
    let config = repo.save_project_container_config(claims.user_id, project_id, req).await?;
    Ok(Json(config))
}

pub async fn start_container(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    //todo: 实现启动容器的逻辑
    Ok(Json(HashMap::from([("message".to_string(), "Container started successfully".to_string())])))
}


//暂时不考虑越权问题 横向与纵向都不考虑。