use axum::{extract::Path, Json};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Agent, AgentDetail, CreateAgentRequest};
use crate::repository::AgentRepository;

pub async fn get_agents() -> ServiceResult<Json<Vec<Agent>>> {
    let repo = AgentRepository::new();
    let agents = repo.get_all_agents().await?;
    Ok(Json(agents))
}

pub async fn get_agent(Path(id): Path<i64>) -> ServiceResult<Json<AgentDetail>> {
    let repo = AgentRepository::new();
    let agent = repo.get_agent_detail(id).await?;
    Ok(Json(agent))
}

pub async fn create_agent(Json(req): Json<CreateAgentRequest>) -> ServiceResult<Json<Agent>> {
    let repo = AgentRepository::new();
    let agent = repo.create_agent(req).await?;
    Ok(Json(agent))
}

pub async fn update_agent(Path(id): Path<i64>, Json(req): Json<CreateAgentRequest>) -> ServiceResult<Json<Agent>> {
    let repo = AgentRepository::new();
    let agent = repo.update_agent(id, req).await?;
    Ok(Json(agent))
}

pub async fn delete_agent(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AgentRepository::new();
    repo.delete_agent(id).await?;
    Ok(Json(()))
}