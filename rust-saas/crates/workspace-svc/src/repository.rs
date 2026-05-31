use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{KanbanBoard, KanbanItem, KanbanSubscription, CreateKanbanBoardRequest, UpdateKanbanBoardRequest};
use shared::schema::{kanban_boards, kanban_items, kanban_subscriptions};
use std::env;

pub struct WorkspaceRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl WorkspaceRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        WorkspaceRepository { pool }
    }

    pub async fn get_public_kanban_boards(&self) -> ServiceResult<Vec<KanbanBoard>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let boards = kanban_boards::table
            .filter(kanban_boards::is_public.eq(true))
            .order(kanban_boards::created_at.desc())
            .load::<KanbanBoard>(&mut conn)?;
        
        Ok(boards)
    }

    pub async fn get_kanban_board_by_id(&self, board_id: i64) -> ServiceResult<Option<KanbanBoard>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let board = kanban_boards::table
            .filter(kanban_boards::id.eq(board_id))
            .first::<KanbanBoard>(&mut conn)
            .optional()?;
        
        Ok(board)
    }

    pub async fn create_kanban_board(&self, user_id: i64, req: CreateKanbanBoardRequest) -> ServiceResult<KanbanBoard> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let board = diesel::insert_into(kanban_boards::table)
            .values((
                kanban_boards::name.eq(req.name),
                kanban_boards::description.eq(req.description),
                kanban_boards::is_public.eq(req.is_public.unwrap_or(true)),
                kanban_boards::created_by.eq(user_id),
                kanban_boards::created_at.eq(now),
                kanban_boards::updated_at.eq(now),
            ))
            .returning(KanbanBoard::as_select())
            .get_result(&mut conn)?;
        
        Ok(board)
    }

    pub async fn update_kanban_board(&self, board_id: i64, user_id: i64, req: UpdateKanbanBoardRequest) -> ServiceResult<KanbanBoard> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();

        if req.name.is_none() || req.is_public.is_none() {
            return Err(ServiceError::BadRequest("At least one field must be provided for update".to_string()));
        }
        
        let mut update_query = diesel::update(
            kanban_boards::table
                .filter(kanban_boards::id.eq(board_id))
                .filter(kanban_boards::created_by.eq(user_id))
        ).set((
            kanban_boards::name.eq(req.name.unwrap_or("".to_string())),
            kanban_boards::description.eq(req.description),
            kanban_boards::is_public.eq(req.is_public.unwrap_or(false)),
            kanban_boards::updated_at.eq(now)
        ));
        
        let board = update_query
            .returning(KanbanBoard::as_select())
            .get_result(&mut conn)?;
        
        Ok(board)
    }

    pub async fn delete_kanban_board(&self, board_id: i64, user_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let result = diesel::delete(
            kanban_boards::table
                .filter(kanban_boards::id.eq(board_id))
                .filter(kanban_boards::created_by.eq(user_id))
        )
        .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::NotFound);
        }
        
        Ok(())
    }

    pub async fn get_kanban_items(&self, board_id: i64) -> ServiceResult<Vec<KanbanItem>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let items = kanban_items::table
            .filter(kanban_items::board_id.eq(board_id))
            .order(kanban_items::shared_at.desc())
            .load::<KanbanItem>(&mut conn)?;
        
        Ok(items)
    }

    pub async fn add_kanban_item(&self, board_id: i64, user_id: i64, file_path: String, file_name: String) -> ServiceResult<KanbanItem> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let item = diesel::insert_into(kanban_items::table)
            .values((
                kanban_items::board_id.eq(board_id),
                kanban_items::user_id.eq(user_id),
                kanban_items::file_path.eq(file_path),
                kanban_items::file_name.eq(file_name),
                kanban_items::shared_at.eq(now),
            ))
            .returning(KanbanItem::as_select())
            .get_result(&mut conn)?;
        
        Ok(item)
    }

    pub async fn remove_kanban_item(&self, item_id: i64, user_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let result = diesel::delete(
            kanban_items::table
                .filter(kanban_items::id.eq(item_id))
                .filter(kanban_items::user_id.eq(user_id))
        )
        .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::NotFound);
        }
        
        Ok(())
    }

    pub async fn subscribe_board(&self, board_id: i64, user_id: i64) -> ServiceResult<KanbanSubscription> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let existing = kanban_subscriptions::table
            .filter(kanban_subscriptions::board_id.eq(board_id))
            .filter(kanban_subscriptions::user_id.eq(user_id))
            .first::<KanbanSubscription>(&mut conn)
            .optional()?;
        
        if let Some(sub) = existing {
            return Ok(sub);
        }
        
        let subscription = diesel::insert_into(kanban_subscriptions::table)
            .values((
                kanban_subscriptions::board_id.eq(board_id),
                kanban_subscriptions::user_id.eq(user_id),
                kanban_subscriptions::subscribed_at.eq(now),
            ))
            .returning(KanbanSubscription::as_select())
            .get_result(&mut conn)?;
        
        Ok(subscription)
    }

    pub async fn unsubscribe_board(&self, board_id: i64, user_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let result = diesel::delete(
            kanban_subscriptions::table
                .filter(kanban_subscriptions::board_id.eq(board_id))
                .filter(kanban_subscriptions::user_id.eq(user_id))
        )
        .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::NotFound);
        }
        
        Ok(())
    }

    pub async fn get_user_subscriptions(&self, user_id: i64) -> ServiceResult<Vec<KanbanSubscription>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let subscriptions = kanban_subscriptions::table
            .filter(kanban_subscriptions::user_id.eq(user_id))
            .order(kanban_subscriptions::subscribed_at.desc())
            .load::<KanbanSubscription>(&mut conn)?;
        
        Ok(subscriptions)
    }

    pub async fn get_subscriber_count(&self, board_id: i64) -> ServiceResult<i64> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let count = kanban_subscriptions::table
            .filter(kanban_subscriptions::board_id.eq(board_id))
            .count()
            .get_result::<i64>(&mut conn)?;
        
        Ok(count)
    }
}
