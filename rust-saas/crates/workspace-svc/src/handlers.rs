use axum::{extract::{Path, Extension, Query}, Json, http::{HeaderName, HeaderValue}, response::IntoResponse,response::Response};
use axum::body::Body;
use axum::response::sse::{Event, Sse};
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
    CreateOrUpdateProjectSummaryRequest, ProjectContainerConfig, ChatMessage, MCPTool, LlmTool, LlmToolFunction,
    FileContainerAssignment, NewFileContainerAssignment, FileAssignmentRequest, FileAssignmentResult,
    FileAssignmentInfo, ContainerConfigInfo
};
use shared::utils::Claims;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::pin::Pin;
use std::thread;
use tokio::runtime::Runtime; 
use tokio::task;
use futures::{Stream, StreamExt};
use crate::repository::WorkspaceRepository;
use crate::voice::{Article};
use crate::llm_client::{LlmClient, StreamResponse, ToolExecutor};
use crate::container::{
    ProjectDeploymentContext, ProjectFileInfo, ContainerDeploymentResult,
    ContainerDeployer, ExecuteCommandResult, ContainerStatus,
    format_container_name, get_debug_directory
};
use crate::model::SaveProjectConfigPathRarams;

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

/// 刷新项目文件到容器并执行命令
#[derive(Debug, Deserialize)]
pub struct RefreshFileRequest {
    pub file_id: i64,
    pub config_id: i64,
    pub content: String,
    pub command: String,
}

pub async fn refresh_project_file_to_container(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<RefreshFileRequest>,
) -> ServiceResult<Json<RefreshFileResult>> {
    let result = ContainerDeployer::refresh_file_and_execute(
        claims.user_id,
        project_id,
        req.config_id,
        req.file_id,
        &req.content,
        &req.command,
    ).await;
    
    match result {
        Ok(output) => Ok(Json(RefreshFileResult {
            success: true,
            message: "File refreshed and command executed".to_string(),
            output,
        })),
        Err(e) => Err(ServiceError::BadRequest(e)),
    }
}

