//! Vec-svc 数据模型定义
//!
//! ?diesel ORM 操作

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::expression::SelectableHelper;
use serde::{Deserialize, Serialize};

// ============ 文档相关模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::documents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Document {
    pub id: i64,
    pub project_id: Option<i64>,
    pub title: Option<String>,
    pub topic: Option<String>,
    pub content: Option<String>,
    pub content_hash: Option<String>,
    pub source_type: Option<String>,
    pub source_url: Option<String>,
    pub file_path: Option<String>,
    pub file_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub boundary_level: Option<i32>,
    pub token_count: Option<i32>,
    pub version: i32,
    pub word_count: Option<i32>,
    pub chunk_count: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub indexed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::documents)]
pub struct NewDocument {
    pub project_id: Option<i64>,
    pub title: Option<String>,
    pub topic: Option<String>,
    pub content: Option<String>,
    pub content_hash: Option<String>,
    pub source_type: Option<String>,
    pub source_url: Option<String>,
    pub file_path: Option<String>,
    pub file_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub boundary_level: Option<i32>,
    pub token_count: Option<i32>,
    pub version: i32,
    pub word_count: Option<i32>,
    pub chunk_count: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::documents)]
pub struct DocumentUpdate {
    pub title: Option<String>,
    pub topic: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub chunk_count: Option<i32>,
    pub indexed_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// ============ 文档分块模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_chunks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentChunk {
    pub id: i64,
    pub document_id: i64,
    pub chunk_index: i32,
    pub chunk_text: Option<String>,
    pub embedding_status: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_chunks)]
pub struct NewDocumentChunk {
    pub document_id: i64,
    pub chunk_index: i32,
    pub chunk_text: Option<String>,
    pub embedding_status: Option<String>,
}

// ============ 项目 RAG 配置模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::project_rag_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectRagConfig {
    pub id: i64,
    pub project_id: Option<i64>,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub chunk_strategy: Option<String>,
    pub min_chunk_size: i32,
    pub top_k: i32,
    pub min_score: f64,
    pub rerank: bool,
    pub rerank_top_k: i32,
    pub search_type: Option<String>,
    pub temperature: f64,
    pub max_tokens: i32,
    pub context_window: i32,
    pub batch_size: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::project_rag_configs)]
pub struct NewProjectRagConfig {
    pub project_id: Option<i64>,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub chunk_strategy: Option<String>,
    pub min_chunk_size: i32,
    pub top_k: i32,
    pub min_score: f64,
    pub rerank: bool,
    pub rerank_top_k: i32,
    pub search_type: Option<String>,
    pub temperature: f64,
    pub max_tokens: i32,
    pub context_window: i32,
    pub batch_size: i32,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::project_rag_configs)]
pub struct ProjectRagConfigUpdate {
    pub chunk_size: Option<i32>,
    pub chunk_overlap: Option<i32>,
    pub top_k: Option<i32>,
    pub min_score: Option<f64>,
    pub temperature: Option<f64>,
    pub updated_at: Option<NaiveDateTime>,
}

// ============ 知识点模�?============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::knowledge_points)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgePoint {
    pub id: i64,
    pub document_id: i64,
    pub point_type: Option<String>,
    pub point_content: Option<String>,
    pub confidence: Option<f64>,
    pub keywords: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::knowledge_points)]
pub struct NewKnowledgePoint {
    pub document_id: i64,
    pub point_type: Option<String>,
    pub point_content: Option<String>,
    pub confidence: Option<f64>,
    pub keywords: Option<serde_json::Value>,
}

// ============ 知识实体模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::knowledge_entities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeEntity {
    pub id: i64,
    pub project_id: Option<i64>,
    pub name: Option<String>,
    pub entity_type: Option<String>,
    pub description: Option<String>,
    pub aliases: Option<serde_json::Value>,
    pub confidence: Option<f64>,
    pub source_document_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::knowledge_entities)]
pub struct NewKnowledgeEntity {
    pub project_id: Option<i64>,
    pub name: Option<String>,
    pub entity_type: Option<String>,
    pub description: Option<String>,
    pub aliases: Option<serde_json::Value>,
    pub confidence: Option<f64>,
    pub source_document_id: Option<i64>,
}

