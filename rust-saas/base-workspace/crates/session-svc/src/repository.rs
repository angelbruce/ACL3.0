use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Session, SessionWithNames, SessionItem, SessionType, CreateSessionRequest, UpdateSessionRequest, AddMessageRequest};
use shared::schema::{sessions, session_items, agents, llm_models};
use std::env;

pub struct SessionRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl SessionRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        SessionRepository { pool }
    }

    pub async fn get_all_sessions(&self) -> ServiceResult<Vec<SessionWithNames>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let sessions: Vec<Session> = sessions::table
            .order(sessions::created_at.desc())
            .load::<Session>(&mut conn)?;
        
        let mut result = Vec::new();
        for session in sessions {
            let agent_name = if let Some(aid) = session.agent_id {
                agents::table.filter(agents::id.eq(aid))
                    .select(agents::name)
                    .first::<String>(&mut conn)
                    .ok()
            } else { None };
            
            let model_name = if let Some(mid) = session.model_id {
                llm_models::table.filter(llm_models::id.eq(mid))
                    .select(llm_models::name)
                    .first::<String>(&mut conn)
                    .ok()
            } else { None };
            
            result.push(SessionWithNames {
                id: session.id,
                user_id: session.user_id,
                description: session.description,
                agent_id: session.agent_id,
                model_id: session.model_id,
                agent_name,
                model_name,
                created_at: session.created_at,
            });
        }
        
        Ok(result)
    }

    pub async fn get_session(&self, id: i64) -> ServiceResult<SessionWithNames> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let session: Session = sessions::table
            .filter(sessions::id.eq(id))
            .first::<Session>(&mut conn)?;
        
        let agent_name = if let Some(aid) = session.agent_id {
            agents::table.filter(agents::id.eq(aid))
                .select(agents::name)
                .first::<String>(&mut conn)
                .ok()
        } else { None };
        
        let model_name = if let Some(mid) = session.model_id {
            llm_models::table.filter(llm_models::id.eq(mid))
                .select(llm_models::name)
                .first::<String>(&mut conn)
                .ok()
        } else { None };
        
        Ok(SessionWithNames {
            id: session.id,
            user_id: session.user_id,
            description: session.description,
            agent_id: session.agent_id,
            model_id: session.model_id,
            agent_name,
            model_name,
            created_at: session.created_at,
        })
    }

    pub async fn create_session(&self, req: CreateSessionRequest) -> ServiceResult<Session> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let session = diesel::insert_into(sessions::table)
            .values((
                sessions::user_id.eq(req.user_id),
                sessions::description.eq(&req.description),
                sessions::agent_id.eq(req.agent_id),
                sessions::model_id.eq(req.model_id),
                sessions::created_at.eq(now),
            ))
            .returning(Session::as_select())
            .get_result(&mut conn)?;
        
        Ok(session)
    }

    pub async fn update_session(&self, id: i64, req: UpdateSessionRequest) -> ServiceResult<Session> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let session = diesel::update(sessions::table.filter(sessions::id.eq(id)))
            .set((
                sessions::agent_id.eq(req.agent_id),
                sessions::model_id.eq(req.model_id),
            ))
            .returning(Session::as_select())
            .get_result(&mut conn)?;
        
        Ok(session)
    }

    pub async fn delete_session(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(session_items::table.filter(session_items::session_id.eq(id))).execute(&mut conn)?;
        diesel::delete(sessions::table.filter(sessions::id.eq(id))).execute(&mut conn)?;
        
        Ok(())
    }

    pub async fn get_session_messages(&self, session_id: i64) -> ServiceResult<Vec<SessionItem>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let items = session_items::table
            .filter(session_items::session_id.eq(session_id))
            .order(session_items::created_at.asc())
            .load::<SessionItem>(&mut conn)?;
        
        Ok(items)
    }

    pub async fn add_message(&self, session_id: i64, req: AddMessageRequest) -> ServiceResult<SessionItem> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let item = diesel::insert_into(session_items::table)
            .values((
                session_items::session_id.eq(session_id),
                session_items::description.eq(&req.description),
                session_items::session_type.eq(req.session_type.to_string()),
                session_items::created_at.eq(now),
            ))
            .returning(SessionItem::as_select())
            .get_result(&mut conn)?;
        
        Ok(item)
    }
}


