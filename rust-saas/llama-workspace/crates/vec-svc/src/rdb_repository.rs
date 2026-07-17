//! Repository 数据库操作
//! RAG配置等数据的 CRUD 操作

use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, TextExpressionMethods, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::sync::Arc;

use crate::schema::*;
use crate::model::*;

/// 
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Repository 错误类型
#[derive(Debug)]
pub enum RepositoryError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    InsertError(String),
    UpdateError(String),
    DeleteError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::ConnectionError(s) => write!(f, "Connection error: {}", s),
            RepositoryError::QueryError(s) => write!(f, "Query error: {}", s),
            RepositoryError::NotFound(s) => write!(f, "Not found: {}", s),
            RepositoryError::InsertError(s) => write!(f, "Insert error: {}", s),
            RepositoryError::UpdateError(s) => write!(f, "Update error: {}", s),
            RepositoryError::DeleteError(s) => write!(f, "Delete error: {}", s),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<diesel::result::Error> for RepositoryError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => RepositoryError::NotFound("Record not found".to_string()),
            _ => RepositoryError::QueryError(e.to_string()),
        }
    }
}

impl From<diesel::r2d2::Error> for RepositoryError {
    fn from(e: diesel::r2d2::Error) -> Self {
        RepositoryError::ConnectionError(e.to_string())
    }
}

// ============ 文档 Repository ============

///  Repository
#[derive(Clone)]
pub struct DocumentRepository {
    pool: Arc<DbPool>,
}

impl DocumentRepository {
    ///  DocumentRepository
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }
    
    /// 
    pub fn create(&self, doc: &NewDocument) -> Result<Document, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::insert_into(documents::table)
            .values(doc)
            .returning(Document::as_returning())
            .get_result(conn)
            .map_err(RepositoryError::from)
    }
    
    /// ID 查询文档
    pub fn get_by_id(&self, id: i64) -> Result<Document, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        documents::table
            .filter(documents::id.eq(id))
            .first::<Document>(conn)
            .map_err(RepositoryError::from)
    }
    
    /// ID 查询文档列表（分页）
    pub fn list_by_project(&self, project_id: Option<i64>, page: usize, page_size: usize) -> Result<Vec<Document>, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let offset = (page.saturating_sub(1)) * page_size;
        
        let query = documents::table
            .order(documents::created_at.desc())
            .offset(offset as i64)
            .limit(page_size as i64);
        
        let results = if let Some(pid) = project_id {
            query.filter(documents::project_id.eq(pid))
                .load::<Document>(conn)?
        } else {
            query.load::<Document>(conn)?
        };
        
        Ok(results)
    }
    
    /// 
    pub fn count(&self, project_id: Option<i64>) -> Result<usize, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let count = if let Some(pid) = project_id {
            documents::table
                .filter(documents::project_id.eq(pid))
                .count()
                .get_result::<i64>(conn)? as usize
        } else {
            documents::table
                .count()
                .get_result::<i64>(conn)? as usize
        };
        
        Ok(count)
    }
    
    /// 
    pub fn update(&self, id: i64, update: &DocumentUpdate) -> Result<Document, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::update(documents::table.filter(documents::id.eq(id)))
            .set(update)
            .returning(Document::as_returning())
            .get_result(conn)
            .map_err(RepositoryError::from)
    }
    
    /// 
    pub fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(documents::table.filter(documents::id.eq(id)))
            .execute(conn)?;
        
        Ok(())
    }
    
    /// chunk_count
    pub fn mark_indexed(&self, id: i64, chunk_count: i32) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let now = chrono::Utc::now().naive_utc();
        
        diesel::update(documents::table.filter(documents::id.eq(id)))
            .set((
                documents::chunk_count.eq(chunk_count),
                documents::indexed_at.eq(now),
                documents::updated_at.eq(now),
            ))
            .execute(conn)?;
        
        Ok(())
    }
}

// ============ 文档分块 Repository ============

///  Repository
#[derive(Clone)]
pub struct ChunkRepository {
    pool: Arc<DbPool>,
}

