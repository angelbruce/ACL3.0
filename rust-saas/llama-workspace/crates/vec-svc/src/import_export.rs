//! Import/Export service
//!
//! Data import and export functionality

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, RunQueryDsl};

use crate::rdb_repository::DbPool;
use crate::model::{Document, NewDocument, DocumentChunk, NewDocumentChunk, KnowledgeEntity, KnowledgeRelation};
use crate::schema;

pub struct ImportExportService {
    db_pool: Arc<DbPool>,
}

impl ImportExportService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    pub async fn import_documents(
        &self,
        project_id: Option<i64>,
        documents: Vec<ImportDocument>,
    ) -> Result<ImportResult, String> {
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();
        let mut document_ids = Vec::new();

        for (index, doc) in documents.iter().enumerate() {
            match self.import_single_document(project_id, doc).await {
                Ok(id) => {
                    success_count += 1;
                    document_ids.push(id);
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(ImportError {
                        index,
                        title: doc.title.clone(),
                        error: e,
                    });
                }
            }
        }

        Ok(ImportResult {
            success_count,
            failed_count,
            total_count: documents.len(),
            errors,
            document_ids,
        })
    }

    async fn import_single_document(
        &self,
        project_id: Option<i64>,
        doc: &ImportDocument,
    ) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let now = chrono::Utc::now().naive_utc();
        let content_hash = format!("{:x}", md5_hash(&doc.content));

        let document_id = diesel::insert_into(schema::documents::table)
            .values((
                schema::documents::project_id.eq(project_id),
                schema::documents::title.eq(Some(&doc.title)),
                schema::documents::topic.eq(Some(&doc.title)),
                schema::documents::content.eq(&doc.content),
                schema::documents::content_hash.eq(&content_hash),
                schema::documents::status.eq("processing"),
                schema::documents::chunk_count.eq(0),
                schema::documents::token_count.eq(doc.content.len() as i32),
                schema::documents::created_at.eq(now),
                schema::documents::updated_at.eq(now),
            ))
            .returning(schema::documents::id)
            .get_result::<i64>(&mut conn)
            .map_err(|e| format!("Failed to insert document: {}", e))?;

        Ok(document_id)
    }

    pub async fn export_documents(
        &self,
        project_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
        format: ExportFormat,
    ) -> Result<ExportResult, String> {
        let conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let mut query = schema::documents::table.into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(schema::documents::project_id.eq(pid));
        }

        if let Some(ids) = &document_ids {
            query = query.filter(schema::documents::id.eq_any(ids));
        }

        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let documents: Vec<Document> = query
            .load(&mut conn)
            .map_err(|e| format!("Failed to load documents: {}", e))?;

        let export_docs: Vec<ExportDocument> = documents
            .iter()
            .map(|d| ExportDocument {
                id: d.id,
                title: d.title.clone().unwrap_or_default(),
                content: d.content.clone().unwrap_or_default(),
                project_id: d.project_id,
                created_at: Some(d.created_at.to_string()),
            })
            .collect();

        let content = match format {
            ExportFormat::Json => serde_json::to_string_pretty(&export_docs)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?,
            ExportFormat::Markdown => self.export_markdown(&export_docs),
            ExportFormat::Csv => self.export_csv(&export_docs),
        };

        Ok(ExportResult {
            content,
            format,
            document_count: export_docs.len(),
        })
    }

    fn export_markdown(&self, documents: &[ExportDocument]) -> String {
        let mut md = String::new();

        for doc in documents {
            md.push_str(&format!("# {}\n\n", doc.title));
            if let Some(created) = &doc.created_at {
                md.push_str(&format!("*创建时间: {}*\n\n", created));
            }
            md.push_str(&doc.content);
            md.push_str("\n\n---\n\n");
        }

        md
    }

    fn export_csv(&self, documents: &[ExportDocument]) -> String {
        let mut csv = String::from("id,title,project_id,created_at,content\n");

        for doc in documents {
            let content = doc.content.replace('"', "\"\"").replace('\n', " ");
            csv.push_str(&format!(
                "{},\"{}\",{},{},\"{}\"\n",
                doc.id,
                doc.title.replace('"', "\"\""),
                doc.project_id.unwrap_or(0),
                doc.created_at.as_deref().unwrap_or(""),
                content
            ));
        }

        csv
    }

    pub async fn import_from_zip(&self, _project_id: Option<i64>, _zip_data: &[u8]) -> Result<ImportResult, String> {
        Err("Zip import requires additional dependencies (zip crate)".to_string())
    }

    pub async fn export_knowledge_graph(
        &self,
        project_id: Option<i64>,
    ) -> Result<KnowledgeGraphExport, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let entities: Vec<KnowledgeEntity> = {
            let mut query = schema::knowledge_entities::table.into_boxed();
            if let Some(pid) = project_id {
                query = query.filter(schema::knowledge_entities::project_id.eq(pid));
            }
            query
                .load(&mut conn)
                .map_err(|e| format!("Failed to load entities: {}", e))?
        };

        let relations: Vec<KnowledgeRelation> = {
            let mut query = schema::knowledge_relations::table.into_boxed();
            if let Some(pid) = project_id {
                query = query.filter(schema::knowledge_relations::project_id.eq(pid));
            }
            query
                .load(&mut conn)
                .map_err(|e| format!("Failed to load relations: {}", e))?
        };

        let export_entities: Vec<ExportEntity> = entities
            .iter()
            .map(|e| ExportEntity {
                id: e.id,
                name: e.name.clone().unwrap_or_default(),
                entity_type: e.entity_type.clone().unwrap_or_default(),
                description: e.description.clone(),
            })
            .collect();

        let export_relations: Vec<ExportRelation> = relations
            .iter()
            .map(|r| ExportRelation {
                id: r.id,
                source_id: r.source_entity_id,
                target_id: r.target_entity_id,
                relation_type: r.relation_type.clone().unwrap_or_default(),
                evidence: r.evidence_text.clone(),
            })
            .collect();

        let entity_count = export_entities.len();
        let relation_count = export_relations.len();

        Ok(KnowledgeGraphExport {
            entities: export_entities,
            relations: export_relations,
            entity_count,
            relation_count,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDocument {
    pub title: String,
    pub content: String,
    pub source: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_count: usize,
    pub errors: Vec<ImportError>,
    pub document_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub index: usize,
    pub title: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Markdown,
    Csv,
}

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub content: String,
    pub format: ExportFormat,
    pub document_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ExportDocument {
    id: i64,
    title: String,
    content: String,
    project_id: Option<i64>,
    created_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KnowledgeGraphExport {
    pub entities: Vec<ExportEntity>,
    pub relations: Vec<ExportRelation>,
    pub entity_count: usize,
    pub relation_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportEntity {
    pub id: i64,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportRelation {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub relation_type: String,
    pub evidence: Option<String>,
}

fn md5_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}