#[derive(Debug, Serialize)]
pub struct RefreshFileResult {
    pub success: bool,
    pub message: String,
    pub output: String,
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
    Query(params): Query<SaveProjectConfigPathRarams>,
    Json(req): Json<Vec<ProjectContainerConfig>>,    
) -> ServiceResult<Json<Vec<ProjectContainerConfig>>> {
    let fetch = params.fetch;
    println!("([save_project_container_config] fetch: {}", fetch); 
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    println!("[save_project_container_config] Stopping existing containers for project_id: {}", project_id);
    let debug_dir = get_debug_directory(user_id, project_id);
    let _ = ContainerDeployer::stop_containers(&debug_dir).await;

    println!("[save_project_container_config] Saving container configs for project_id: {}", project_id);
    let config =    if !fetch {   repo.save_project_container_config(user_id, project_id, req).await? }
                                                else { repo.replace_project_container_config(user_id, project_id, req).await? };    

    println!("[save_project_container_config] Starting containers for project_id: {}", project_id);
    let project = repo.get_project_info_for_deployment(project_id).await?
        .ok_or(ServiceError::NotFound)?;

    let container_configs = repo.get_project_container_config(project_id).await?;
    let project_files = repo.get_project_files_info(project_id).await?;

    let deploy_result = ContainerDeployer::deploy_project(
        user_id,
        project_id,
        project.name.clone(),
        project.agent_id,
        project.model_id,
        container_configs.clone(),
        project_files.clone(),
    ).await
    .map_err(|e| ServiceError::InternalError)?;

    let compose_config_content = std::fs::read_to_string(&deploy_result.docker_compose_path)
        .map_err(|_| ServiceError::InternalError)?;

    let existing_compose_config = repo.get_project_container_config(project_id).await?
        .into_iter()
        .find(|c| c.container_name == "docker-compose.yml");

    if let Some(mut cfg) = existing_compose_config {
        cfg.command = compose_config_content;
        cfg.updated_at = chrono::Utc::now().naive_utc();
        repo.update_project_container_config(cfg).await?;
    } else {
        let compose_config_entry = ProjectContainerConfig {
            id: 0,
            project_id,
            project_dir: "/debug".to_string(),
            published_ports: "".to_string(),
            volumes: "".to_string(),
            environment: "".to_string(),
            command: compose_config_content,
            working_dir: "".to_string(),
            tags: "".to_string(),
            container_name: "docker-compose.yml".to_string(),
            cpu_usage: "".to_string(),
            memory_usage: "".to_string(),
            image_name: "".to_string(),
            creator_id: user_id,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        repo.insert_project_container_config(user_id, project_id, compose_config_entry).await?;
    }

    let final_configs = repo.get_project_container_config(project_id).await?;

    task::spawn(async move {
        match ContainerDeployer::start_containers(&debug_dir).await {
            Ok(result) => {
                println!("[save_project_container_config] Docker compose up output: {}", result.output);
                println!("[save_project_container_config] Port mappings: {:?}", result.port_mappings);
            }
            Err(e) => {
                println!("[save_project_container_config] Failed to start containers: {}", e);
            }
        }
    });

    Ok(Json(final_configs))
}

pub async fn start_container(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<ContainerDeploymentResult>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    println!("[start_container] Received request for project_id: {}, user_id: {}", project_id, user_id);

    let project = repo.get_project_info_for_deployment(project_id).await?
        .ok_or(ServiceError::NotFound)?;

    println!("[start_container] Project found: {:?}, agent_id: {:?}, model_id: {:?}", project.name, project.agent_id, project.model_id);

    let container_configs = repo.get_project_container_config(project_id).await?;
    println!("[start_container] Container configs count: {}", container_configs.len());

    let project_files = repo.get_project_files_info(project_id).await?;
    println!("[start_container] Project files count: {}", project_files.len());

    let unassigned_files = repo.check_unassigned_files(project_id).await?;
    let mut unassigned_file_id_map = HashMap::new();
    for f in &unassigned_files {
        unassigned_file_id_map.insert(f.id, f.id);
    }
    println!("[start_container] Unassigned files: {:?}", unassigned_files);
    //代码按照项目进行分组
    if !unassigned_files.is_empty() {
        println!("[start_container] Found {} unassigned files, starting LLM file assignment...", unassigned_files.len());
        
        let file_infos = repo.get_file_assignment_info(project_id).await?;
        let container_config_infos = repo.get_container_config_info(project_id).await?;

        if !container_config_infos.is_empty() && !file_infos.is_empty() {
            if let Some(model_id) = project.model_id {
                let model = repo.get_model(model_id).await?;
                let llm_client = LlmClient::new(&model.access_url, &model.api_key, &model.name);
                
                let assignments = llm_client.assign_files_to_containers(file_infos, container_config_infos, project_id).await?;
                println!("code classifed by project over LLM completed!!!");
                let new_assignments: Vec<NewFileContainerAssignment> = assignments
                    .iter()
                    .filter(|a| unassigned_file_id_map.contains_key(&a.file_id))
                    .flat_map(|a| {
                        a.container_config_ids.iter().map(move |config_id| {
                            NewFileContainerAssignment {
                                project_id,
                                file_id: a.file_id,
                                container_config_id: *config_id,
                                file_path: a.file_path.clone(),
                                assigned_by: "llm".to_string(),
                                confidence_score: a.confidence_score,
                                assignment_reason: Some(a.assignment_reason.clone()),
                            }
                        })
                    })
                    .collect();
                
                repo.save_file_assignments(project_id, new_assignments).await?;
                println!("[start_container] LLM file assignment completed successfully");
            } else {
                println!("[start_container] No model configured, skipping file assignment");
            }
        }
    }

    let deploy_result = ContainerDeployer::deploy_project(
        user_id,
        project_id,
        project.name.clone(),
        project.agent_id,
        project.model_id,
        container_configs.clone(),
        project_files.clone(),
    ).await
    .map_err(|e| ServiceError::InternalError)?;

    println!("[start_container] Deploy result: {:?}", deploy_result);

    let ids = container_configs.clone().into_iter().filter(|c| c.container_name != "docker-compose.yml")
        .map(|c| c.id).collect::<Vec<_>>();

    let first_container_id = match ids.len() {
                                0 => 0,
                                _ => ids[0],
                            };
    let container_name = format!("{}-{}-{}", user_id, project_id, first_container_id);
        
    let mcp_server_url = format!("http://{}:80", container_name);
    // let mcp_sse_port = container_configs.iter()
    //     .find(|c| c.id == first_container_id)
    //     .map(|c| {
    //         if !c.environment.is_empty() {
    //             let envs: Vec<&str> = c.environment.split(',').collect();
    //             for env in envs {
    //                 let trimmed = env.trim();
    //                 if trimmed.starts_with("MCP_SSE_PORT=") {
    //                     return trimmed.split('=').nth(1).unwrap_or(crate::container::DEFAULT_MCP_SSE_PORT).to_string();
    //                 }
    //             }
    //         }
    //         crate::container::DEFAULT_MCP_SSE_PORT.to_string()
    //     })
    //     .unwrap_or(crate::container::DEFAULT_MCP_SSE_PORT.to_string());
    
    // let mcp_server_url = format!("http://{}:{}", container_name, mcp_sse_port);

    let debug_dir = PathBuf::from(&deploy_result.debug_dir);

    let context = ProjectDeploymentContext {
        user_id,
        project_id,
        project_name: project.name,
        agent_id: project.agent_id,
        model_id: project.model_id,
        container_configs: container_configs.clone(),
        project_files: project_files.clone(),
        container_name: container_name.clone(),
        mcp_server_url,
        debug_dir: debug_dir.clone(),
    };

    let compose_config_content = std::fs::read_to_string(&deploy_result.docker_compose_path)
        .map_err(|_| ServiceError::InternalError)?;

    let existing_configs: Vec<ProjectContainerConfig> = container_configs
        .into_iter()
        .filter(|c| c.container_name != "docker-compose.yml")
        .collect();

    repo.save_project_container_config(user_id, project_id, existing_configs).await?;

    let existing_compose_config = repo.get_project_container_config(project_id).await?
        .into_iter()
        .find(|c| c.container_name == "docker-compose.yml");

    if let Some(mut config) = existing_compose_config {
        config.command = compose_config_content;
        config.updated_at = chrono::Utc::now().naive_utc();
        repo.update_project_container_config(config).await?;
    } else {
        let compose_config_entry = ProjectContainerConfig {
            id: 0,
            project_id,
            project_dir: "/debug".to_string(),
            published_ports: "".to_string(),
            volumes: "".to_string(),
            environment: "".to_string(),
            command: compose_config_content,
            working_dir: "".to_string(),
            tags: "".to_string(),
            container_name: "docker-compose.yml".to_string(),
            cpu_usage: "".to_string(),
            memory_usage: "".to_string(),
            image_name: "".to_string(),
            creator_id: user_id,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        repo.insert_project_container_config(user_id, project_id, compose_config_entry).await?;
    }

    task::spawn(async move {
        println!("[start_container] Spawning async task to start containers...");
        println!("[start_container] Context debug: user_id={}, project_id={}, model_id={:?}", 
                 context.user_id, context.project_id, context.model_id);
        println!("[start_container] Project files count: {}", context.project_files.len());
        
        match ContainerDeployer::start_containers(&debug_dir).await {
            Ok(result) => {
                println!("[start_container] Docker compose up output: {}", result.output);
                println!("[start_container] Port mappings: {:?}", result.port_mappings);
                println!("[start_container] Calling LLM for deployment...");
                match ContainerDeployer::call_llm_for_deployment(&context).await {
                    Ok(response) => println!("[start_container] LLM deployment call completed: {}", response),
                    Err(e) => println!("[start_container] LLM deployment call failed: {}", e),
                }
            }
            Err(e) => println!("[start_container] Failed to start docker compose: {}", e),
        }
    });

    println!("[start_container] Returning success response");

    Ok(Json(ContainerDeploymentResult {
        success: true,
        message: "Deployment started successfully".to_string(),
        debug_dir: deploy_result.debug_dir,
        container_names: deploy_result.container_names,
        docker_compose_path: deploy_result.docker_compose_path,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteCommandRequest {
    pub project_id: i64,
    pub config_id: i64,
    pub command: String,
}

pub async fn execute_command(
    Extension(claims): Extension<Claims>,
    Json(req): Json<ExecuteCommandRequest>,
) -> ServiceResult<Json<ExecuteCommandResult>> {
    let user_id = claims.user_id;

    let result = ContainerDeployer::execute_command_in_container(
        user_id,
        req.project_id,
        req.config_id,
        &req.command,
    ).await
    .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

pub async fn execute_command_stream_handler(
    Extension(claims): Extension<Claims>,
    Json(req): Json<ExecuteCommandRequest>,
) -> ServiceResult<Response<Body>> {
    let user_id = claims.user_id;

    let stream = ContainerDeployer::execute_command_stream(
        user_id,
        req.project_id,
        req.config_id,
        &req.command,
    ).await
    .map_err(|e| ServiceError::InternalError)?;

    let sse_stream = futures::stream::StreamExt::map(stream, |result| {
        let data = match result {
            Ok(line) => format!("data: {}\n\n", line),
            Err(e) => format!("data: {}\n\n", e),
        };
        Ok::<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>(bytes::Bytes::from(data))
    });

    let body = Body::from_stream(sse_stream);

    let response = Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(body)
        .map_err(|e| ServiceError::InternalError)?;

    Ok(response)
}

pub async fn stop_container(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;
    let debug_dir = get_debug_directory(user_id, project_id);

    let result = ContainerDeployer::stop_containers(&debug_dir).await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(HashMap::from([("message".to_string(), result)])))
}

pub async fn get_container_status(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Query(params): Query<GetContainerStatusParams>,
) -> ServiceResult<Json<ContainerStatusResponse>> {
    let user_id = claims.user_id;

    let statuses = ContainerDeployer::get_container_status(user_id, project_id).await
        .map_err(|e| ServiceError::InternalError)?;

    let target_status = params.config_id.and_then(|config_id| {
        let full_container_name = format!("{}-{}-{}", user_id, project_id, config_id);
        statuses.iter().find(|s| s.name == full_container_name || s.name.contains(&full_container_name)).cloned()
    });

    Ok(Json(ContainerStatusResponse {
        statuses,
        target_status,
    }))
}

#[derive(Debug, Deserialize)]
pub struct GetContainerStatusParams {
    pub config_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ContainerStatusResponse {
    pub statuses: Vec<ContainerStatus>,
    pub target_status: Option<ContainerStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetContainerLogsRequest {
    pub container_name: Option<String>,
    pub tail: Option<usize>,
}

pub async fn get_container_logs(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<GetContainerLogsRequest>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;

    let logs = ContainerDeployer::get_container_logs(
        user_id,
        project_id,
        req.container_name.as_deref(),
        req.tail,
    ).await
    .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(HashMap::from([("logs".to_string(), logs)])))
}

pub async fn cleanup_container(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;

    let result = ContainerDeployer::cleanup_debug_directory(user_id, project_id).await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(HashMap::from([("message".to_string(), result)])))
}


//暂时不考虑越权问题 横向与纵向都不考虑。

// =============================================
// Workspace 专用的 Chat/Stream 接口
// =============================================

type SSEStream = Pin<Box<dyn Stream<Item = Result<Event, std::convert::Infallible>> + Send>>;

#[derive(Debug, Deserialize)]
pub struct WorkspaceChatRequest {
    pub model_id: i64,
    pub agent_id: Option<i64>,
    pub project_id: i64,
    pub config_id: i64,
    pub messages: Vec<ChatMessage>,
}

/// Workspace 专用的 Chat/Stream 接口
/// 从数据库读取工具配置，并通过容器的 MCP-SSE 服务执行工具调用
pub async fn workspace_chat_stream(
    Extension(claims): Extension<Claims>,
    Json(req): Json<WorkspaceChatRequest>,
) -> Sse<SSEStream> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();
    
    // 1. 获取模型信息
    let model = match repo.get_model(req.model_id).await {
        Ok(m) => m,
        Err(e) => {
            let error_msg = format!("{{\"error\": \"Model not found: {}\"}}", e);
            let stream: SSEStream = Box::pin(futures::stream::once(async move {
                Ok::<Event, std::convert::Infallible>(Event::default().data(error_msg))
            }));
            return Sse::new(stream);
        }
    };
    
    // 2. 获取容器 MCP-SSE URL
    let container_name = format!("{}-{}-{}", user_id, req.project_id, req.config_id);
    let mcp_server_url = format!("http://{}:80", container_name);
    // let mcp_sse_port = match repo.get_container_config(req.config_id).await {
    //     Ok(config) => {
    //         if !config.environment.is_empty() {
    //             let envs: Vec<&str> = config.environment.split(',').collect();
    //             for env in envs {
    //                 let trimmed = env.trim();
    //                 if trimmed.starts_with("MCP_SSE_PORT=") {
    //                     trimmed.split('=').nth(1).unwrap_or(crate::container::DEFAULT_MCP_SSE_PORT).to_string()
    //                 } else {
    //                     continue
    //                 }
    //             }
    //         }
    //         crate::container::DEFAULT_MCP_SSE_PORT.to_string()
    //     },
    //     Err(_) => crate::container::DEFAULT_MCP_SSE_PORT.to_string(),
    // };
    
    // let mcp_server_url = format!("http://{}:{}", container_name, mcp_sse_port);
    
    // 3. 从容器 MCP-SSE 服务动态获取工具列表
    let tool_executor = ToolExecutor::new(HashMap::new(), &mcp_server_url, None);
    let mut tools = match tool_executor.list_tools(None).await {
        Ok(t) => {
            println!("[DEBUG] Retrieved {} tools from container MCP-SSE", t.len());
            t
        },
        Err(e) => {
            println!("[DEBUG] Failed to get tools from container: {}, using default tools", e);
            // 使用默认工具列表，因为容器 MCP-SSE 服务可能不支持 tools/list 方法
            vec![
                MCPTool {
                    name: "execute_command".to_string(),
                    description: "Execute a shell command in the debug container. Use this to run commands like npm install, npm run serve, etc.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute"
                            },
                            "workDir": {
                                "type": "string",
                                "description": "The working directory for the command, defaults to /app"
                            }
                        },
                        "required": ["command"]
                    }),
                    output_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "output": {
                                "type": "string",
                                "description": "The command output"
                            },
                            "exit_code": {
                                "type": "integer",
                                "description": "The exit code of the command"
                            }
                        }
                    }),
                    server_id: None,
                }
            ]
        }
    };
    
    // 4. 如果有 agent_id，获取 agent 的系统提示词和工具
    let mut messages = req.messages.clone();
    if let Some(agent_id) = req.agent_id {
        println!("[DEBUG] Using agent_id: {}", agent_id);
        
        if let Ok(Some(system_prompt)) = repo.get_agent_system_prompt(agent_id).await {
            println!("[DEBUG] Got agent system prompt, length: {}", system_prompt.len());
            messages.insert(0, ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt),
                ..Default::default()
            });
        }
        
        if let Ok(agent_tools) = repo.get_agent_tools(agent_id).await {
            println!("[DEBUG] Got {} agent tools", agent_tools.len());
            tools.extend(agent_tools);
        }
    }
    
    // 5. 创建 LLM 客户端
    let llm_client = LlmClient::new(&model.access_url, &model.api_key, &model.name);
    
    // 5. 创建空 MCP 服务器映射，工具调用使用默认 URL
    let mcp_servers: HashMap<i64, String> = HashMap::new();
    
    // 6. 调用 LLM + 工具执行循环
    let stream: Pin<Box<dyn Stream<Item = StreamResponse> + Send>> = if tools.is_empty() {
        // 没有工具，使用普通聊天流
        match llm_client.chat_stream(&messages, None).await {
            Ok(s) => Box::pin(s) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>,
            Err(e) => {
                let stream: SSEStream = Box::pin(futures::stream::once(async move {
                    Ok::<Event, std::convert::Infallible>(Event::default().data(format!("{{\"error\": \"LLM error: {}\"}}", e)))
                }));
                return Sse::new(stream);
            }
        }
    } else {
        // 有工具，使用工具执行循环
        match llm_client.chat_with_tools(
            messages,
            Some(&tools),
            mcp_servers,
            &mcp_server_url,
            10,
            user_id,
            req.project_id,
            Some(req.config_id),
            None
        ).await {
            Ok(s) => Box::pin(s) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>,
            Err(e) => {
                let stream: SSEStream = Box::pin(futures::stream::once(async move {
                    Ok::<Event, std::convert::Infallible>(Event::default().data(format!("{{\"error\": \"LLM error: {}\"}}", e)))
                }));
                return Sse::new(stream);
            }
        }
    };
    
    // 8. 转换为 SSE 事件流
    let sse_stream: SSEStream = Box::pin(stream.map(|res: StreamResponse| {
        match serde_json::to_string(&res) {
            Ok(json_str) => Ok::<Event, std::convert::Infallible>(Event::default().data(json_str)),
            Err(_) => Ok::<Event, std::convert::Infallible>(Event::default().data("{\"error\": \"Serialization error\"}")),
        }
    }));
    
    Sse::new(sse_stream)
}