impl ChunkRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }
    
    /// 
    pub fn batch_insert(&self, chunks: &[NewDocumentChunk]) -> Result<Vec<DocumentChunk>, RepositoryError> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }
        
        let conn = &mut self.get_conn()?;
        
        diesel::insert_into(document_chunks::table)
            .values(chunks)
            .returning(DocumentChunk::as_returning())
            .get_results(conn)
            .map_err(RepositoryError::from)
    }
    
    pub fn get_by_document_id(&self, document_id: i64) -> Result<Vec<DocumentChunk>, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        document_chunks::table
            .filter(document_chunks::document_id.eq(document_id))
            .order(document_chunks::chunk_index.asc())
            .load::<DocumentChunk>(conn)
            .map_err(RepositoryError::from)
    }
    
    pub fn count_by_document(&self, document_id: i64) -> Result<usize, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let count = document_chunks::table
            .filter(document_chunks::document_id.eq(document_id))
            .count()
            .get_result::<i64>(conn)? as usize;
        
        Ok(count)
    }
    
    pub fn delete_by_document_id(&self, document_id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(document_chunks::table.filter(document_chunks::document_id.eq(document_id)))
            .execute(conn)?;
        
        Ok(())
    }
    
    pub fn update_embedding_status(&self, id: i64, status: &str) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::update(document_chunks::table.filter(document_chunks::id.eq(id)))
            .set(document_chunks::embedding_status.eq(status))
            .execute(conn)?;
        
        Ok(())
    }
    
    pub fn get_by_ids_with_document(&self, ids: &[i64]) -> Result<Vec<(DocumentChunk, Document)>, RepositoryError> {
        use crate::schema::documents::dsl as doc_dsl;
        use crate::schema::document_chunks::dsl as chunk_dsl;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let conn = &mut self.get_conn()?;

        chunk_dsl::document_chunks
            .inner_join(doc_dsl::documents.on(chunk_dsl::document_id.eq(doc_dsl::id)))
            .filter(chunk_dsl::id.eq_any(ids))
            .load::<(DocumentChunk, Document)>(conn)
            .map_err(RepositoryError::from)
    }

    /// 关键词搜索（LIKE 查询 chunk_text）
    pub fn search_by_keyword(&self, pattern: &str, project_id: Option<i64>, limit: usize) -> Result<Vec<(DocumentChunk, Document)>, RepositoryError> {
        use crate::schema::documents::dsl as doc_dsl;
        use crate::schema::document_chunks::dsl as chunk_dsl;

        let conn = &mut self.get_conn()?;

        let mut query = chunk_dsl::document_chunks
            .inner_join(doc_dsl::documents.on(chunk_dsl::document_id.eq(doc_dsl::id)))
            .filter(chunk_dsl::chunk_text.ilike(pattern))
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(doc_dsl::project_id.eq(pid));
        }

        let results = query
            .order(chunk_dsl::chunk_index.asc())
            .limit(limit as i64)
            .load::<(DocumentChunk, Document)>(conn)?;

        Ok(results)
    }
}

// ============ RAG 配置 Repository ============

/// RAG 配置 Repository
#[derive(Clone)]
pub struct RagConfigRepository {
    pool: Arc<DbPool>,
}

