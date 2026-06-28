//! Application State
//! 
//! ?
use std::sync::Arc;
use std::collections::HashMap;

use crate::embedding::EmbeddingService;
use crate::milvus::{MilvusClient, MilvusConfig, SearchResult};
use crate::minio::{MinioService, MinioConfig};
use crate::task_queue::TaskQueue;
use crate::knowledge_graph::KnowledgeGraphService;
use crate::verification::VerificationService;
use crate::search_suggestions::SearchSuggestionService;
use crate::cache::SearchCache;
use crate::rerank::RerankService;
use crate::analytics::AnalyticsService;
use crate::version_control::VersionService;
use crate::import_export::ImportExportService;
use crate::semantic_chunk::SemanticChunker;
use crate::rdb_repository::{Repositories, DbPool, create_pool, RepositoryError};
use crate::model::{Document, NewDocument, DocumentChunk, NewDocumentChunk, ProjectRagConfig, KnowledgePoint, NewKnowledgePoint, DocumentCategory, NewDocumentCategory, DocumentLevel, NewDocumentLevel, DocumentCategoryMapping, NewDocumentCategoryMapping, DocumentLevelMapping, NewDocumentLevelMapping, DocumentBoundary, NewDocumentBoundary, DocumentShare, NewDocumentShare};
use crate::distillation::DistillationService;

/// Enhanced search result with additional metadata
#[derive(Debug, Clone)]
pub struct EnhancedSearchResult {
    pub id: i64,
    pub score: f32,
    pub content: String,
    pub chunk_id: Option<i64>,
    pub document_id: Option<i64>,
    pub document_topic: Option<String>,
    pub chunk_index: Option<i32>,
    pub created_at: Option<String>,
}

/// Application state containing all services
pub struct AppState {
    pub embedding_service: EmbeddingService,
    pub milvus_client: MilvusClient,
    pub milvus_config: MilvusConfig,
    pub minio_service: MinioService,
    pub minio_config: MinioConfig,
    pub task_queue: TaskQueue,
    pub knowledge_graph_service: KnowledgeGraphService,
    pub verification_service: VerificationService,
    pub search_suggestion_service: SearchSuggestionService,
    pub search_cache: SearchCache,
    pub rerank_service: RerankService,
    pub analytics_service: AnalyticsService,
    pub version_service: VersionService,
    pub import_export_service: ImportExportService,
    pub semantic_chunker: SemanticChunker,
    pub collections: std::sync::RwLock<HashMap<i64, String>>,
    pub repos: Repositories,
    pub db_pool: Arc<DbPool>,
    pub distillation_service: DistillationService,
}