// ============ 知识关系模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::knowledge_relations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeRelation {
    pub id: i64,
    pub project_id: Option<i64>,
    pub source_entity_id: i64,
    pub target_entity_id: i64,
    pub relation_type: Option<String>,
    pub relation_strength: Option<f64>,
    pub evidence_text: Option<String>,
    pub source_document_id: Option<i64>,
    pub confidence: Option<f64>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::knowledge_relations)]
pub struct NewKnowledgeRelation {
    pub project_id: Option<i64>,
    pub source_entity_id: i64,
    pub target_entity_id: i64,
    pub relation_type: Option<String>,
    pub relation_strength: Option<f64>,
    pub evidence_text: Option<String>,
    pub source_document_id: Option<i64>,
    pub confidence: Option<f64>,
}

// ============ 知识边界模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_boundaries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentBoundary {
    pub id: i64,
    pub document_id: i64,
    pub boundary_type: Option<String>,
    pub owner_id: Option<i64>,
    pub project_id: Option<i64>,
    pub team_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_boundaries)]
pub struct NewDocumentBoundary {
    pub document_id: i64,
    pub boundary_type: Option<String>,
    pub owner_id: Option<i64>,
    pub project_id: Option<i64>,
    pub team_id: Option<i64>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_shares)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentShare {
    pub id: i64,
    pub document_id: i64,
    pub share_type: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub granted_by: Option<i64>,
    pub expire_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_shares)]
pub struct NewDocumentShare {
    pub document_id: i64,
    pub share_type: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub granted_by: Option<i64>,
    pub expire_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::verification_conflicts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VerificationConflict {
    pub id: i64,
    pub project_id: Option<i64>,
    pub query_text: Option<String>,
    pub llm_summary: Option<String>,
    pub conflict_type: Option<String>,
    pub conflict_description: Option<String>,
    pub confidence_score: Option<f64>,
    pub resolved: bool,
    pub resolution: Option<String>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

// ============ 边界类型枚举 ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisibilityLevel {
    Private,
    Project,
    Team,
    Org,
    Public,
}

impl VisibilityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            VisibilityLevel::Private => "private",
            VisibilityLevel::Project => "project",
            VisibilityLevel::Team => "team",
            VisibilityLevel::Org => "org",
            VisibilityLevel::Public => "public",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "private" => Some(VisibilityLevel::Private),
            "project" => Some(VisibilityLevel::Project),
            "team" => Some(VisibilityLevel::Team),
            "org" => Some(VisibilityLevel::Org),
            "public" => Some(VisibilityLevel::Public),
            _ => None,
        }
    }
}

// ============ 嵌入状态枚�?============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingStatus {
    Pending,
    Processing,
    Done,
    Failed,
}

impl EmbeddingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmbeddingStatus::Pending => "pending",
            EmbeddingStatus::Processing => "processing",
            EmbeddingStatus::Done => "done",
            EmbeddingStatus::Failed => "failed",
        }
    }
}

// ============ 辅助函数 ============

pub fn now() -> NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

// ============ 分类模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentCategory {
    pub id: i64,
    pub project_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_type: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_categories)]
pub struct NewDocumentCategory {
    pub project_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_type: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

// ============ 分级模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_levels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentLevel {
    pub id: i64,
    pub project_id: Option<i64>,
    pub level_name: Option<String>,
    pub level_type: Option<String>,
    pub level_value: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_levels)]
pub struct NewDocumentLevel {
    pub project_id: Option<i64>,
    pub level_name: Option<String>,
    pub level_type: Option<String>,
    pub level_value: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

// ============ 文档-分类关联模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_category_mappings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentCategoryMapping {
    pub id: i64,
    pub document_id: i64,
    pub category_id: i64,
    pub confidence: Option<f64>,
    pub is_primary: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_category_mappings)]
pub struct NewDocumentCategoryMapping {
    pub document_id: i64,
    pub category_id: i64,
    pub confidence: Option<f64>,
    pub is_primary: bool,
}

// ============ 文档-分级关联模型 ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::document_level_mappings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentLevelMapping {
    pub id: i64,
    pub document_id: i64,
    pub level_id: i64,
    pub confidence: Option<f64>,
    pub is_primary: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::document_level_mappings)]
pub struct NewDocumentLevelMapping {
    pub document_id: i64,
    pub level_id: i64,
    pub confidence: Option<f64>,
    pub is_primary: bool,
}