impl RagConfigRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }
    
    /// RAG 配置（不存在则返回默认值）
    pub fn get_by_project_id(&self, project_id: i64) -> Result<ProjectRagConfig, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let config = project_rag_configs::table
            .filter(project_rag_configs::project_id.eq(project_id))
            .first::<ProjectRagConfig>(conn);
        
        match config {
            Ok(c) => Ok(c),
            Err(diesel::result::Error::NotFound) => {
                // 
                Ok(ProjectRagConfig {
                    id: 0,
                    project_id: Some(project_id),
                    chunk_size: 512,
                    chunk_overlap: 50,
                    chunk_strategy: Some("semantic".to_string()),
                    min_chunk_size: 100,
                    top_k: 5,
                    min_score: 0.3,
                    rerank: false,
                    rerank_top_k: 3,
                    search_type: Some("similarity".to_string()),
                    temperature: 0.7,
                    max_tokens: 2048,
                    context_window: 4096,
                    batch_size: 32,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                })
            },
            Err(e) => Err(RepositoryError::from(e)),
        }
    }
    
    ///  RAG 配置
    pub fn upsert(&self, project_id: i64, config: &NewProjectRagConfig) -> Result<ProjectRagConfig, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        // 
        let existing = project_rag_configs::table
            .filter(project_rag_configs::project_id.eq(project_id))
            .first::<ProjectRagConfig>(conn);
        
        match existing {
            Ok(_) => {
                // 
                diesel::update(project_rag_configs::table.filter(project_rag_configs::project_id.eq(project_id)))
                    .set((
                        project_rag_configs::chunk_size.eq(config.chunk_size),
                        project_rag_configs::chunk_overlap.eq(config.chunk_overlap),
                        project_rag_configs::top_k.eq(config.top_k),
                        project_rag_configs::min_score.eq(config.min_score),
                        project_rag_configs::temperature.eq(config.temperature),
                        project_rag_configs::updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .returning(ProjectRagConfig::as_returning())
                    .get_result(conn)
                    .map_err(RepositoryError::from)
            },
            Err(diesel::result::Error::NotFound) => {
                // 
                diesel::insert_into(project_rag_configs::table)
                    .values(config)
                    .returning(ProjectRagConfig::as_returning())
                    .get_result(conn)
                    .map_err(RepositoryError::from)
            },
            Err(e) => Err(RepositoryError::from(e)),
        }
    }
    
    /// RAG 配置
    pub fn delete(&self, project_id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(project_rag_configs::table.filter(project_rag_configs::project_id.eq(project_id)))
            .execute(conn)?;
        
        Ok(())
    }
}

// ============ 知识边界 Repository ============

///  Repository
#[derive(Clone)]
pub struct BoundaryRepository {
    pool: Arc<DbPool>,
}

impl BoundaryRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }
    
    /// 
    pub fn create(&self, boundary: &NewDocumentBoundary) -> Result<DocumentBoundary, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::insert_into(document_boundaries::table)
            .values(boundary)
            .returning(DocumentBoundary::as_returning())
            .get_result(conn)
            .map_err(RepositoryError::from)
    }
    
    /// ID 查询边界
    pub fn get_by_document_id(&self, document_id: i64) -> Result<DocumentBoundary, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        document_boundaries::table
            .filter(document_boundaries::document_id.eq(document_id))
            .first::<DocumentBoundary>(conn)
            .map_err(RepositoryError::from)
    }
    
    /// 
    pub fn update_boundary_type(&self, document_id: i64, boundary_type: &str) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::update(document_boundaries::table.filter(document_boundaries::document_id.eq(document_id)))
            .set((
                document_boundaries::boundary_type.eq(boundary_type),
                document_boundaries::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)?;
        
        Ok(())
    }
    
    /// 
    pub fn delete(&self, document_id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(document_boundaries::table.filter(document_boundaries::document_id.eq(document_id)))
            .execute(conn)?;
        
        Ok(())
    }
    
    pub fn set_visibility(&self, document_id: i64, visibility: &str, owner_id: Option<i64>, project_id: Option<i64>, team_id: Option<i64>) -> Result<DocumentBoundary, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        match self.get_by_document_id(document_id) {
            Ok(existing) => {
                diesel::update(document_boundaries::table.find(existing.id))
                    .set((
                        document_boundaries::boundary_type.eq(visibility),
                        document_boundaries::owner_id.eq(owner_id),
                        document_boundaries::project_id.eq(project_id),
                        document_boundaries::team_id.eq(team_id),
                        document_boundaries::updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .returning(DocumentBoundary::as_returning())
                    .get_result(conn)
                    .map_err(RepositoryError::from)
            }
            Err(RepositoryError::NotFound(_)) => {
                diesel::insert_into(document_boundaries::table)
                    .values(NewDocumentBoundary {
                        document_id,
                        boundary_type: Some(visibility.to_string()),
                        owner_id,
                        project_id,
                        team_id,
                    })
                    .returning(DocumentBoundary::as_returning())
                    .get_result(conn)
                    .map_err(RepositoryError::from)
            }
            Err(e) => Err(e),
        }
    }
  pub fn check_access(&self, document_id: i64, user_id: i64, user_projects: &[i64], user_teams: &[i64]) -> Result<bool, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let boundary = document_boundaries::table
            .filter(document_boundaries::document_id.eq(document_id))
            .first::<DocumentBoundary>(conn);
        
        match boundary {
            Ok(b) => {
                let boundary_type = b.boundary_type.as_deref().unwrap_or("private");
                
                match boundary_type {
                    "public" => Ok(true),
                    "org" => Ok(true),
                    "team" => {
                        if let Some(team_id) = b.team_id {
                            Ok(user_teams.contains(&team_id))
                        } else {
                            Ok(false)
                        }
                    }
                    "project" => {
                        if let Some(project_id) = b.project_id {
                            Ok(user_projects.contains(&project_id))
                        } else {
                            Ok(false)
                        }
                    }
                    "private" => {
                        Ok(b.owner_id == Some(user_id))
                    }
                    _ => Ok(false),
                }
            }
            Err(diesel::NotFound) => Ok(false),
            Err(e) => Err(RepositoryError::QueryError(e.to_string())),
        }
    }
    
    ///  ID 列表
    pub fn get_accessible_document_ids(&self, user_id: i64, user_projects: &[i64], user_teams: &[i64]) -> Result<Vec<i64>, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        document_boundaries::table
            .filter(
                document_boundaries::boundary_type.eq("public")
                    .or(document_boundaries::boundary_type.eq("org"))
                    .or(document_boundaries::owner_id.eq(user_id))
                    .or(document_boundaries::project_id.eq_any(user_projects))
                    .or(document_boundaries::team_id.eq_any(user_teams))
            )
            .select(document_boundaries::document_id)
            .load::<i64>(conn)
            .map_err(RepositoryError::from)
    }
}