impl AppState {
    pub async fn new() -> Result<Self, String> {
        // 1. Load Embedding service
        let model_path = std::env::var("EMBEDDING_MODEL_PATH")
            .unwrap_or_else(|_| "/models/gemma-4-E4B-it-Q4_0.gguf".to_string());
        let embedding_dim: usize = std::env::var("EMBEDDING_DIM")
            .unwrap_or_else(|_| "2560".to_string())
            .parse()
            .unwrap_or(2560);

        tracing::info!("Loading embedding model from: {}", model_path);

        let embedding_service = EmbeddingService::load(&model_path, embedding_dim)
            .map_err(|e| format!("Failed to load embedding model: {}", e))?;

        tracing::info!("Embedding model loaded successfully. Dim={}", embedding_dim);

        // 2. Initialize Milvus client
        let milvus_config = MilvusConfig::from_env();
        let milvus_client = MilvusClient::new(milvus_config.clone());

        tracing::info!("Milvus client initialized: {}", milvus_config.url());

        // 3. Initialize PostgreSQL connection pool
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/vec_svc".to_string());

        tracing::info!("Initializing database connection pool...");

        let db_pool = create_pool(&database_url)
            .map_err(|e| format!("Failed to create database pool: {}", e))?;

        let repos = Repositories::new(db_pool.clone());

        tracing::info!("Database connection pool initialized successfully");

        // 4. Initialize distillation service (reuse embedding tokenizer)
        let tokenizer = embedding_service.tokenizer().clone();
        let distillation_service = DistillationService::new(tokenizer.clone());

        tracing::info!("Distillation service initialized");

        // 5. Initialize MinIO service
        let minio_config = MinioConfig::from_env();
        let minio_service = MinioService::new(minio_config.clone())
            .map_err(|e| format!("Failed to initialize MinIO service: {}", e))?;

        tracing::info!("MinIO service initialized: {}", minio_config.endpoint);

        // 6. Initialize knowledge graph service with semantic extractor
        let knowledge_graph_service = KnowledgeGraphService::with_extractor(
            db_pool.clone(),
            Arc::new(embedding_service.clone()),
        );

        tracing::info!("Knowledge graph service initialized");

        // 7. Initialize task queue
        let task_queue = TaskQueue::new(
            db_pool.clone(),
            embedding_service.clone(),
            distillation_service.clone(),
            milvus_client.clone(),
            milvus_config.clone(),
            knowledge_graph_service.clone(),
        );
        task_queue.start_workers(2).await;

        tracing::info!("Task queue initialized with 2 workers");

        // 8. Initialize verification service
        let verification_service = VerificationService::new(db_pool.clone());

        tracing::info!("Verification service initialized");

        // 9. Initialize search suggestion service
        let search_suggestion_service = SearchSuggestionService::new(db_pool.clone());

        tracing::info!("Search suggestion service initialized");

        // 10. Initialize search cache
        let search_cache = SearchCache::new();

        tracing::info!("Search cache initialized");

        // 11. Initialize rerank service
        let rerank_service = RerankService::new();

        tracing::info!("Rerank service initialized");

        // 12. Initialize analytics service
        let analytics_service = AnalyticsService::new(db_pool.clone());

        tracing::info!("Analytics service initialized");

        // 13. Initialize version service
        let version_service = VersionService::new(db_pool.clone());

        tracing::info!("Version service initialized");

        // 14. Initialize import/export service
        let import_export_service = ImportExportService::new(db_pool.clone());

        tracing::info!("Import/export service initialized");

        // 15. Initialize semantic chunker
        let semantic_chunker = SemanticChunker::new(Arc::new(tokenizer.clone()));

        tracing::info!("Semantic chunker initialized");

        Ok(Self {
            embedding_service,
            milvus_client,
            milvus_config,
            minio_service,
            minio_config,
            task_queue,
            knowledge_graph_service,
            verification_service,
            search_suggestion_service,
            search_cache,
            rerank_service,
            analytics_service,
            version_service,
            import_export_service,
            semantic_chunker,
            collections: std::sync::RwLock::new(HashMap::new()),
            repos,
            db_pool,
            distillation_service,
        })
    }

    /// Get collection name for a project
    pub fn get_collection_name(&self, project_id: Option<i64>) -> String {
        if self.milvus_config.database == "default" {
            match project_id {
                Some(id) => format!("project_{}", id),
                None => "default".to_string(),
            }
        } else {
            match project_id {
                Some(id) => format!("{}_project_{}", self.milvus_config.database, id),
                None => format!("{}_default", self.milvus_config.database),
            }
        }
    }

    /// Ensure collection exists in Milvus
    pub async fn ensure_collection(&self, collection_name: &str) -> Result<(), String> {
        let embedding_dim = self.embedding_service.config().embedding_dim;

        self.milvus_client
            .create_collection(collection_name, embedding_dim)
            .await
            .map_err(|e| format!("Failed to create collection: {}", e))?;

        Ok(())
    }

