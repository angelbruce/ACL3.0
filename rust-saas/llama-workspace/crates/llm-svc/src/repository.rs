use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{LlmModel, CreateLlmModelRequest};
use shared::schema::llm_models;
use std::env;

pub struct ModelRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl ModelRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        ModelRepository { pool }
    }

    pub async fn get_all_models(&self) -> ServiceResult<Vec<LlmModel>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let models = llm_models::table
            .order(llm_models::name.asc())
            .load::<LlmModel>(&mut conn)?;
        
        Ok(models)
    }

    pub async fn get_model(&self, id: i64) -> ServiceResult<LlmModel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let model = llm_models::table
            .filter(llm_models::id.eq(id))
            .first::<LlmModel>(&mut conn)?;
        
        Ok(model)
    }

    pub async fn create_model(&self, req: CreateLlmModelRequest) -> ServiceResult<LlmModel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        if req.is_default {
            diesel::update(llm_models::table)
                .set(llm_models::is_default.eq(false))
                .execute(&mut conn)?;
        }
        
        let new_model = diesel::insert_into(llm_models::table)
            .values((
                llm_models::name.eq(&req.name),
                llm_models::access_url.eq(&req.access_url),
                llm_models::api_key.eq(&req.api_key),
                llm_models::is_default.eq(req.is_default),
            ))
            .returning(LlmModel::as_select())
            .get_result(&mut conn)?;
        
        Ok(new_model)
    }

    pub async fn update_model(&self, id: i64, req: CreateLlmModelRequest) -> ServiceResult<LlmModel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        if req.is_default {
            diesel::update(llm_models::table)
                .set(llm_models::is_default.eq(false))
                .execute(&mut conn)?;
        }
        
        let updated_model = diesel::update(llm_models::table)
            .filter(llm_models::id.eq(id))
            .set((
                llm_models::name.eq(&req.name),
                llm_models::access_url.eq(&req.access_url),
                llm_models::api_key.eq(&req.api_key),
                llm_models::is_default.eq(req.is_default),
            ))
            .returning(LlmModel::as_select())
            .get_result(&mut conn)?;
        
        Ok(updated_model)
    }

    pub async fn delete_model(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(llm_models::table.filter(llm_models::id.eq(id))).execute(&mut conn)?;
        
        Ok(())
    }
}