// ============ 共享 Repository ============

///  Repository
#[derive(Clone)]
pub struct ShareRepository {
    pool: Arc<DbPool>,
}

impl ShareRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }

    /// 
    pub fn create(&self, share: &NewDocumentShare) -> Result<DocumentShare, RepositoryError> {
        use crate::schema::document_shares;
        diesel::insert_into(document_shares::table)
            .values(share)
            .returning(DocumentShare::as_returning())
            .get_result(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    pub fn get_by_document_id(&self, doc_id: i64) -> Result<Vec<DocumentShare>, RepositoryError> {
        use crate::schema::document_shares::dsl::*;
        document_shares
            .filter(document_id.eq(doc_id))
            .order(created_at.desc())
            .load::<DocumentShare>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn delete(&self, share_id: i64) -> Result<(), RepositoryError> {
        use crate::schema::document_shares::dsl::*;
        diesel::delete(document_shares.find(share_id))
            .execute(&mut self.get_conn()?)
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    pub fn delete_by_document_id(&self, doc_id: i64) -> Result<(), RepositoryError> {
        use crate::schema::document_shares::dsl::*;
        diesel::delete(document_shares.filter(document_id.eq(doc_id)))
            .execute(&mut self.get_conn()?)
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    /// 
    pub fn batch_create(&self, shares: &[NewDocumentShare]) -> Result<Vec<DocumentShare>, RepositoryError> {
        use crate::schema::document_shares;
        diesel::insert_into(document_shares::table)
            .values(shares)
            .returning(DocumentShare::as_returning())
            .get_results(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn check_share_access(&self, doc_id: i64, user_id: i64) -> Result<bool, RepositoryError> {
        use crate::schema::document_shares::dsl::*;
        
        let now = chrono::Utc::now().naive_utc();
        let count = document_shares
            .filter(document_id.eq(doc_id))
            .filter(target_type.eq("user"))
            .filter(target_id.eq(user_id))
            .filter(expire_at.is_null().or(expire_at.gt(now)))
            .count()
            .get_result::<i64>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)?;
        
        Ok(count > 0)
    }

    /// ID
    pub fn get_shared_document_ids(&self, user_id: i64) -> Result<Vec<i64>, RepositoryError> {
        use crate::schema::document_shares::dsl::*;
        
        let now = chrono::Utc::now().naive_utc();
        document_shares
            .filter(target_type.eq("user"))
            .filter(target_id.eq(user_id))
            .filter(expire_at.is_null().or(expire_at.gt(now)))
            .select(document_id)
            .load::<i64>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }
}

// ============ 分类 Repository ============

///  Repository
#[derive(Clone)]
pub struct CategoryRepository {
    pool: Arc<DbPool>,
}

impl CategoryRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }

    /// 
    pub fn create(&self, category: &NewDocumentCategory) -> Result<DocumentCategory, RepositoryError> {
        use crate::schema::document_categories;
        diesel::insert_into(document_categories::table)
            .values(category)
            .returning(DocumentCategory::as_returning())
            .get_result(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    ///  ID 获取分类
    pub fn get_by_id(&self, id: i64) -> Result<DocumentCategory, RepositoryError> {
        use crate::schema::document_categories::dsl::*;
        document_categories
            .filter(id.eq(id))
            .first::<DocumentCategory>(&mut self.get_conn()?)
            .map_err(|e| match e {
                diesel::NotFound => RepositoryError::NotFound("".to_string()),
                other => RepositoryError::QueryError(other.to_string()),
            })
    }

    pub fn list_root_categories(&self, proj_id: i64) -> Result<Vec<DocumentCategory>, RepositoryError> {
        use crate::schema::document_categories::dsl::*;
        document_categories
            .filter(project_id.eq(proj_id))
            .filter(parent_id.is_null())
            .filter(is_active.eq(true))
            .order(sort_order.asc())
            .load::<DocumentCategory>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    pub fn list_children(&self, parent: i64) -> Result<Vec<DocumentCategory>, RepositoryError> {
        use crate::schema::document_categories::dsl::*;
        document_categories
            .filter(parent_id.eq(parent))
            .filter(is_active.eq(true))
            .order(sort_order.asc())
            .load::<DocumentCategory>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn update(&self, id: i64, category: &NewDocumentCategory) -> Result<DocumentCategory, RepositoryError> {
        use crate::schema::document_categories::dsl::*;
        diesel::update(document_categories.find(id))
            .set((
                category_name.eq(category.category_name.as_deref()),
                category_type.eq(category.category_type.as_deref()),
                parent_id.eq(category.parent_id),
                level.eq(category.level),
                description.eq(category.description.as_deref()),
                icon.eq(category.icon.as_deref()),
                color.eq(category.color.as_deref()),
                sort_order.eq(category.sort_order),
                is_active.eq(category.is_active),
            ))
            .returning(DocumentCategory::as_returning())
            .get_result(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

  pub fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        use crate::schema::document_categories::dsl::*;
        diesel::update(document_categories.find(id))
            .set(is_active.eq(false))
            .execute(&mut self.get_conn()?)
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    /// 
    pub fn assign_categories(&self, doc_id: i64, mappings: &[NewDocumentCategoryMapping]) -> Result<Vec<DocumentCategoryMapping>, RepositoryError> {
        use crate::schema::document_category_mappings;
        let mut conn = self.get_conn()?;

        // 
        diesel::delete(document_category_mappings::table)
            .filter(document_category_mappings::document_id.eq(doc_id))
            .execute(&mut conn)
            .map_err(RepositoryError::from)?;

        diesel::insert_into(document_category_mappings::table)
            .values(mappings)
            .returning(DocumentCategoryMapping::as_returning())
            .get_results(&mut conn)
            .map_err(RepositoryError::from)
    }

    pub fn get_document_categories(&self, doc_id: i64) -> Result<Vec<DocumentCategory>, RepositoryError> {
        use crate::schema::{document_categories, document_category_mappings};
        let mut conn = self.get_conn()?;

        document_category_mappings::table
            .inner_join(document_categories::table.on(document_category_mappings::category_id.eq(document_categories::id)))
            .filter(document_category_mappings::document_id.eq(doc_id))
            .filter(document_categories::is_active.eq(true))
            .select(DocumentCategory::as_select())
            .order(document_category_mappings::is_primary.desc())
            .load::<DocumentCategory>(&mut conn)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn count_documents_in_category(&self, cat_id: i64) -> Result<i64, RepositoryError> {
        use crate::schema::document_category_mappings::dsl::*;
        document_category_mappings
            .filter(category_id.eq(cat_id))
            .count()
            .get_result::<i64>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }
}

// ============ 分级 Repository ============

///  Repository
#[derive(Clone)]
pub struct LevelRepository {
    pool: Arc<DbPool>,
}

impl LevelRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }

    /// 
    pub fn create(&self, level: &NewDocumentLevel) -> Result<DocumentLevel, RepositoryError> {
        use crate::schema::document_levels;
        diesel::insert_into(document_levels::table)
            .values(level)
            .returning(DocumentLevel::as_returning())
            .get_result(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    ///  ID 获取分级
    pub fn get_by_id(&self, id: i64) -> Result<DocumentLevel, RepositoryError> {
        use crate::schema::document_levels::dsl::*;
        document_levels
            .find(id)
            .first::<DocumentLevel>(&mut self.get_conn()?)
            .map_err(|e| match e {
                diesel::NotFound => RepositoryError::NotFound("".to_string()),
                other => RepositoryError::QueryError(other.to_string()),
            })
    }

 pub fn list_by_project(&self, proj_id: i64) -> Result<Vec<DocumentLevel>, RepositoryError> {
        use crate::schema::document_levels::dsl::*;
        document_levels
            .filter(project_id.eq(proj_id))
            .order(level_value.asc())
            .load::<DocumentLevel>(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn update(&self, _id: i64, level: &NewDocumentLevel) -> Result<DocumentLevel, RepositoryError> {
        use crate::schema::document_levels::dsl::*;
        diesel::update(document_levels.find(id))
            .set((
                level_name.eq(level.level_name.as_deref()),
                level_type.eq(level.level_type.as_deref()),
                level_value.eq(level.level_value),
                description.eq(level.description.as_deref()),
                icon.eq(level.icon.as_deref()),
                color.eq(level.color.as_deref()),
            ))
            .returning(DocumentLevel::as_returning())
            .get_result(&mut self.get_conn()?)
            .map_err(RepositoryError::from)
    }

    /// 
    pub fn delete(&self, _id: i64) -> Result<(), RepositoryError> {
        use crate::schema::document_levels::dsl::*;
        diesel::delete(document_levels.find(id))
            .execute(&mut self.get_conn()?)
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    /// 
    pub fn assign_levels(&self, doc_id: i64, mappings: &[NewDocumentLevelMapping]) -> Result<Vec<DocumentLevelMapping>, RepositoryError> {
        use crate::schema::document_level_mappings;
        let mut conn = self.get_conn()?;

        // 
        diesel::delete(document_level_mappings::table)
            .filter(document_level_mappings::document_id.eq(doc_id))
            .execute(&mut conn)
            .map_err(RepositoryError::from)?;

         diesel::insert_into(document_level_mappings::table)
            .values(mappings)
            .returning(DocumentLevelMapping::as_returning())
            .get_results(&mut conn)
            .map_err(RepositoryError::from)
    }

   pub fn get_document_levels(&self, doc_id: i64) -> Result<Vec<DocumentLevel>, RepositoryError> {
        use crate::schema::{document_levels, document_level_mappings};
        let mut conn = self.get_conn()?;

        document_level_mappings::table
            .inner_join(document_levels::table.on(document_level_mappings::level_id.eq(document_levels::id)))
            .filter(document_level_mappings::document_id.eq(doc_id))
            .select(DocumentLevel::as_select())
            .order(document_level_mappings::is_primary.desc())
            .load::<DocumentLevel>(&mut conn)
            .map_err(RepositoryError::from)
    }
}

// ============ 知识�?Repository ============

/// Repository
#[derive(Clone)]
pub struct KnowledgePointRepository {
    pool: Arc<DbPool>,
}

impl KnowledgePointRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, RepositoryError> {
        self.pool.get().map_err(|e| RepositoryError::ConnectionError(e.to_string()))
    }
    
 pub fn create(&self, point: &NewKnowledgePoint) -> Result<KnowledgePoint, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::insert_into(knowledge_points::table)
            .values(point)
            .returning(KnowledgePoint::as_returning())
            .get_result(conn)
            .map_err(RepositoryError::from)
    }
    
 pub fn batch_create(&self, points: &[NewKnowledgePoint]) -> Result<Vec<KnowledgePoint>, RepositoryError> {
        if points.is_empty() {
            return Ok(vec![]);
        }
        
        let conn = &mut self.get_conn()?;
        
        diesel::insert_into(knowledge_points::table)
            .values(points)
            .returning(KnowledgePoint::as_returning())
            .get_results(conn)
            .map_err(RepositoryError::from)
    }
    
    /// ID 查询
    pub fn get_by_id(&self, id: i64) -> Result<KnowledgePoint, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        knowledge_points::table
            .find(id)
            .first::<KnowledgePoint>(conn)
            .map_err(RepositoryError::from)
    }
    
    /// ID 查询所有知识点
    pub fn get_by_document_id(&self, document_id: i64) -> Result<Vec<KnowledgePoint>, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        knowledge_points::table
            .filter(knowledge_points::document_id.eq(document_id))
            .order(knowledge_points::created_at.desc())
            .load::<KnowledgePoint>(conn)
            .map_err(RepositoryError::from)
    }
    
 pub fn get_by_document_and_type(&self, document_id: i64, point_type: &str) -> Result<Vec<KnowledgePoint>, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        knowledge_points::table
            .filter(knowledge_points::document_id.eq(document_id))
            .filter(knowledge_points::point_type.eq(point_type))
            .order(knowledge_points::created_at.desc())
            .load::<KnowledgePoint>(conn)
            .map_err(RepositoryError::from)
    }
    
    pub fn count_by_document(&self, document_id: i64) -> Result<usize, RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        let count = knowledge_points::table
            .filter(knowledge_points::document_id.eq(document_id))
            .count()
            .get_result::<i64>(conn)? as usize;
        
        Ok(count)
    }
    
    pub fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(knowledge_points::table.filter(knowledge_points::id.eq(id)))
            .execute(conn)?;
        
        Ok(())
    }
    
    /// 
    pub fn delete_by_document_id(&self, document_id: i64) -> Result<(), RepositoryError> {
        let conn = &mut self.get_conn()?;
        
        diesel::delete(knowledge_points::table.filter(knowledge_points::document_id.eq(document_id)))
            .execute(conn)?;
        
        Ok(())
    }
}

// ============ 工厂函数 ============

/// 
pub fn create_pool(database_url: &str) -> Result<Arc<DbPool>, RepositoryError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
    
    Ok(Arc::new(pool))
}

/// Repository 组合结构
#[derive(Clone)]
pub struct Repositories {
    pub documents: DocumentRepository,
    pub chunks: ChunkRepository,
    pub rag_configs: RagConfigRepository,
    pub boundaries: BoundaryRepository,
    pub shares: ShareRepository,
    pub knowledge_points: KnowledgePointRepository,
    pub categories: CategoryRepository,
    pub levels: LevelRepository,
}

impl Repositories {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            documents: DocumentRepository::new(pool.clone()),
            chunks: ChunkRepository::new(pool.clone()),
            rag_configs: RagConfigRepository::new(pool.clone()),
            boundaries: BoundaryRepository::new(pool.clone()),
            shares: ShareRepository::new(pool.clone()),
            knowledge_points: KnowledgePointRepository::new(pool.clone()),
            categories: CategoryRepository::new(pool.clone()),
            levels: LevelRepository::new(pool),
        }
    }
}