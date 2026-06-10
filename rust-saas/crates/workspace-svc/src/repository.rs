use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{
    KanbanBoard, KanbanItem, KanbanSubscription, CreateKanbanBoardRequest, UpdateKanbanBoardRequest,
    Project, ProjectFile, ProjectMessage, ProjectSummary, ProjectWithNames,
    CreateProjectRequest, UpdateProjectRequest,
    CreateProjectFileRequest, UpdateProjectFileRequest, AddProjectMessageRequest,
    CreateOrUpdateProjectSummaryRequest, ProjectContainerConfig
};
use shared::schema::{kanban_boards, kanban_items, kanban_subscriptions, projects,
     project_files, project_messages, project_container_configs,
     project_summaries, agents, llm_models
    };
use std::env;
use std::ops::Index;

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

    pub async fn get_projects_by_user(&self, user_id: i64) -> ServiceResult<Vec<ProjectWithNames>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        println!("1");
        let project_list: Vec<Project> = projects::table
            .filter(projects::user_id.eq(user_id))
            .order(projects::last_accessed_at.desc())
            .load(&mut conn)?;
        
        let mut result: Vec<ProjectWithNames> = Vec::new();
        
        println!("2");
        for p in project_list {
            let model_name = if let Some(model_id) = p.model_id {
                llm_models::table
                    .filter(llm_models::id.eq(model_id))
                    .first::<shared::models::LlmModel>(&mut conn)
                    .optional()?
                    .map(|m| m.name)
            } else {
                None
            };
            
        println!("3");
            let agent_name = if let Some(agent_id) = p.agent_id {
                agents::table
                    .filter(agents::id.eq(agent_id))
                    .first::<shared::models::Agent>(&mut conn)
                    .optional()?
                    .map(|a| a.name)
            } else {
                None
            };
            
        println!("4");
            result.push(ProjectWithNames {
                id: p.id,
                user_id: p.user_id,
                name: p.name,
                purpose: p.purpose,
                description: p.description,
                model_id: p.model_id,
                agent_id: p.agent_id,
                model_name,
                agent_name,
                last_accessed_at: p.last_accessed_at,
                created_at: p.created_at,
                updated_at: p.updated_at,
            });
        }
        
        Ok(result)
    }

    pub async fn get_project_by_id(&self, project_id: i64) -> ServiceResult<Option<ProjectWithNames>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let project = projects::table
            .filter(projects::id.eq(project_id))
            .first::<Project>(&mut conn)
            .optional()?;
        
        if let Some(p) = project {
            let model_name = if let Some(model_id) = p.model_id {
                llm_models::table
                    .filter(llm_models::id.eq(model_id))
                    .first::<shared::models::LlmModel>(&mut conn)
                    .optional()?
                    .map(|m| m.name)
            } else {
                None
            };
            
            let agent_name = if let Some(agent_id) = p.agent_id {
                agents::table
                    .filter(agents::id.eq(agent_id))
                    .first::<shared::models::Agent>(&mut conn)
                    .optional()?
                    .map(|a| a.name)
            } else {
                None
            };
            
            Ok(Some(ProjectWithNames {
                id: p.id,
                user_id: p.user_id,
                name: p.name,
                purpose: p.purpose,
                description: p.description,
                model_id: p.model_id,
                agent_id: p.agent_id,
                model_name,
                agent_name,
                last_accessed_at: p.last_accessed_at,
                created_at: p.created_at,
                updated_at: p.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_project(&self, user_id: i64, req: CreateProjectRequest) -> ServiceResult<Project> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let project = diesel::insert_into(projects::table)
            .values((
                projects::user_id.eq(user_id),
                projects::name.eq(req.name),
                projects::purpose.eq(req.purpose),
                projects::description.eq(req.description),
                projects::model_id.eq(req.model_id),
                projects::agent_id.eq(req.agent_id),
                projects::last_accessed_at.eq(now),
                projects::created_at.eq(now),
                projects::updated_at.eq(now),
            ))
            .returning(Project::as_select())
            .get_result(&mut conn)?;
        
        let workspace_root = env::var("WORKSPACE_ROOT").unwrap_or_else(|_| "/workspace_storage".to_string());
        let user_dir = std::path::Path::new(&workspace_root).join(user_id.to_string());
        let project_dir = user_dir.join(project.id.to_string());
        
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            tracing::warn!("Failed to create project directory: {}", e);
        }
        
        Ok(project)
    }

    pub async fn update_project(&self, project_id: i64, user_id: i64, req: UpdateProjectRequest) -> ServiceResult<Project> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let project = diesel::update(
            projects::table
                .filter(projects::id.eq(project_id))
                .filter(projects::user_id.eq(user_id))
        )
        .set((
            projects::name.eq(req.name),
            projects::description.eq(req.description),
            projects::model_id.eq(req.model_id),
            projects::agent_id.eq(req.agent_id),
            projects::updated_at.eq(now),
        ))
        .returning(Project::as_select())
        .get_result(&mut conn)?;
        
        Ok(project)
    }

    pub async fn delete_project(&self, project_id: i64, user_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(
            project_files::table
                .filter(project_files::project_id.eq(project_id))
        )
        .execute(&mut conn)?;
        
        diesel::delete(
            project_messages::table
                .filter(project_messages::project_id.eq(project_id))
        )
        .execute(&mut conn)?;
        
        diesel::delete(
            project_summaries::table
                .filter(project_summaries::project_id.eq(project_id))
        )
        .execute(&mut conn)?;
        
        let result = diesel::delete(
            projects::table
                .filter(projects::id.eq(project_id))
                .filter(projects::user_id.eq(user_id))
        )
        .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::NotFound);
        }
        
        Ok(())
    }

    pub async fn get_project_files(&self, project_id: i64, user_id: i64) -> ServiceResult<Vec<ProjectFile>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let _project = projects::table
            .filter(projects::id.eq(project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)?;
        
        let files = project_files::table
            .filter(project_files::project_id.eq(project_id))
            .order(project_files::created_at.asc())
            .load(&mut conn)?;
        
        Ok(files)
    }

    pub async fn create_project_file(&self, project_id: i64, user_id: i64, req: CreateProjectFileRequest) -> ServiceResult<ProjectFile> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        diesel::update(
            projects::table
                .filter(projects::id.eq(project_id))
                .filter(projects::user_id.eq(user_id))
        )
        .set(projects::updated_at.eq(now))
        .execute(&mut conn)?;
        
        let mut file_name = req.name.clone();
        let mut file_dir = "".to_string();
        if let Some(idx) = file_name.rfind('/') {
            file_dir = file_name[0..idx].to_string();
            file_name = file_name[idx + 1..].to_string();
        }

        let file = diesel::insert_into(project_files::table)
            .values((
                project_files::project_id.eq(project_id),
                project_files::name.eq(file_name),
                project_files::content.eq(req.content),
                project_files::directory.eq(file_dir),
                project_files::state.eq(0),
                project_files::created_at.eq(now),
                project_files::updated_at.eq(now),
            ))
            .returning(ProjectFile::as_select())
            .get_result(&mut conn)?;
        
        Ok(file)
    }


    pub async fn get_project_file_by_id(&self, file_id: i64,user_id: i64) -> ServiceResult<ProjectFile> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let file = project_files::table
            .filter(project_files::id.eq(file_id))
            .first::<ProjectFile>(&mut conn)
            .optional()?;
        
        if file.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        Ok(file.unwrap())
    }

    pub async fn update_project_file_status(&self, file_id: i64, user_id: i64, status: i32) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        diesel::update(project_files::table.filter(project_files::id.eq(file_id)))
            .set((
                project_files::state.eq(status),
                project_files::updated_at.eq(now),
            ))
            .execute(&mut conn)?;
        
        Ok(())
    }

    pub async fn update_project_file(&self, file_id: i64, user_id: i64, req: UpdateProjectFileRequest) -> ServiceResult<ProjectFile> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let file = project_files::table
            .filter(project_files::id.eq(file_id))
            .first::<ProjectFile>(&mut conn)
            .optional()?;
        
        if file.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        let project = projects::table
            .filter(projects::id.eq(file.as_ref().unwrap().project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)
            .optional()?;
        
        if project.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        let now = Utc::now().naive_utc();
        
        let updated_file = diesel::update(project_files::table.filter(project_files::id.eq(file_id)))
            .set((
                project_files::content.eq(req.content),
                project_files::updated_at.eq(now),
            ))
            .returning(ProjectFile::as_select())
            .get_result(&mut conn)?;
        
        Ok(updated_file)
    }

    pub async fn delete_project_file(&self, file_id: i64, user_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let file = project_files::table
            .filter(project_files::id.eq(file_id))
            .first::<ProjectFile>(&mut conn)
            .optional()?;
        
        if file.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        let project = projects::table
            .filter(projects::id.eq(file.as_ref().unwrap().project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)
            .optional()?;
        
        if project.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        diesel::delete(
            project_summaries::table
                .filter(project_summaries::file_name.eq(file.as_ref().unwrap().name.clone()))
                .filter(project_summaries::project_id.eq(file.as_ref().unwrap().project_id))
        )
        .execute(&mut conn)?;
        
        let result = diesel::delete(project_files::table.filter(project_files::id.eq(file_id)))
            .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::NotFound);
        }
        
        Ok(())
    }


    pub async fn get_project_container_config(&self, project_id: i64) -> ServiceResult<Vec<ProjectContainerConfig>> {
        let mut conn: r2d2::PooledConnection<ConnectionManager<PgConnection>> = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let config = project_container_configs::table
            .filter(project_container_configs::project_id.eq(project_id))            
            .load::<ProjectContainerConfig>(&mut conn)?;
        
        if config.is_empty() {
            return Ok(Vec::new());
        }
        
        Ok(config)
    }

    pub async fn save_project_container_config(&self,
        creator_id: i64, 
        project_id: i64,
        datas: Vec<ProjectContainerConfig>) -> ServiceResult<Vec<ProjectContainerConfig>> {

        let affected = self.delete_project_container_config_by_project_id(project_id).await;
        match affected {
            Ok(_) => {},
            Err(e) =>{ }
        };

        let now = Utc::now().naive_utc();
        let mut configs = Vec::new();
        for project_container_config in datas {
            let config = self.insert_project_container_config(creator_id, project_id, project_container_config).await?;
            configs.push(config.clone());        
        };
       
        Ok(configs)
    }

    pub async fn insert_project_container_config(&self,
        creator_id: i64, 
        project_id: i64,
        project_container_config: ProjectContainerConfig) -> ServiceResult<ProjectContainerConfig> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();
        
       let config = diesel::insert_into(project_container_configs::table)
                .values((
                project_container_configs::project_id.eq(project_id),
                project_container_configs::project_dir.eq(project_container_config.project_dir),
                project_container_configs::published_ports.eq(project_container_config.published_ports),
                project_container_configs::volumes.eq(project_container_config.volumes),
                project_container_configs::environment.eq(project_container_config.environment),
                project_container_configs::command.eq(project_container_config.command),
                project_container_configs::working_dir.eq(project_container_config.working_dir),
                project_container_configs::tags.eq(project_container_config.tags),
                project_container_configs::container_name.eq(project_container_config.container_name),
                project_container_configs::cpu_usage.eq(project_container_config.cpu_usage),
                project_container_configs::memory_usage.eq(project_container_config.memory_usage),
                project_container_configs::image_name.eq(project_container_config.image_name),
                project_container_configs::creator_id.eq(creator_id),
                project_container_configs::created_at.eq(now),
                project_container_configs::updated_at.eq(now),
            ))
            .returning(ProjectContainerConfig::as_select())
            .get_result(&mut conn)?;

        
        Ok(config)
    }
 
    pub async fn delete_project_container_config_by_project_id(&self, project_id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let result = diesel::delete(project_container_configs::table
            .filter(project_container_configs::project_id.eq(project_id)))
        .execute(&mut conn)?;
        
        if result == 0 {
            return Err(ServiceError::DatabaseError("Project container config not found".to_string()));
        }
        
        Ok(())
    }

    pub async fn get_project_messages(&self, project_id: i64, user_id: i64) -> ServiceResult<Vec<ProjectMessage>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let _project = projects::table
            .filter(projects::id.eq(project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)?;
        
        let messages = project_messages::table
            .filter(project_messages::project_id.eq(project_id))
            .order(project_messages::created_at.asc())
            .load(&mut conn)?;
        
        Ok(messages)
    }

    pub async fn add_project_message(&self, project_id: i64, user_id: i64, req: AddProjectMessageRequest) -> ServiceResult<ProjectMessage> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        diesel::update(
            projects::table
                .filter(projects::id.eq(project_id))
                .filter(projects::user_id.eq(user_id))
        )
        .set(projects::last_accessed_at.eq(now))
        .execute(&mut conn)?;
        
        let message = diesel::insert_into(project_messages::table)
            .values((
                project_messages::project_id.eq(project_id),
                project_messages::role.eq(req.role),
                project_messages::content.eq(req.content),
                project_messages::created_at.eq(now),
            ))
            .returning(ProjectMessage::as_select())
            .get_result(&mut conn)?;
        
        Ok(message)
    }

    pub async fn get_project_summaries(&self, project_id: i64, user_id: i64) -> ServiceResult<Vec<ProjectSummary>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let _project = projects::table
            .filter(projects::id.eq(project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)?;
        
        let summaries = project_summaries::table
            .filter(project_summaries::project_id.eq(project_id))
            .filter(project_summaries::user_id.eq(user_id))
            .order(project_summaries::updated_at.desc())
            .load(&mut conn)?;
        
        Ok(summaries)
    }

    pub async fn create_or_update_project_summary(
        &self,
        project_id: i64,
        user_id: i64,
        req: CreateOrUpdateProjectSummaryRequest
    ) -> ServiceResult<ProjectSummary> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let project = projects::table
            .filter(projects::id.eq(project_id))
            .filter(projects::user_id.eq(user_id))
            .first::<Project>(&mut conn)
            .optional()?;
        
        if project.is_none() {
            return Err(ServiceError::NotFound);
        }
        
        let now = Utc::now().naive_utc();
        
        let existing = project_summaries::table
            .filter(project_summaries::project_id.eq(project_id))
            .filter(project_summaries::file_name.eq(&req.file_name))
            .filter(project_summaries::user_id.eq(user_id))
            .first::<ProjectSummary>(&mut conn)
            .optional()?;
        
        if let Some(existing_summary) = existing {
            let updated = diesel::update(project_summaries::table.filter(project_summaries::id.eq(existing_summary.id)))
                .set((
                    project_summaries::summary.eq(&req.summary),
                    project_summaries::updated_at.eq(now),
                ))
                .returning(ProjectSummary::as_select())
                .get_result(&mut conn)?;
            
            return Ok(updated);
        }
        
        let summary = diesel::insert_into(project_summaries::table)
            .values((
                project_summaries::user_id.eq(user_id),
                project_summaries::project_id.eq(project_id),
                project_summaries::file_name.eq(&req.file_name),
                project_summaries::summary.eq(&req.summary),
                project_summaries::created_at.eq(now),
                project_summaries::updated_at.eq(now),
            ))
            .returning(ProjectSummary::as_select())
            .get_result(&mut conn)?;
        
        Ok(summary)
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
        
        let board = diesel::update(
            kanban_boards::table
                .filter(kanban_boards::id.eq(board_id))
                .filter(kanban_boards::created_by.eq(user_id))
        ).set((
            kanban_boards::name.eq(req.name.unwrap_or("".to_string())),
            kanban_boards::description.eq(req.description),
            kanban_boards::is_public.eq(req.is_public.unwrap_or(false)),
            kanban_boards::updated_at.eq(now)
        ))
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
