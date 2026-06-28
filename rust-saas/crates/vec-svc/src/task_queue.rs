
use std::sync::Arc;
use std::time::Duration;

use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, TextExpressionMethods, RunQueryDsl};
use tokio::sync::RwLock;

use crate::rdb_repository::{DbPool, create_pool};
use crate::model::{Document, NewDocument, DocumentChunk, NewDocumentChunk};
use crate::embedding::EmbeddingService;
use crate::distillation::DistillationService;
use crate::milvus::{MilvusClient, MilvusConfig};
use crate::knowledge_graph::KnowledgeGraphService;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    DocumentProcess,
    DocumentReindex,
    DocumentDistill,
    BatchProcess,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::DocumentProcess => "document_process",
            TaskType::DocumentReindex => "document_reindex",
            TaskType::DocumentDistill => "document_distill",
            TaskType::BatchProcess => "batch_process",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Processing => "processing",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Task {
    pub id: i64,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub payload: serde_json::Value,
    pub progress: f32,
    pub message: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub started_at: Option<chrono::NaiveDateTime>,
    pub completed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewTask {
    pub task_type: TaskType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskProgress {
    pub task_id: i64,
    pub status: TaskStatus,
    pub progress: f32,
    pub message: Option<String>,
}

pub struct TaskQueue {
    db_pool: Arc<DbPool>,
    workers: RwLock<Vec<tokio::task::JoinHandle<()>>>,
    stop_signal: Arc<tokio::sync::Notify>,
    embedding_service: EmbeddingService,
    distillation_service: DistillationService,
    milvus_client: MilvusClient,
    milvus_config: MilvusConfig,
    knowledge_graph_service: KnowledgeGraphService,
}

impl TaskQueue {
    pub fn new(
        db_pool: Arc<DbPool>,
        embedding_service: EmbeddingService,
        distillation_service: DistillationService,
        milvus_client: MilvusClient,
        milvus_config: MilvusConfig,
        knowledge_graph_service: KnowledgeGraphService,
    ) -> Self {
        Self {
            db_pool,
            workers: RwLock::new(Vec::new()),
            stop_signal: Arc::new(tokio::sync::Notify::new()),
            embedding_service,
            distillation_service,
            milvus_client,
            milvus_config,
            knowledge_graph_service,
        }
    }

    pub async fn enqueue(&self, task: NewTask) -> Result<Task, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        let now = chrono::Utc::now().naive_utc();
        
        let task_id = diesel::insert_into(crate::schema::tasks::table)
            .values((
                crate::schema::tasks::task_type.eq(task.task_type.as_str()),
                crate::schema::tasks::status.eq("pending"),
                crate::schema::tasks::payload.eq(&task.payload),
                crate::schema::tasks::progress.eq(0.0),
                crate::schema::tasks::created_at.eq(now),
            ))
            .returning(crate::schema::tasks::id)
            .get_result::<i64>(&mut conn)
            .map_err(|e| format!("Failed to enqueue task: {}", e))?;

        self.get_task(task_id).await
    }

    pub async fn get_task(&self, task_id: i64) -> Result<Task, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        let row = crate::schema::tasks::table
            .filter(crate::schema::tasks::id.eq(task_id))
            .first::<TaskRow>(&mut conn)
            .map_err(|e| format!("Failed to get task: {}", e))?;

        Ok(row.into())
    }

    pub async fn update_task_status(&self, task_id: i64, status: TaskStatus, message: Option<String>) -> Result<(), String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        let now = chrono::Utc::now().naive_utc();
        
        diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
            .set((
                crate::schema::tasks::status.eq(status.as_str()),
                crate::schema::tasks::message.eq(message),
                crate::schema::tasks::completed_at.eq(if status == TaskStatus::Completed || status == TaskStatus::Failed || status == TaskStatus::Cancelled { Some(now) } else { None }),
                crate::schema::tasks::started_at.eq(if status == TaskStatus::Processing { Some(now) } else { None }),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update task status: {}", e))?;

        Ok(())
    }

    pub async fn update_task_progress(&self, task_id: i64, progress: f32, message: Option<String>) -> Result<(), String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
            .set((
                crate::schema::tasks::progress.eq(progress),
                crate::schema::tasks::message.eq(message),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update task progress: {}", e))?;

        Ok(())
    }

    pub async fn cancel_task(&self, task_id: i64) -> Result<(), String> {
        self.update_task_status(task_id, TaskStatus::Cancelled, Some("Task cancelled".to_string())).await
    }

    pub async fn start_workers(&self, num_workers: usize) {
        let mut workers = self.workers.write().await;

        for _ in 0..num_workers {
            let db_pool = self.db_pool.clone();
            let stop_signal = self.stop_signal.clone();
            let embedding_service = self.embedding_service.clone();
            let distillation_service = self.distillation_service.clone();
            let milvus_client = self.milvus_client.clone();
            let milvus_config = self.milvus_config.clone();
            let knowledge_graph_service = self.knowledge_graph_service.clone();

            let handle = tokio::spawn(async move {
                worker_loop(db_pool, stop_signal, embedding_service, distillation_service, milvus_client, milvus_config, knowledge_graph_service).await;
            });

            workers.push(handle);
        }
    }

    pub async fn stop_workers(&self) {
        self.stop_signal.notify_waiters();
        
        let mut workers = self.workers.write().await;
        for handle in workers.drain(..) {
            let _ = handle.await;
        }
    }

    pub async fn list_tasks(&self, status: Option<String>, limit: usize) -> Result<Vec<Task>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        let mut query = crate::schema::tasks::table
            .order(crate::schema::tasks::created_at.desc())
            .limit(limit as i64)
            .into_boxed();
        
        if let Some(s) = status {
            query = query.filter(crate::schema::tasks::status.eq(s));
        }
        
        let rows = query.load::<TaskRow>(&mut conn)
            .map_err(|e| format!("Failed to list tasks: {}", e))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::tasks)]
struct TaskRow {
    id: i64,
    task_type: String,
    status: String,
    payload: serde_json::Value,
    progress: f32,
    message: Option<String>,
    created_at: chrono::NaiveDateTime,
    started_at: Option<chrono::NaiveDateTime>,
    completed_at: Option<chrono::NaiveDateTime>,
}

impl From<TaskRow> for Task {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            task_type: match row.task_type.as_str() {
                "document_process" => TaskType::DocumentProcess,
                "document_reindex" => TaskType::DocumentReindex,
                "document_distill" => TaskType::DocumentDistill,
                "batch_process" => TaskType::BatchProcess,
                _ => TaskType::DocumentProcess,
            },
            status: match row.status.as_str() {
                "pending" => TaskStatus::Pending,
                "processing" => TaskStatus::Processing,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                "cancelled" => TaskStatus::Cancelled,
                _ => TaskStatus::Pending,
            },
            payload: row.payload,
            progress: row.progress,
            message: row.message,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

async fn worker_loop(
    db_pool: Arc<DbPool>,
    stop_signal: Arc<tokio::sync::Notify>,
    embedding_service: EmbeddingService,
    distillation_service: DistillationService,
    milvus_client: MilvusClient,
    milvus_config: MilvusConfig,
    knowledge_graph_service: KnowledgeGraphService,
) {
    loop {
        tokio::select! {
            _ = stop_signal.notified() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                if let Err(e) = process_next_task(&db_pool, &embedding_service, &distillation_service, &milvus_client, &milvus_config, &knowledge_graph_service).await {
                    tracing::error!("Worker error: {}", e);
                }
            }
        }
    }
}

async fn process_next_task(
    db_pool: &Arc<DbPool>,
    embedding_service: &EmbeddingService,
    distillation_service: &DistillationService,
    milvus_client: &MilvusClient,
    milvus_config: &MilvusConfig,
    knowledge_graph_service: &KnowledgeGraphService,
) -> Result<(), String> {
    let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    
    let row = crate::schema::tasks::table
        .filter(crate::schema::tasks::status.eq("pending"))
        .order(crate::schema::tasks::created_at.asc())
        .first::<TaskRow>(&mut conn)
        .optional()
        .map_err(|e| format!("Failed to fetch next task: {}", e))?;
    
    if let Some(task_row) = row {
        let task_id = task_row.id;
        
        diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
            .set((
                crate::schema::tasks::status.eq("processing"),
                crate::schema::tasks::started_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to mark task as processing: {}", e))?;

        let result = match task_row.task_type.as_str() {
            "document_process" => process_document_task(task_id, &task_row.payload, db_pool, embedding_service, milvus_client, milvus_config, knowledge_graph_service).await,
            "document_reindex" => process_reindex_task(task_id, &task_row.payload, db_pool, embedding_service, milvus_client, milvus_config, knowledge_graph_service).await,
            "document_distill" => process_distill_task(task_id, &task_row.payload, db_pool, distillation_service).await,
            "batch_process" => process_batch_task(task_id, &task_row.payload, db_pool, embedding_service).await,
            _ => Err(format!("Unknown task type: {}", task_row.task_type)),
        };

        let mut conn2 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        match result {
            Ok(_) => {
                diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
                    .set((
                        crate::schema::tasks::status.eq("completed"),
                        crate::schema::tasks::progress.eq(100.0),
                        crate::schema::tasks::completed_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(&mut conn2)
                    .map_err(|e| format!("Failed to mark task as completed: {}", e))?;
            }
            Err(e) => {
                diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
                    .set((
                        crate::schema::tasks::status.eq("failed"),
                        crate::schema::tasks::message.eq(Some(e.clone())),
                        crate::schema::tasks::completed_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(&mut conn2)
                    .map_err(|e| format!("Failed to mark task as failed: {}", e))?;
                
                tracing::error!("Task {} failed: {}", task_id, e);
            }
        }
    }

    Ok(())
}

async fn process_document_task(
    task_id: i64,
    payload: &serde_json::Value,
    db_pool: &Arc<DbPool>,
    embedding_service: &EmbeddingService,
    milvus_client: &MilvusClient,
    milvus_config: &MilvusConfig,
    knowledge_graph_service: &KnowledgeGraphService,
) -> Result<(), String> {
    let document_id: Option<i64> = payload.get("document_id").and_then(|v| v.as_i64());
    let topic: Option<String> = payload.get("topic").and_then(|v| v.as_str()).map(|s| s.to_string());
    let project_id: Option<i64> = payload.get("project_id").and_then(|v| v.as_i64());
    let source_type: Option<String> = payload.get("source_type").and_then(|v| v.as_str()).map(|s| s.to_string());
    let source_url: Option<String> = payload.get("source_url").and_then(|v| v.as_str()).map(|s| s.to_string());
    let file_path: Option<String> = payload.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
    let file_type: Option<String> = payload.get("file_type").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let (doc_id, doc_content, chunk_project_id) = if let Some(input_id) = document_id {
        use crate::schema::documents::dsl::*;
        let doc: crate::model::Document = documents.filter(id.eq(input_id)).first(&mut conn)
            .map_err(|e| format!("Failed to get document: {}", e))?;
        let doc_text = doc.content.unwrap_or_default();
        if doc_text.is_empty() {
            return Err("Document content is empty".to_string());
        }
        (doc.id, doc_text, doc.project_id)
    } else {
        let doc_content: String = payload.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let new_doc = NewDocument {
            project_id,
            title: None,
            topic: topic.clone(),
            content: Some(doc_content.clone()),
            content_hash: None,
            source_type: source_type.clone(),
            source_url: source_url.clone(),
            file_path: file_path.clone(),
            file_type: file_type.clone(),
            status: None,
            visibility: None,
            boundary_level: None,
            token_count: None,
            version: 1,
            word_count: None,
            chunk_count: 0,
            metadata: None,
        };
        let doc_id: i64 = diesel::insert_into(crate::schema::documents::table)
            .values(&new_doc)
            .returning(crate::schema::documents::id)
            .get_result(&mut conn)
            .map_err(|e| format!("Failed to create document: {}", e))?;
        (doc_id, doc_content, project_id)
    };

    update_progress(db_pool, task_id, 10.0, Some("Chunking document".to_string())).await?;

    let chunks: Vec<crate::app_state::TextChunk> = chunk_text(&doc_content, chunk_project_id, db_pool).await?;
        let chunk_count = chunks.len();

    update_progress(db_pool, task_id, 20.0, Some("Vectorizing chunks".to_string())).await?;

    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let documents_for_milvus = texts.clone();
    let embedding_dim = embedding_service.config().embedding_dim;
    let embedding_service = embedding_service.clone();
    let vectors = tokio::task::spawn_blocking(move || {
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        embedding_service.embed_batch(&text_refs)
    }).await.map_err(|e| format!("Spawn blocking error: {}", e))??;

    update_progress(db_pool, task_id, 60.0, Some("Storing chunks".to_string())).await?;

    // Remove existing chunks and vectors if reprocessing
    if document_id.is_some() {
        let old_chunk_ids: Vec<i64> = crate::schema::document_chunks::table
            .filter(crate::schema::document_chunks::document_id.eq(doc_id))
            .select(crate::schema::document_chunks::id)
            .load(&mut conn)
            .map_err(|e| format!("Failed to get old chunk ids: {}", e))?;

        delete_vectors_by_chunk_ids(milvus_client, milvus_config, chunk_project_id, &old_chunk_ids).await?;

        diesel::delete(crate::schema::document_chunks::table.filter(crate::schema::document_chunks::document_id.eq(doc_id)))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to delete old chunks: {}", e))?;
    }

    let new_chunks: Vec<NewDocumentChunk> = chunks
        .iter()
        .enumerate()
        .map(|(i, chunk)| NewDocumentChunk {
            document_id: doc_id,
            chunk_index: i as i32,
            chunk_text: Some(chunk.content.clone()),
            embedding_status: Some("done".to_string()),
        })
        .collect();

    let inserted_chunk_ids: Vec<i64> = diesel::insert_into(crate::schema::document_chunks::table)
        .values(&new_chunks)
        .returning(crate::schema::document_chunks::id)
        .get_results(&mut conn)
        .map_err(|e| format!("Failed to insert chunks: {}", e))?;

    update_progress(db_pool, task_id, 70.0, Some("Storing vectors".to_string())).await?;

    let project_ids: Vec<Option<i64>> = vec![chunk_project_id; vectors.len()];
    let collection_name = get_collection_name(milvus_config, chunk_project_id);
    milvus_client.create_collection(&collection_name, embedding_dim).await.map_err(|e| e.to_string())?;
    milvus_client.insert(&collection_name, vectors, documents_for_milvus, inserted_chunk_ids, project_ids).await.map_err(|e| e.to_string())?;

    update_progress(db_pool, task_id, 90.0, Some("Extracting entities".to_string())).await?;

    // Extract entities for knowledge graph
    if let Err(e) = knowledge_graph_service.extract_from_document(doc_id, &doc_content, chunk_project_id).await {
        tracing::warn!("Entity extraction failed for document {}: {}", doc_id, e);
    }

    update_progress(db_pool, task_id, 95.0, Some("Finalizing".to_string())).await?;

    diesel::update(crate::schema::documents::table.filter(crate::schema::documents::id.eq(doc_id)))
        .set((
            crate::schema::documents::chunk_count.eq(chunk_count as i32),
            crate::schema::documents::indexed_at.eq(chrono::Utc::now().naive_utc()),
            crate::schema::documents::status.eq(Some("completed".to_string())),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update document: {}", e))?;

    Ok(())
}

async fn process_reindex_task(
    task_id: i64,
    payload: &serde_json::Value,
    db_pool: &Arc<DbPool>,
    embedding_service: &EmbeddingService,
    milvus_client: &MilvusClient,
    milvus_config: &MilvusConfig,
    knowledge_graph_service: &KnowledgeGraphService,
) -> Result<(), String> {
    let document_id: i64 = payload.get("document_id").and_then(|v| v.as_i64()).unwrap_or(0);

    let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    let doc: Document = crate::schema::documents::table
        .filter(crate::schema::documents::id.eq(document_id))
        .first(&mut conn)
        .map_err(|e| format!("Failed to get document: {}", e))?;

    let content = doc.content.clone().unwrap_or_default();
    let project_id = doc.project_id;

    update_progress(db_pool, task_id, 10.0, Some("Deleting old chunks and vectors".to_string())).await?;

    let old_chunk_ids: Vec<i64> = crate::schema::document_chunks::table
        .filter(crate::schema::document_chunks::document_id.eq(document_id))
        .select(crate::schema::document_chunks::id)
        .load(&mut conn)
        .map_err(|e| format!("Failed to get old chunk ids: {}", e))?;

    delete_vectors_by_chunk_ids(milvus_client, milvus_config, project_id, &old_chunk_ids).await?;

    let mut conn2 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    diesel::delete(crate::schema::document_chunks::table.filter(crate::schema::document_chunks::document_id.eq(document_id)))
        .execute(&mut conn2)
        .map_err(|e| format!("Failed to delete chunks: {}", e))?;

    let chunks = chunk_text(&content, project_id, db_pool).await?;
    let chunk_count = chunks.len();
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let documents_for_milvus = texts.clone();
    let embedding_dim = embedding_service.config().embedding_dim;

    update_progress(db_pool, task_id, 30.0, Some("Vectorizing".to_string())).await?;

    let embedding_service = embedding_service.clone();
    let vectors = tokio::task::spawn_blocking(move || {
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        embedding_service.embed_batch(&text_refs)
    }).await.map_err(|e| format!("Spawn blocking error: {}", e))??;

    let new_chunks: Vec<NewDocumentChunk> = chunks
        .iter()
        .enumerate()
        .map(|(i, chunk)| NewDocumentChunk {
            document_id,
            chunk_index: i as i32,
            chunk_text: Some(chunk.content.clone()),
            embedding_status: Some("done".to_string()),
        })
        .collect();

    update_progress(db_pool, task_id, 60.0, Some("Storing chunks".to_string())).await?;

    let mut conn3 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    let inserted_chunk_ids: Vec<i64> = diesel::insert_into(crate::schema::document_chunks::table)
        .values(&new_chunks)
        .returning(crate::schema::document_chunks::id)
        .get_results(&mut conn3)
        .map_err(|e| format!("Failed to insert chunks: {}", e))?;

    update_progress(db_pool, task_id, 80.0, Some("Storing vectors".to_string())).await?;

    let project_ids: Vec<Option<i64>> = vec![project_id; vectors.len()];
    let collection_name = get_collection_name(milvus_config, project_id);
    milvus_client.create_collection(&collection_name, embedding_dim).await.map_err(|e| e.to_string())?;
    milvus_client.insert(&collection_name, vectors, documents_for_milvus, inserted_chunk_ids, project_ids).await.map_err(|e| e.to_string())?;

    update_progress(db_pool, task_id, 90.0, Some("Extracting entities".to_string())).await?;

    // Extract entities for knowledge graph
    if let Err(e) = knowledge_graph_service.extract_from_document(document_id, &content, project_id).await {
        tracing::warn!("Entity extraction failed for document {}: {}", document_id, e);
    }

    let mut conn4 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    diesel::update(crate::schema::documents::table.filter(crate::schema::documents::id.eq(document_id)))
        .set((
            crate::schema::documents::indexed_at.eq(chrono::Utc::now().naive_utc()),
            crate::schema::documents::chunk_count.eq(chunk_count as i32),
        ))
        .execute(&mut conn4)
        .map_err(|e| format!("Failed to update document: {}", e))?;

    Ok(())
}

async fn process_distill_task(
    task_id: i64,
    payload: &serde_json::Value,
    db_pool: &Arc<DbPool>,
    distillation_service: &DistillationService,
) -> Result<(), String> {
    let document_id: i64 = payload.get("document_id").and_then(|v| v.as_i64()).unwrap_or(0);

    let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    let doc: Document = crate::schema::documents::table
        .filter(crate::schema::documents::id.eq(document_id))
        .first(&mut conn)
        .map_err(|e| format!("Failed to get document: {}", e))?;

    let content = doc.content.clone().unwrap_or_default();

    update_progress(db_pool, task_id, 20.0, Some("Distilling content".to_string())).await?;

    let distillation_clone = distillation_service.clone();
    let content_clone = content.clone();
    let new_points = tokio::task::spawn_blocking(move || {
        distillation_clone.distill(&content_clone, document_id)
    }).await.map_err(|e| format!("Spawn blocking error: {}", e))??;

    update_progress(db_pool, task_id, 60.0, Some("Saving knowledge points".to_string())).await?;

    let mut conn2 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    diesel::delete(crate::schema::knowledge_points::table.filter(crate::schema::knowledge_points::document_id.eq(document_id)))
        .execute(&mut conn2)
        .map_err(|e| format!("Failed to delete old points: {}", e))?;

    let mut conn3 = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    diesel::insert_into(crate::schema::knowledge_points::table)
        .values(&new_points)
        .execute(&mut conn3)
        .map_err(|e| format!("Failed to insert points: {}", e))?;

    Ok(())
}

async fn process_batch_task(
    task_id: i64,
    payload: &serde_json::Value,
    db_pool: &Arc<DbPool>,
    embedding_service: &EmbeddingService,
) -> Result<(), String> {
    let documents: Vec<serde_json::Value> = payload.get("documents")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .clone();

    let total = documents.len();
    
    for (i, doc_payload) in documents.iter().enumerate() {
        let topic: Option<String> = doc_payload.get("topic").and_then(|v| v.as_str()).map(|s| s.to_string());
        let content: String = doc_payload.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let project_id: Option<i64> = doc_payload.get("project_id").and_then(|v| v.as_i64());

        if !content.is_empty() {
            let new_doc = NewDocument {
                project_id,
                title: None,
                topic,
                content: Some(content),
                content_hash: None,
                source_type: None,
                source_url: None,
                file_path: None,
                file_type: None,
                status: None,
                visibility: None,
                boundary_level: None,
                token_count: None,
                version: 1,
                word_count: None,
                chunk_count: 0,
                metadata: None,
            };

            let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
            let _ = diesel::insert_into(crate::schema::documents::table)
                .values(&new_doc)
                .execute(&mut conn)
            .map_err(|e| format!("Failed to create document: {}", e))?;
        }

        let progress = ((i + 1) as f32 / total as f32) * 100.0;
        update_progress(db_pool, task_id, progress, Some(format!("Processed {}/{}", i + 1, total))).await?;
    }

    Ok(())
}

async fn update_progress(db_pool: &Arc<DbPool>, task_id: i64, progress: f32, message: Option<String>) -> Result<(), String> {
    let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
    diesel::update(crate::schema::tasks::table.filter(crate::schema::tasks::id.eq(task_id)))
        .set((
            crate::schema::tasks::progress.eq(progress),
            crate::schema::tasks::message.eq(message),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update progress: {}", e))?;
    Ok(())
}

fn get_collection_name(milvus_config: &MilvusConfig, project_id: Option<i64>) -> String {
    if milvus_config.database == "default" {
        match project_id {
            Some(id) => format!("project_{}", id),
            None => "default".to_string(),
        }
    } else {
        match project_id {
            Some(id) => format!("{}_project_{}", milvus_config.database, id),
            None => format!("{}_default", milvus_config.database),
        }
    }
}

async fn delete_vectors_by_chunk_ids(
    milvus_client: &MilvusClient,
    milvus_config: &MilvusConfig,
    project_id: Option<i64>,
    chunk_ids: &[i64],
) -> Result<(), String> {
    if chunk_ids.is_empty() {
        return Ok(());
    }
    let collection_name = get_collection_name(milvus_config, project_id);
    let ids_str = chunk_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let filter = format!("chunk_id in [{}]", ids_str);
    milvus_client
        .delete_by_filter(&collection_name, &filter)
        .await
        .map_err(|e| format!("Delete vectors error: {}", e))
}

async fn chunk_text(content: &str, project_id: Option<i64>, db_pool: &Arc<DbPool>) -> Result<Vec<crate::app_state::TextChunk>, String> {
    let (chunk_size, overlap) = if let Some(pid) = project_id {
        let mut conn = db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        match crate::schema::project_rag_configs::table
            .filter(crate::schema::project_rag_configs::project_id.eq(pid))
            .first::<crate::model::ProjectRagConfig>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get config: {}", e))?
        {
            Some(config) => (config.chunk_size as usize, config.chunk_overlap as usize),
            None => (512, 50),
        }
    } else {
        (512, 50)
    };

    let mut chunks = Vec::new();
    let bytes = content.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        let end = std::cmp::min(pos + chunk_size, bytes.len());
        let actual_end = if end < bytes.len() {
            bytes[pos..end]
                .iter()
                .rposition(|&b| b == b' ' || b == b'\n' || b == b'\t')
                .map(|p| pos + p + 1)
                .unwrap_or(end)
        } else {
            end
        };

        if actual_end <= pos {
            break;
        }

        let chunk_content = String::from_utf8_lossy(&bytes[pos..actual_end]).to_string();
        if !chunk_content.trim().is_empty() {
            chunks.push(crate::app_state::TextChunk {
                content: chunk_content,
                start_pos: pos,
                end_pos: actual_end,
            });
        }

        pos = if actual_end > pos + overlap {
            actual_end - overlap
        } else {
            actual_end
        };
    }

    Ok(chunks)
}