#[derive(Debug, Serialize)]
pub struct MessageDeleteProjectMessageResponse {
    pub project_id: i64,
    pub message_id: i64,
    pub message: String,
}

pub async fn delete_project_message(
    Extension(claims): Extension<Claims>,
    Path((project_id, message_id)): Path<(i64, i64)>,
) -> ServiceResult<Json<MessageDeleteProjectMessageResponse>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();
    repo.delete_project_message(project_id, message_id).await?;
    Ok(Json(MessageDeleteProjectMessageResponse {
        project_id,
        message_id,
        message: "Message deleted".to_string(),
    }))
}

// ============================================
// 文件容器分配相关接口
// ============================================

#[derive(Debug, Deserialize)]
pub struct AssignFilesRequest {
    pub force: Option<bool>,
}

pub async fn assign_files_to_containers(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Query(params): Query<AssignFilesRequest>,
) -> ServiceResult<Json<Vec<FileAssignmentResult>>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    println!("[assign_files_to_containers] User: {}, Project: {}", user_id, project_id);

    let project = repo.get_project_by_id(project_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    if project.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }

    let unassigned_files = repo.check_unassigned_files(project_id).await?;
    let force = params.force.unwrap_or(false);

    if unassigned_files.is_empty() && !force {
        println!("[assign_files_to_containers] No unassigned files found and force=false, returning existing assignments");
        let assignments = repo.get_file_assignments(project_id).await?;
        let results: Vec<FileAssignmentResult> = assignments
            .into_iter()
            .map(|a| FileAssignmentResult {
                file_id: a.file_id,
                file_path: a.file_path,
                container_config_ids: vec![a.container_config_id],
                confidence_score: a.confidence_score,
                assignment_reason: a.assignment_reason.unwrap_or_default(),
            })
            .collect();
        return Ok(Json(results));
    }

    let file_infos = repo.get_file_assignment_info(project_id).await?;
    let container_configs = repo.get_container_config_info(project_id).await?;

    if container_configs.is_empty() {
        return Err(ServiceError::BadRequest("No container configs found for project".to_string()));
    }

    if file_infos.is_empty() {
        return Err(ServiceError::BadRequest("No files found for project".to_string()));
    }
    println!("12");

    let model_id = project.model_id
        .ok_or(ServiceError::BadRequest("Project has no model configured".to_string()))?;

    let model = repo.get_model(model_id).await?;

    let llm_client = LlmClient::new(&model.access_url, &model.api_key, &model.name);

    println!("[assign_files_to_containers] Calling LLM for file assignment...");
    println!("[assign_files_to_containers] Files count: {}, Configs count: {}", file_infos.len(), container_configs.len());

    let assignments = llm_client.assign_files_to_containers(file_infos, container_configs, project_id).await?;
    println!("11");
    println!("[assign_files_to_containers] LLM returned {} assignments", assignments.len());

    let new_assignments: Vec<NewFileContainerAssignment> = assignments
        .iter()
        .flat_map(|a| {
            a.container_config_ids.iter().map(move |config_id| {
                NewFileContainerAssignment {
                    project_id,
                    file_id: a.file_id,
                    container_config_id: *config_id,
                    file_path: a.file_path.clone(),
                    assigned_by: "llm".to_string(),
                    confidence_score: a.confidence_score,
                    assignment_reason: Some(a.assignment_reason.clone()),
                }
            })
        })
        .collect();

    if force {
        repo.delete_project_file_assignments(project_id).await?;
    }

    repo.save_file_assignments(project_id, new_assignments).await?;

    Ok(Json(assignments))
}

