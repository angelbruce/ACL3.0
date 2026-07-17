use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::schema::{agents,agent_tools};
use std::env;
use serde::*;
use shared::models::{Agent,AgentTool,MCPTool,McpServer};

#[derive(Debug,Clone)]
pub struct AgentRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl AgentRepository {
     pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        let pool = pool.clone();
        Self { pool }
    }

    pub fn get_pool(&self) -> &r2d2::Pool<ConnectionManager<PgConnection>> {
        &self.pool
    }


    pub async fn get_agent_by_id(&self, id: i64) -> ServiceResult<Agent> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let agent = agents::table
            .filter(agents::id.eq(id))        
            .first::<Agent>(&mut conn)?;
        Ok(agent)
    }

    pub async fn get_agent_tools(&self, id: i64) -> ServiceResult<Vec<MCPTool>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let agent = agents::table.first::<Agent>(&mut conn)?;

        let tools =  agent_tools::table
            .filter(agent_tools::agent_id.eq(id))
            .load::<AgentTool>(&mut conn)?;

        let mut mcp_tools = vec![];
        for x in tools {
            mcp_tools.push(MCPTool{
                name : x.name,
                description : x.description,
                input_schema : serde_json::json!(x.input_schema),
                output_schema : serde_json::json!(x.output_schema),
                server_id : x.server_id,
            });
        }

        Ok(mcp_tools)
    }

    pub async fn get_mcp_servers(&self) -> ServiceResult<Vec<McpServer>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let mcp_servers = shared::schema::mcp_servers::table
            .filter(shared::schema::mcp_servers::enabled.eq(true))
            .order( shared::schema::mcp_servers::id.asc())
            .load::<McpServer>(&mut conn)?;
        Ok(mcp_servers)
    }
}