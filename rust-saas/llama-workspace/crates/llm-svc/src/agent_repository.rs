use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{AgentDetail, AgentSkill, AgentTool, Agent, ContentStoreConfig, MCPTool, McpServer};
use shared::schema::{agent_skills, agent_tools, agents, content_store_configs, mcp_servers};
use std::collections::HashMap;
use std::env;

pub struct AgentRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl AgentRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        AgentRepository { pool }
    }

    pub async fn get_agent_detail(&self, id: i64) -> ServiceResult<AgentDetail> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let agent = agents::table
            .filter(agents::id.eq(id))
            .first::<Agent>(&mut conn)?;

        let tools = agent_tools::table
            .filter(agent_tools::agent_id.eq(id))
            .load::<AgentTool>(&mut conn)?;

        let skills = agent_skills::table
            .filter(agent_skills::agent_id.eq(id))
            .load::<AgentSkill>(&mut conn)?;

        let content_stores = content_store_configs::table
            .filter(content_store_configs::agent_id.eq(id))
            .load::<ContentStoreConfig>(&mut conn)?;

        Ok(AgentDetail {
            id: agent.id,
            name: agent.name,
            defination: agent.defination,
            tools,
            skills,
            content_stores,
            created_at: chrono::DateTime::from_naive_utc_and_offset(agent.created_at, chrono::Utc),
            updated_at: chrono::DateTime::from_naive_utc_and_offset(agent.updated_at, chrono::Utc),
        })
    }

    pub async fn get_agent_tools(&self, id: i64) -> ServiceResult<Vec<MCPTool>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let tools = agent_tools::table
            .left_join(mcp_servers::table.on(agent_tools::server_id.eq(mcp_servers::id.nullable())))
            .filter(agent_tools::agent_id.eq(id))
            .select((
                agent_tools::name,
                agent_tools::description,
                agent_tools::input_schema,
                agent_tools::output_schema,
                agent_tools::server_id,
            ))
            .load::<(String, String, String, String, Option<i64>)>(&mut conn)?;

        let result: Vec<MCPTool> = tools.into_iter().map(|(name, description, input_schema, output_schema, server_id)| MCPTool {
            name,
            description,
            input_schema: serde_json::from_str(&input_schema).unwrap_or(serde_json::json!({})),
            output_schema: serde_json::from_str(&output_schema).unwrap_or(serde_json::json!({})),
            server_id,
        }).collect();

        Ok(result)
    }

    pub async fn get_mcp_server_url(&self, server_id: Option<i64>) -> ServiceResult<Option<String>> {
        match server_id {
            None => Ok(None),
            Some(id) => {
                let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
                
                let server = mcp_servers::table
                    .filter(mcp_servers::id.eq(id))
                    .filter(mcp_servers::enabled.eq(true))
                    .first::<McpServer>(&mut conn)
                    .optional()?;
                
                Ok(server.map(|s| s.url))
            }
        }
    }

    pub async fn get_all_mcp_servers(&self) -> ServiceResult<HashMap<i64, String>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let servers = mcp_servers::table
            .filter(mcp_servers::enabled.eq(true))
            .load::<McpServer>(&mut conn)?;
        
        let map: HashMap<i64, String> = servers.into_iter()
            .map(|s| (s.id, s.url))
            .collect();
        
        Ok(map)
    }

    pub async fn get_agent_system_prompt(&self, id: i64) -> ServiceResult<Option<String>> {
        let detail = self.get_agent_detail(id).await?;
        
        let mut parts: Vec<String> = Vec::new();
        
        if let Some(def) = detail.defination {
            if !def.is_empty() {
                parts.push(def);
            }
        }
        
        for skill in detail.skills {
            if !skill.skill_prompt.is_empty() {
                parts.push(skill.skill_prompt);
            }
        }
        
        if parts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(parts.join("\n\n")))
        }
    }
}
