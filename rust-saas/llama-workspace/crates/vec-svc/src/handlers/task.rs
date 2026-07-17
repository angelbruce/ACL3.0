
use axum::{extract::{State, Path, Query}, Json};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::task_queue::{Task, TaskType, TaskStatus, NewTask};
use shared::errors::{ServiceResult, ServiceError};

#[derive(Debug, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub task_type: TaskType,
    pub payload: serde_json::Value,
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTaskRequest>,
) -> ServiceResult<Json<Task>> {
    let task = NewTask {
        task_type: req.task_type,
        payload: req.payload,
    };

    let result = state.task_queue
        .enqueue(task)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<i64>,
) -> ServiceResult<Json<Task>> {
    let result = state.task_queue
        .get_task(task_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(result))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListTasksRequest {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<Task>,
}

pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ListTasksRequest>,
) -> ServiceResult<Json<ListTasksResponse>> {
    let limit = req.limit.unwrap_or(20);
    let tasks = state.task_queue
        .list_tasks(req.status, limit)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(ListTasksResponse { tasks }))
}

pub async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<i64>,
) -> ServiceResult<Json<()>> {
    state.task_queue
        .cancel_task(task_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(()))
}

#[derive(Debug, serde::Serialize)]
pub struct TaskProgressResponse {
    pub task_id: i64,
    pub status: TaskStatus,
    pub progress: f32,
    pub message: Option<String>,
}

pub async fn get_task_progress(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<i64>,
) -> ServiceResult<Json<TaskProgressResponse>> {
    let task = state.task_queue
        .get_task(task_id)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(TaskProgressResponse {
        task_id: task.id,
        status: task.status,
        progress: task.progress,
        message: task.message,
    }))
}
