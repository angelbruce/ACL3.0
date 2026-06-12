use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Agent, CreateFlowRequest, Flow, FlowRuntime, FlowRuntimeNode, LlmModel, NodeStatus};
use shared::schema::{flows, flow_runtimes, flow_runtime_nodes, agents};
use std::env;

pub struct AgentRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl AgentRepository {
     pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        let pool = pool.clone();
        Self { pool }
    }


    pub async fn get_agent_by_id(&self, id: i64) -> ServiceResult<Agent> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let agent = agents::table
            .filter(agents::id.eq(id))        
            .first::<Agent>(&mut conn)?;
        Ok(agent)
    }
}