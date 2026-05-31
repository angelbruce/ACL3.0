use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{McpServer, CreateMcpServerRequest};
use shared::schema::mcp_servers;
use std::env;

pub struct McpServerRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl McpServerRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        McpServerRepository { pool }
    }

    pub async fn get_all_servers(&self) -> ServiceResult<Vec<McpServer>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let servers = mcp_servers::table
            .order(mcp_servers::created_at.desc())
            .load::<McpServer>(&mut conn)?;
        
        Ok(servers)
    }

    pub async fn get_enabled_servers(&self) -> ServiceResult<Vec<McpServer>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let servers = mcp_servers::table
            .filter(mcp_servers::enabled.eq(true))
            .order(mcp_servers::created_at.desc())
            .load::<McpServer>(&mut conn)?;
        
        Ok(servers)
    }

    pub async fn get_server(&self, id: i64) -> ServiceResult<McpServer> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let server = mcp_servers::table
            .filter(mcp_servers::id.eq(id))
            .first::<McpServer>(&mut conn)?;
        
        Ok(server)
    }

    pub async fn create_server(&self, req: CreateMcpServerRequest) -> ServiceResult<McpServer> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let server = diesel::insert_into(mcp_servers::table)
            .values((
                mcp_servers::name.eq(&req.name),
                mcp_servers::description.eq(&req.description),
                mcp_servers::server_type.eq(&req.server_type),
                mcp_servers::url.eq(&req.url),
                mcp_servers::headers.eq(&req.headers),
                mcp_servers::enabled.eq(req.enabled.unwrap_or(true)),
                mcp_servers::stateless.eq(req.stateless.unwrap_or(false)),
                mcp_servers::created_at.eq(now),
                mcp_servers::updated_at.eq(now),
            ))
            .returning(McpServer::as_select())
            .get_result(&mut conn)?;
        
        Ok(server)
    }

    pub async fn update_server(&self, id: i64, req: CreateMcpServerRequest) -> ServiceResult<McpServer> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let server = diesel::update(mcp_servers::table)
            .filter(mcp_servers::id.eq(id))
            .set((
                mcp_servers::name.eq(&req.name),
                mcp_servers::description.eq(&req.description),
                mcp_servers::server_type.eq(&req.server_type),
                mcp_servers::url.eq(&req.url),
                mcp_servers::headers.eq(&req.headers),
                mcp_servers::enabled.eq(req.enabled.unwrap_or(true)),
                mcp_servers::stateless.eq(req.stateless.unwrap_or(false)),
                mcp_servers::updated_at.eq(now),
            ))
            .returning(McpServer::as_select())
            .get_result(&mut conn)?;
        
        Ok(server)
    }

    pub async fn delete_server(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(mcp_servers::table.filter(mcp_servers::id.eq(id))).execute(&mut conn)?;
        
        Ok(())
    }

    pub async fn set_enabled(&self, id: i64, enabled: bool) -> ServiceResult<McpServer> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let server = diesel::update(mcp_servers::table)
            .filter(mcp_servers::id.eq(id))
            .set((
                mcp_servers::enabled.eq(enabled),
                mcp_servers::updated_at.eq(now),
            ))
            .returning(McpServer::as_select())
            .get_result(&mut conn)?;
        
        Ok(server)
    }
}
