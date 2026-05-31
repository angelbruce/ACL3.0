use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{User, Personnel};
use shared::schema::{users, personnel};
use std::env;

pub struct UserRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl UserRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        UserRepository { pool }
    }

    pub async fn get_all_users(&self) -> ServiceResult<Vec<User>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let users = users::table
            .order(users::created_at.desc())
            .load::<User>(&mut conn)?;
        
        Ok(users)
    }

    pub async fn get_user(&self, id: i64) -> ServiceResult<User> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let user = users::table
            .filter(users::id.eq(id))
            .first::<User>(&mut conn)?;
        
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> ServiceResult<Option<User>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let user = users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut conn)
            .optional()?;
        
        Ok(user)
    }

    pub async fn get_personnel_by_user_id(&self, user_id: i64) -> ServiceResult<Option<Personnel>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let personnel = personnel::table
            .filter(personnel::user_id.eq(user_id))
            .first::<Personnel>(&mut conn)
            .optional()?;
        
        Ok(personnel)
    }

    pub async fn update_last_login_date(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        diesel::update(personnel::table.filter(personnel::id.eq(id)))
            .set(personnel::last_login_date.eq(now))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn create_user(&self, email: &str, password_hash: &str) -> ServiceResult<User> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let user = diesel::insert_into(users::table)
            .values((
                users::email.eq(email),
                users::password_hash.eq(password_hash),
                users::created_at.eq(now),
            ))
            .returning(User::as_select())
            .get_result(&mut conn)?;
        
        Ok(user)
    }

    pub async fn create_personnel(&self, user_id: i64, name: &str, email: &str) -> ServiceResult<Personnel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let personnel = diesel::insert_into(personnel::table)
            .values((
                personnel::user_id.eq(user_id),
                personnel::name.eq(name),
                personnel::email.eq(email),
                personnel::created_at.eq(now),
                personnel::updated_at.eq(now),
            ))
            .returning(Personnel::as_select())
            .get_result(&mut conn)?;
        
        Ok(personnel)
    }
}
