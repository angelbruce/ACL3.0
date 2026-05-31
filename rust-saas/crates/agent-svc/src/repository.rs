use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Agent, AgentDetail, AgentSkill, AgentTool, ContentStoreConfig, CreateAgentRequest};
use shared::schema::{
    agent_skills, agent_tools, agents, content_store_configs,
};
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

    pub async fn get_all_agents(&self) -> ServiceResult<Vec<Agent>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let agents = agents::table
            .order(agents::created_at.desc())
            .load::<Agent>(&mut conn)?;
        
        Ok(agents)
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

    pub async fn create_agent(&self, req: CreateAgentRequest) -> ServiceResult<Agent> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let agent = diesel::insert_into(agents::table)
            .values((
                agents::name.eq(&req.name),
                agents::defination.eq(&req.defination),
                agents::created_at.eq(now),
                agents::updated_at.eq(now),
            ))
            .returning(Agent::as_select())
            .get_result(&mut conn)?;

        if let Some(tools) = &req.tools {
            for tool in tools {
                diesel::insert_into(agent_tools::table)
                        .values((
                            agent_tools::agent_id.eq(agent.id),
                            agent_tools::name.eq(&tool.name),
                            agent_tools::description.eq(&tool.description),
                            agent_tools::input_schema.eq(&tool.input_schema),
                            agent_tools::output_schema.eq(&tool.output_schema),
                            agent_tools::server_id.eq(&tool.server_id),
                        ))
                        .execute(&mut conn)?;
            }
        }

        if let Some(skills) = &req.skills {
            for skill in skills {
                diesel::insert_into(agent_skills::table)
                    .values((
                        agent_skills::agent_id.eq(agent.id),
                        agent_skills::skill_prompt.eq(&skill.skill_prompt),
                    ))
                    .execute(&mut conn)?;
            }
        }

        Ok(agent)
    }

    pub async fn update_agent(&self, id: i64, req: CreateAgentRequest) -> ServiceResult<Agent> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let agent = diesel::update(agents::table)
            .filter(agents::id.eq(id))
            .set((
                agents::name.eq(&req.name),
                agents::defination.eq(&req.defination),
                agents::updated_at.eq(now),
            ))
            .returning(Agent::as_select())
            .get_result(&mut conn)?;

        diesel::delete(agent_tools::table.filter(agent_tools::agent_id.eq(id))).execute(&mut conn)?;
        if let Some(tools) = &req.tools {
            for tool in tools {
                diesel::insert_into(agent_tools::table)
                        .values((
                            agent_tools::agent_id.eq(id),
                            agent_tools::name.eq(&tool.name),
                            agent_tools::description.eq(&tool.description),
                            agent_tools::input_schema.eq(&tool.input_schema),
                            agent_tools::output_schema.eq(&tool.output_schema),
                            agent_tools::server_id.eq(&tool.server_id),
                        ))
                        .execute(&mut conn)?;
            }
        }

        diesel::delete(agent_skills::table.filter(agent_skills::agent_id.eq(id))).execute(&mut conn)?;
        if let Some(skills) = &req.skills {
            for skill in skills {
                diesel::insert_into(agent_skills::table)
                    .values((
                        agent_skills::agent_id.eq(id),
                        agent_skills::skill_prompt.eq(&skill.skill_prompt),
                    ))
                    .execute(&mut conn)?;
            }
        }

        Ok(agent)
    }

    pub async fn delete_agent(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(agent_tools::table.filter(agent_tools::agent_id.eq(id))).execute(&mut conn)?;
        diesel::delete(agent_skills::table.filter(agent_skills::agent_id.eq(id))).execute(&mut conn)?;
        diesel::delete(content_store_configs::table.filter(content_store_configs::agent_id.eq(id))).execute(&mut conn)?;
        diesel::delete(agents::table.filter(agents::id.eq(id))).execute(&mut conn)?;
        
        Ok(())
    }
}
