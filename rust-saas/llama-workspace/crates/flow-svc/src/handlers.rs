use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Flow, FlowRuntime, FlowRuntimeNode, CreateFlowRequest};
use crate::repository::FlowRepository;
use crate::state_machine::FlowStateMachine;
use crate::executor::{FlowExecutor};
use crate::flow::EXECUTOR_MANAGER;
// use once_cell::sync::Lazy;
use axum::{
    extract::{Path, Extension, Query}, 
    Json, http::{HeaderName, HeaderValue}, 
    response::IntoResponse,response::Response
};
use shared::utils::Claims;

pub async fn get_flows() -> ServiceResult<Json<Vec<Flow>>> {
    let repo = FlowRepository::new();
    let flows = repo.get_all_flows().await?;
    Ok(Json(flows))
}

pub async fn get_flow(Path(id): Path<i64>) -> ServiceResult<Json<Flow>> {
    let repo = FlowRepository::new();
    let flow = repo.get_flow(id).await?;
    Ok(Json(flow))
}

pub async fn create_flow(Json(req): Json<CreateFlowRequest>) -> ServiceResult<Json<Flow>> {
    let repo = FlowRepository::new();
    let flow = repo.create_flow(req).await?;
    Ok(Json(flow))
}

pub async fn update_flow(Path(id): Path<i64>, Json(req): Json<CreateFlowRequest>) -> ServiceResult<Json<Flow>> {
    let repo = FlowRepository::new();
    let flow = repo.update_flow(id, req).await?;
    Ok(Json(flow))
}

pub async fn delete_flow(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = FlowRepository::new();
    repo.delete_flow(id).await?;
    
    let mut manager = EXECUTOR_MANAGER.write().await;
    manager.remove(id);
    
    Ok(Json(()))
}

pub async fn start_flow(
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>
) -> ServiceResult<Json<FlowRuntime>> {
    let repo = FlowRepository::new();
    
    {
        let mut manager = EXECUTOR_MANAGER.write().await;
        if let Some(executor) = manager.get(id) {
            if executor.is_running().await {
                tracing::info!("Flow {} executor already running, stopping it first", id);
                let _ = executor.stop().await;
            }
            manager.remove(id);
        }
    }
    
    if let Some(running) = repo.get_running_flow_runtime(id).await? {
        tracing::info!("Flow {} has running runtime {}, stopping it", id, running.id);
        let _ = repo.stop_flow_runtime(running.id).await;
    }
    
    let state_machine = FlowStateMachine::new(repo.clone());
    let runtime = state_machine.start_flow(id).await?;
    tracing::info!("Flow {} runtime created: {}", id, runtime.id);
    
    let executor = Arc::new(FlowExecutor::new(claims.user_id, id, runtime.id, repo.clone()));
    {
        let mut manager = EXECUTOR_MANAGER.write().await;
        manager.insert(id, executor.clone());
        tracing::info!("Flow {} executor inserted into manager", id);
    }
    
    tracing::info!("Flow {} starting executor", id);
    let start_result = executor.start().await;
    tracing::info!("Flow {} executor.start() result: {:?}", id, start_result);
    
    {
        let manager = EXECUTOR_MANAGER.read().await;
        tracing::info!("Flow {} after start, executor in manager: {}", id, manager.get(id).is_some());
    }
    
    Ok(Json(runtime))
}


pub async fn get_flow_runtime_by_flow_id(Path(id):Path<i64>) -> ServiceResult<Json<Option<FlowRuntime>>> {
    let repo = FlowRepository::new();
    let runtime = repo.get_running_flow_runtime(id).await?;
    Ok(Json(runtime))
}

pub async fn get_flow_runtimes(Path(id): Path<i64>) -> ServiceResult<Json<Vec<FlowRuntime>>> {
    let repo = FlowRepository::new();
    let runtimes = repo.get_flow_runtimes(id).await?;
    Ok(Json(runtimes))
}

pub async fn get_flow_runtime(Path(id): Path<i64>) -> ServiceResult<Json<(FlowRuntime, Vec<FlowRuntimeNode>)>> {
    let repo = FlowRepository::new();
    let (runtime, nodes) = repo.get_flow_runtime_with_nodes(id).await?;
    Ok(Json((runtime, nodes)))
}

pub async fn stop_flow(Path(id): Path<i64>) -> ServiceResult<Json<FlowRuntime>> {
    let repo = FlowRepository::new();
    let (runtime, nodes) = repo.get_flow_runtime_with_nodes(id).await?;
    
    {
        let manager = EXECUTOR_MANAGER.read().await;
        if let Some(executor) = manager.get(id) {
            executor.stop().await?;
        }
    }
    let runtime = repo.stop_flow_runtime(runtime.id).await?;
        
    {
        let mut manager = EXECUTOR_MANAGER.write().await;
        manager.remove(id);
    }
    
    Ok(Json(runtime))
}

pub async fn get_flow_status(Path(id): Path<i64>) -> ServiceResult<Json<FlowStatusResponse>> {
    let manager = EXECUTOR_MANAGER.read().await;
    
    if let Some(executor) = manager.get(id) {
        let is_running = executor.is_running().await;
        Ok(Json(FlowStatusResponse {
            flow_id: id,
            is_running,
            message: if is_running { "Running" } else { "Stopped" }.to_string(),
        }))
    } else {
        Ok(Json(FlowStatusResponse {
            flow_id: id,
            is_running: false,
            message: "Not started".to_string(),
        }))
    }
}

#[derive(serde::Serialize)]
pub struct FlowStatusResponse {
    pub flow_id: i64,
    pub is_running: bool,
    pub message: String,
}

pub async fn complete_node(Path((runtime_id, node_id)): Path<(i64, i64)>) -> ServiceResult<Json<()>> {
    let repo = FlowRepository::new();
    let state_machine = FlowStateMachine::new(repo);
    state_machine.complete_node(runtime_id, node_id, None).await?;
    Ok(Json(()))
}

#[derive(serde::Deserialize)]
pub struct HumanInputRequest {
    pub message: String,
}

pub async fn send_human_input(Path((flow_id, node_id)): Path<(i64, i64)>, Json(req): Json<HumanInputRequest>) -> ServiceResult<Json<()>> {
    let manager = EXECUTOR_MANAGER.read().await;
    
    if let Some(executor) = manager.get(flow_id) {
        executor.send_human_input(node_id, &req.message).await?;
        Ok(Json(()))
    } else {
        tracing::error!("Flow executor not found for flow_id: {}", flow_id);
        Err(ServiceError::NotFound)
    }
}

pub async fn get_flow_runtime_sessions(Path(runtime_id): Path<i64>) -> ServiceResult<Json<Vec<crate::model::FlowRuntimeSession>>> {
    let repo = FlowRepository::new();
    let sessions = repo.get_flow_runtime_sessions_by_flow_runtime_id(runtime_id).await?;
    Ok(Json(sessions))
}

pub async fn get_flow_runtime_session_items(Path((runtime_id, session_id)): Path<(i64, i64)>) -> ServiceResult<Json<Vec<crate::model::FlowRuntimeSessionItem>>> {
    let repo = FlowRepository::new();
    let items = repo.get_flow_runtime_session_items_by_session_id(runtime_id, session_id).await?;
    Ok(Json(items))
}