pub async fn get_file_assignments(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<FileContainerAssignment>>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    let project = repo.get_project_by_id(project_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    if project.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }

    let assignments = repo.get_file_assignments(project_id).await?;
    Ok(Json(assignments))
}

pub async fn get_files_by_container(
    Extension(claims): Extension<Claims>,
    Path((project_id, container_config_id)): Path<(i64, i64)>,
) -> ServiceResult<Json<Vec<FileContainerAssignment>>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    let project = repo.get_project_by_id(project_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    if project.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }

    let assignments = repo.get_files_by_container(project_id, container_config_id).await?;
    Ok(Json(assignments))
}

pub async fn get_shared_files(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
) -> ServiceResult<Json<Vec<FileContainerAssignment>>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    let project = repo.get_project_by_id(project_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    if project.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }

    let assignments = repo.get_shared_files(project_id).await?;
    Ok(Json(assignments))
}

pub async fn update_file_assignment(
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<FileAssignmentRequest>,
) -> ServiceResult<Json<FileContainerAssignment>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    let project = repo.get_project_by_id(project_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    if project.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }

    let file = repo.get_project_file_by_id(req.file_id, user_id).await?;
    if file.project_id != project_id {
        return Err(ServiceError::BadRequest("File does not belong to project".to_string()));
    }

    let new_assignments: Vec<NewFileContainerAssignment> = req.container_config_ids
        .iter()
        .map(|config_id| {
            NewFileContainerAssignment {
                project_id,
                file_id: req.file_id,
                container_config_id: *config_id,
                file_path: req.file_path.clone(),
                assigned_by: "manual".to_string(),
                confidence_score: 100.0,
                assignment_reason: req.assignment_reason.clone(),
            }
        })
        .collect();

    let existing_assignments = repo.get_file_assignment_by_file(req.file_id).await?;
    for existing in existing_assignments {
        if !req.container_config_ids.contains(&existing.container_config_id) {
            repo.delete_file_assignments(existing.file_id).await?;
        }
    }

    let saved = repo.save_file_assignments(project_id, new_assignments).await?;
    
    Ok(Json(saved.into_iter().next().unwrap()))
}

pub async fn delete_file_assignment(
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i64>,
) -> ServiceResult<Json<HashMap<String, String>>> {
    let user_id = claims.user_id;
    let repo = WorkspaceRepository::new();

    let file = repo.get_project_file_by_id(file_id, user_id).await?;
    repo.delete_file_assignments(file_id).await?;

    Ok(Json(HashMap::from([("message".to_string(), "File assignments deleted successfully".to_string())])))
}