    /// Embed text asynchronously
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
        let text = text.to_string();
        let embedding_service = self.embedding_service.clone();
        tokio::task::spawn_blocking(move || {
            embedding_service
                .embed(&text)
                .map_err(|e| format!("Embedding error: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Embed batch of texts asynchronously
    pub async fn embed_text_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let texts = texts.to_vec();
        let embedding_service = self.embedding_service.clone();
        tokio::task::spawn_blocking(move || {
            let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            embedding_service
                .embed_batch(&text_refs)
                .map_err(|e| format!("Embedding error: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn vector_search(
        &self,
        query: &str,
        top_k: usize,
        project_id: Option<i64>,
    ) -> Result<Vec<SearchResult>, String> {
        // 1. 向量化查询
        let query_vector = self.embed_text(query).await?;

        // 2. 获取集合名称
        let collection_name = self.get_collection_name(project_id);

        // 3. 搜索
        self.milvus_client
            .search(&collection_name, query_vector, top_k, project_id)
            .await
            .map_err(|e| format!("Search error: {}", e))
    }

    /// 
    pub async fn search_with_document_info(
        &self,
        query: &str,
        top_k: usize,
        project_id: Option<i64>,
    ) -> Result<Vec<EnhancedSearchResult>, String> {
        // 1. 向量搜索
        let results = self.vector_search(query, top_k, project_id).await?;

        if results.is_empty() {
            return Ok(vec![]);
        }

        // 2. 提取 chunk_id 列表
        let chunk_ids: Vec<i64> = results.iter()
            .filter_map(|r| r.chunk_id)
            .collect();

        if chunk_ids.is_empty() {
            //  chunk_id，直接返回基础结果
            return Ok(results.into_iter().map(|r| EnhancedSearchResult {
                id: r.id,
                score: r.score,
                content: r.document,
                chunk_id: r.chunk_id,
                document_id: None,
                document_topic: None,
                chunk_index: None,
                created_at: None,
            }).collect());
        }

        // 3. 批量查询分块信息（含文档信息）      
        let repos = self.repos.clone();
        let chunks_with_docs = tokio::task::spawn_blocking(move || {
            repos.chunks.get_by_ids_with_document(&chunk_ids)
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
        .map_err(|e| format!("Query chunks error: {}", e))?;

        // 4. 组装结果
        let mut enhanced_results = Vec::with_capacity(results.len());
        for result in results {
            let chunk_info = chunks_with_docs.iter()
                .find(|(c, _)| Some(c.id) == result.chunk_id);
            
            match chunk_info {
                Some((chunk, doc)) => {
                    enhanced_results.push(EnhancedSearchResult {
                        id: result.id,
                        score: result.score,
                        content: result.document.clone(),
                        chunk_id: Some(chunk.id),
                        document_id: Some(doc.id),
                        document_topic: doc.topic.clone(),
                        chunk_index: Some(chunk.chunk_index),
                        created_at: Some(chunk.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
                    });
                }
                None => {
                    enhanced_results.push(EnhancedSearchResult {
                        id: result.id,
                        score: result.score,
                        content: result.document.clone(),
                        chunk_id: result.chunk_id,
                        document_id: None,
                        document_topic: None,
                        chunk_index: None,
                        created_at: None,
                    });
                }
            }
        }

        Ok(enhanced_results)
    }

    /// 
    pub async fn insert_vectors(
        &self,
        project_id: Option<i64>,
        vectors: Vec<Vec<f32>>,
        documents: Vec<String>,
        chunk_ids: Vec<i64>,
    ) -> Result<Vec<i64>, String> {
        let collection_name = self.get_collection_name(project_id);

        //
        self.ensure_collection(&collection_name).await?;

        let project_ids: Vec<Option<i64>> = vec![project_id; vectors.len()];

        //
        self.milvus_client
            .insert(&collection_name, vectors, documents, chunk_ids, project_ids)
            .await
            .map_err(|e| format!("Insert error: {}", e))
    }

    pub async fn delete_vectors_by_chunk_ids(
        &self,
        project_id: Option<i64>,
        chunk_ids: &[i64],
    ) -> Result<(), String> {
        if chunk_ids.is_empty() {
            return Ok(());
        }
        let collection_name = self.get_collection_name(project_id);
        let ids_str = chunk_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        let filter = format!("chunk_id in [{}]", ids_str);
        self.milvus_client
            .delete_by_filter(&collection_name, &filter)
            .await
            .map_err(|e| format!("Delete vectors error: {}", e))
    }

    /// Check if embedding is loaded
    pub fn is_embedding_loaded(&self) -> bool {
        true
    }

    /// Check Milvus connection
    pub async fn check_milvus_connection(&self) -> bool {
        let url = format!("{}/v2/vectordb/collections/list", self.milvus_config.url());

        match reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({"dbName": self.milvus_config.database}))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// RAG configuration
#[derive(Debug, Clone)]
pub struct RagConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub min_score: f32,
}

impl From<ProjectRagConfig> for RagConfig {
    fn from(config: ProjectRagConfig) -> Self {
        Self {
            chunk_size: config.chunk_size as usize,
            chunk_overlap: config.chunk_overlap as usize,
            top_k: config.top_k as usize,
            min_score: config.min_score as f32,
        }
    }
}

impl AppState {
    /// Get RAG config from database
    pub async fn get_rag_config(&self, project_id: i64) -> Result<Option<RagConfig>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.rag_configs
                .get_by_project_id(project_id)
                .map(|c| Some(RagConfig::from(c)))
                .map_err(|e| format!("Failed to get rag config: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Chunk text into segments
    pub async fn chunk_text(&self, text: &str, project_id: Option<i64>) -> Result<Vec<TextChunk>, String> {
        let (chunk_size, overlap) = if let Some(pid) = project_id {
            match self.get_rag_config(pid).await? {
                Some(config) => (config.chunk_size, config.chunk_overlap),
                None => (512, 50),
            }
        } else {
            (512, 50)
        };

        let mut chunks = Vec::new();
        let bytes = text.as_bytes();
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
                chunks.push(TextChunk {
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

    /// + 分块 + 向量�?+ 存储
    pub async fn process_document(
        &self,
        topic: Option<&str>,
        content: &str,
        project_id: Option<i64>,
        source_type: Option<&str>,
        source_url: Option<&str>,
        file_path: Option<&str>,
        file_type: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<(Document, Vec<DocumentChunk>), String> {
        let content_owned = content.to_string();
        let topic_owned = topic.map(|s| s.to_string());
        let source_type_owned = source_type.map(|s| s.to_string());
        let source_url_owned = source_url.map(|s| s.to_string());
        let file_path_owned = file_path.map(|s| s.to_string());
        let file_type_owned = file_type.map(|s| s.to_string());
        let metadata_owned = metadata.cloned();

        // 1. 分块
        let chunks = self.chunk_text(content, project_id).await?;
        let chunk_count = chunks.len() as i32;

        // 2. 创建文档记录
        let new_doc = NewDocument {
            project_id,
            title: topic_owned.clone(),
            topic: topic_owned.clone(),
            content: Some(content_owned.clone()),
            content_hash: None,
            source_type: source_type_owned.clone(),
            source_url: source_url_owned.clone(),
            file_path: file_path_owned.clone(),
            file_type: file_type_owned.clone(),
            status: Some("pending".to_string()),
            visibility: Some("private".to_string()),
            boundary_level: None,
            token_count: None,
            version: 1,
            word_count: None,
            chunk_count,
            metadata: metadata_owned.clone(),
        };

        let repos = self.repos.clone();
        let doc = tokio::task::spawn_blocking(move || {
            repos.documents
                .create(&new_doc)
                .map_err(|e| format!("Failed to create document: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let doc_id = doc.id;

        // 3. 先写入 PostgreSQL 分块以获取 chunk_id
        let new_chunks: Vec<NewDocumentChunk> = chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| NewDocumentChunk {
                document_id: doc_id,
                chunk_index: i as i32,
                chunk_text: Some(chunk.content.clone()),
                embedding_status: Some("pending".to_string()),
            })
            .collect();

        let repos2 = self.repos.clone();
        let saved_chunks = tokio::task::spawn_blocking(move || {
            repos2.chunks
                .batch_insert(&new_chunks)
                .map_err(|e| format!("Failed to insert chunks: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let chunk_ids: Vec<i64> = saved_chunks.iter().map(|c| c.id).collect();

        // 4. 向量化所有块
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let vectors = self.embed_text_batch(&texts).await?;

        // 5. 写入 Milvus
        let _vector_ids = self.insert_vectors(project_id, vectors, texts, chunk_ids).await?;

        // Update document indexed_at
        let repos3 = self.repos.clone();
        let doc_id_for_update = doc_id;
        tokio::task::spawn_blocking(move || {
            repos3.documents
                .mark_indexed(doc_id_for_update, chunk_count)
                .map_err(|e| format!("Failed to mark indexed: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        // 7. 重新获取更新后的文档
        let repos4 = self.repos.clone();
        let final_doc = tokio::task::spawn_blocking(move || {
            repos4.documents
                .get_by_id(doc_id)
                .map_err(|e| format!("Failed to get document: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        Ok((final_doc, saved_chunks))
    }

    /// 
    pub async fn get_document(&self, id: i64) -> Result<Document, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.documents
                .get_by_id(id)
                .map_err(|e| format!("Failed to get document: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Delete document
    pub async fn delete_document(&self, id: i64) -> Result<(), String> {
        let doc = self.get_document(id).await?;
        let project_id = doc.project_id;

        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.chunks
                .delete_by_document_id(id)
                .map_err(|e| format!("Failed to delete chunks: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let repos2 = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos2.documents
                .delete(id)
                .map_err(|e| format!("Failed to delete document: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        Ok(())
    }

    /// Get chunk count
    pub async fn get_chunk_count(&self, id: i64) -> Result<usize, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.chunks
                .count_by_document(id)
                .map_err(|e| format!("Failed to count chunks: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn list_documents(
        &self,
        project_id: Option<i64>,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<Document>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.documents
                .list_by_project(project_id, page, page_size)
                .map_err(|e| format!("Failed to list documents: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn count_documents(&self, project_id: Option<i64>) -> Result<usize, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.documents
                .count(project_id)
                .map_err(|e| format!("Failed to count documents: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn reindex_document(&self, id: i64) -> Result<usize, String> {
        // 1. 获取文档
        let doc = self.get_document(id).await?;
        let content = doc.content.clone().unwrap_or_default();
        let project_id = doc.project_id;

        // 2. 删除旧分�?        
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.chunks
                .delete_by_document_id(id)
                .map_err(|e| format!("Failed to delete old chunks: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        // 3. 重新分块
        let chunks = self.chunk_text(&content, project_id).await?;
        let chunk_count = chunks.len() as i32;

        let new_chunks: Vec<NewDocumentChunk> = chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| NewDocumentChunk {
                document_id: id,
                chunk_index: i as i32,
                chunk_text: Some(chunk.content.clone()),
                embedding_status: Some("pending".to_string()),
            })
            .collect();

        // 先插入 PostgreSQL 分块获取 chunk_id，再写入 Milvus
        let repos2 = self.repos.clone();
        let saved_chunks = tokio::task::spawn_blocking(move || {
            repos2.chunks
                .batch_insert(&new_chunks)
                .map_err(|e| format!("Failed to insert new chunks: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let chunk_ids: Vec<i64> = saved_chunks.iter().map(|c| c.id).collect();

        // 4. 向量
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let vectors = self.embed_text_batch(&texts).await?;

        // 5. 写入 Milvus
        self.insert_vectors(project_id, vectors, texts, chunk_ids).await?;

        let repos3 = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos3.documents
                .mark_indexed(id, chunk_count)
                .map_err(|e| format!("Failed to mark indexed: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        Ok(chunk_count as usize)
    }

    // ============ 知识蒸馏 ============

    /// Distill document
    pub async fn distill_document(&self, document_id: i64) -> Result<Vec<KnowledgePoint>, String> {
        let doc = self.get_document(document_id).await?;
        let content = doc.content.clone().unwrap_or_default();

        if content.is_empty() {
            return Err("Document content is empty".to_string());
        }

        let distillation = self.distillation_service.clone();
        let content_clone = content.clone();
        let new_points = tokio::task::spawn_blocking(move || {
            distillation.distill(&content_clone, document_id)
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let repos1 = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos1.knowledge_points
                .delete_by_document_id(document_id)
                .map_err(|e| format!("Failed to delete old points: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        let repos2 = self.repos.clone();
        let points = tokio::task::spawn_blocking(move || {
            repos2.knowledge_points
                .batch_create(&new_points)
                .map_err(|e| format!("Failed to create points: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))??;

        Ok(points)
    }

    /// 
    pub async fn get_knowledge_points(&self, document_id: i64, point_type: Option<&str>) -> Result<Vec<KnowledgePoint>, String> {
        let repos = self.repos.clone();
        let point_type_owned = point_type.map(|s| s.to_string());
        
        tokio::task::spawn_blocking(move || {
            match point_type_owned {
                Some(ptype) => repos.knowledge_points
                    .get_by_document_and_type(document_id, &ptype)
                    .map_err(|e| format!("Failed to get points: {}", e)),
                None => repos.knowledge_points
                    .get_by_document_id(document_id)
                    .map_err(|e| format!("Failed to get points: {}", e)),
            }
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Preview distillation result
    pub async fn preview_distillation(&self, content: &str) -> Result<crate::distillation::DistillationResult, String> {
        let content = content.to_string();
        let distillation = self.distillation_service.clone();
        
        tokio::task::spawn_blocking(move || {
            distillation.extract_all(&content)
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Delete knowledge point
    pub async fn delete_knowledge_point(&self, id: i64) -> Result<(), String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.knowledge_points
                .delete(id)
                .map_err(|e| format!("Failed to delete point: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    // ============ 分类分级 ============

    /// 
    pub async fn create_category(&self, category: NewDocumentCategory) -> Result<DocumentCategory, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .create(&category)
                .map_err(|e| format!("Failed to create category: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn get_category(&self, id: i64) -> Result<DocumentCategory, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .get_by_id(id)
                .map_err(|e| format!("Failed to get category: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn list_root_categories(&self, project_id: i64) -> Result<Vec<DocumentCategory>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .list_root_categories(project_id)
                .map_err(|e| format!("Failed to list root categories: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// List child categories
    pub async fn list_child_categories(&self, parent_id: i64) -> Result<Vec<DocumentCategory>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .list_children(parent_id)
                .map_err(|e| format!("Failed to list child categories: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn update_category(&self, id: i64, category: NewDocumentCategory) -> Result<DocumentCategory, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .update(id, &category)
                .map_err(|e| format!("Failed to update category: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Delete category
    pub async fn delete_category(&self, id: i64) -> Result<(), String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .delete(id)
                .map_err(|e| format!("Failed to delete category: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Assign categories to document
    pub async fn assign_document_categories(&self, document_id: i64, mappings: Vec<NewDocumentCategoryMapping>) -> Result<Vec<DocumentCategoryMapping>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .assign_categories(document_id, &mappings)
                .map_err(|e| format!("Failed to assign categories: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get document categories
    pub async fn get_document_categories(&self, document_id: i64) -> Result<Vec<DocumentCategory>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.categories
                .get_document_categories(document_id)
                .map_err(|e| format!("Failed to get document categories: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Create level
    pub async fn create_level(&self, level: NewDocumentLevel) -> Result<DocumentLevel, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .create(&level)
                .map_err(|e| format!("Failed to create level: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get level
    pub async fn get_level(&self, id: i64) -> Result<DocumentLevel, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .get_by_id(id)
                .map_err(|e| format!("Failed to get level: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// List levels
    pub async fn list_levels(&self, project_id: i64) -> Result<Vec<DocumentLevel>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .list_by_project(project_id)
                .map_err(|e| format!("Failed to list levels: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Update level
    pub async fn update_level(&self, id: i64, level: NewDocumentLevel) -> Result<DocumentLevel, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .update(id, &level)
                .map_err(|e| format!("Failed to update level: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Delete level
    pub async fn delete_level(&self, id: i64) -> Result<(), String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .delete(id)
                .map_err(|e| format!("Failed to delete level: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn assign_document_levels(&self, document_id: i64, mappings: Vec<NewDocumentLevelMapping>) -> Result<Vec<DocumentLevelMapping>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .assign_levels(document_id, &mappings)
                .map_err(|e| format!("Failed to assign levels: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get document levels
    pub async fn get_document_levels(&self, document_id: i64) -> Result<Vec<DocumentLevel>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.levels
                .get_document_levels(document_id)
                .map_err(|e| format!("Failed to get document levels: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    // ============ Knowledge Boundary ============

    /// Set document visibility
    pub async fn set_document_visibility(&self, document_id: i64, visibility: &str, owner_id: Option<i64>, project_id: Option<i64>, team_id: Option<i64>) -> Result<DocumentBoundary, String> {
        let repos = self.repos.clone();
        let visibility = visibility.to_string();
        tokio::task::spawn_blocking(move || {
            repos.boundaries
                .set_visibility(document_id, &visibility, owner_id, project_id, team_id)
                .map_err(|e| format!("Failed to set visibility: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get document boundary
    pub async fn get_document_boundary(&self, document_id: i64) -> Result<Option<DocumentBoundary>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.boundaries
                .get_by_document_id(document_id)
                .map(Some)
                .or_else(|e| match e {
                    RepositoryError::NotFound(_) => Ok(None),
                    other => Err(format!("Failed to get boundary: {}", other)),
                })
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Check document access
    pub async fn check_document_access(&self, document_id: i64, user_id: i64, user_projects: Vec<i64>, user_teams: Vec<i64>) -> Result<bool, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.boundaries
                .check_access(document_id, user_id, &user_projects, &user_teams)
                .map_err(|e| format!("Failed to check access: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get accessible document IDs
    pub async fn get_accessible_document_ids(&self, user_id: i64, user_projects: Vec<i64>, user_teams: Vec<i64>) -> Result<Vec<i64>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.boundaries
                .get_accessible_document_ids(user_id, &user_projects, &user_teams)
                .map_err(|e| format!("Failed to get accessible documents: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    // ============ Knowledge Sharing ============

    /// Create document share
    pub async fn create_document_share(&self, share: NewDocumentShare) -> Result<DocumentShare, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.shares
                .create(&share)
                .map_err(|e| format!("Failed to create share: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get document shares
    pub async fn get_document_shares(&self, document_id: i64) -> Result<Vec<DocumentShare>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.shares
                .get_by_document_id(document_id)
                .map_err(|e| format!("Failed to get shares: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn delete_document_share(&self, id: i64) -> Result<(), String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.shares
                .delete(id)
                .map_err(|e| format!("Failed to delete share: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn batch_create_shares(&self, shares: Vec<NewDocumentShare>) -> Result<Vec<DocumentShare>, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.shares
                .batch_create(&shares)
                .map_err(|e| format!("Failed to batch create shares: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// 
    pub async fn check_share_access(&self, document_id: i64, user_id: i64) -> Result<bool, String> {
        let repos = self.repos.clone();
        tokio::task::spawn_blocking(move || {
            repos.shares
                .check_share_access(document_id, user_id)
                .map_err(|e| format!("Failed to check share access: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }
}

impl From<RepositoryError> for String {
    fn from(e: RepositoryError) -> Self {
        e.to_string()
    }
}
