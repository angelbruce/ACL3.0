
use axum::{
    extract::{State, Query, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use shared::errors::{ServiceError, ServiceResult};

/// 
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// 
    pub query: String,
    ///  ID（可选，用于项目内搜索）
    pub project_id: Option<i64>,
    /// 
    pub top_k: Option<usize>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: i64,
    pub score: f32,
    pub content: String,
    pub chunk_id: Option<i64>,
    pub document_id: Option<i64>,
    pub document_topic: Option<String>,
    pub chunk_index: Option<i32>,
    pub created_at: Option<String>,
    /// 高亮信息：句子及其相似度分数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<Vec<SentenceHighlight>>,
}

/// 句子高亮信息
#[derive(Debug, Serialize, Clone)]
pub struct SentenceHighlight {
    pub text: String,
    pub score: f32,
}

/// 将文本分割成句子
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    
    for ch in text.chars() {
        current.push(ch);
        // 中文标点：。！？；
        // 英文标点：. ! ? ;
        if matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    
    // 处理最后没有标点的部分
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    
    sentences
}

/// 计算句子级别的语义高亮
async fn compute_sentence_highlights(
    state: &Arc<AppState>,
    query: &str,
    content: &str,
    top_n: usize,
) -> Option<Vec<SentenceHighlight>> {
    let sentences = split_into_sentences(content);
    if sentences.is_empty() {
        return None;
    }
    
    // 批量嵌入所有句子
    let sentence_refs: Vec<&str> = sentences.iter().map(|s| s.as_str()).collect();
    let sentence_embeddings = match state.embedding_service.embed_batch(&sentence_refs) {
        Ok(embeddings) => embeddings,
        Err(e) => {
            tracing::warn!("Failed to embed sentences for highlighting: {}", e);
            return None;
        }
    };
    
    // 嵌入查询
    let query_embedding = match state.embedding_service.embed(query) {
        Ok(embedding) => embedding,
        Err(e) => {
            tracing::warn!("Failed to embed query for highlighting: {}", e);
            return None;
        }
    };
    
    // 计算每个句子与查询的相似度
    let mut sentence_scores: Vec<(String, f32)> = sentences
        .into_iter()
        .zip(sentence_embeddings.iter())
        .map(|(text, emb)| {
            let score = cosine_similarity(&query_embedding, emb);
            (text, score)
        })
        .collect();
    
    // 按相似度排序，取前 top_n 个
    sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sentence_scores.truncate(top_n);
    
    // 按原始顺序重新排序（通过匹配文本）
    let original_order: Vec<SentenceHighlight> = content
        .split(|c| matches!(c, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';'))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .filter_map(|text| {
            sentence_scores
                .iter()
                .find(|(t, _)| t == &text)
                .map(|(_, score)| SentenceHighlight { text, score: *score })
        })
        .collect();
    
    if original_order.is_empty() {
        None
    } else {
        Some(original_order)
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

/// 
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub query: String,
    pub total: usize,
}

/// /// GET /api/projects/{project_id}/search?query=xxx&top_k=5
pub async fn search_by_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i64>,
    Query(params): Query<SearchRequest>,
) -> ServiceResult<Json<SearchResponse>> {
    let top_k = params.top_k.unwrap_or(5);
    
    tracing::info!("Searching in project {}: query='{}', top_k={}", project_id, params.query, top_k);
    
    // 1. 获取项目 RAG 配置
    let rag_config = state.get_rag_config(project_id).await?;
    let effective_top_k = rag_config.as_ref().map(|c| c.top_k).unwrap_or(top_k);
    
    // 2. 增强搜索（带文档元信息）
    let results = state.search_with_document_info(&params.query, effective_top_k, Some(project_id)).await?;
    
    // 3. 语义搜索结果（全部保留）
    let mut filtered: Vec<_> = results.into_iter()
        .map(|r| SearchResultItem {
            id: r.id,
            score: r.score,
            content: r.content,
            chunk_id: r.chunk_id,
            document_id: r.document_id,
            document_topic: r.document_topic,
            chunk_index: r.chunk_index,
            created_at: r.created_at,
            highlights: None,
        })
        .collect();

    // 4. 关键词搜索，拼在语义结果后面（限制5条）
    let pattern = format!("%{}%", params.query);
    let repos = state.repos.clone();
    let kw_project_id = Some(project_id);
    let keyword_results = tokio::task::spawn_blocking(move || {
        repos.chunks.search_by_keyword(&pattern, kw_project_id, 5)
    })
    .await
    .map_err(|e| ServiceError::InternalError)?
    .map_err(|e| ServiceError::InternalError)?;

    let keyword_items: Vec<SearchResultItem> = keyword_results.into_iter()
        .map(|(chunk, doc)| SearchResultItem {
            id: chunk.id,
            score: 0.4,
            content: chunk.chunk_text.unwrap_or_default(),
            chunk_id: Some(chunk.id),
            document_id: Some(doc.id),
            document_topic: doc.topic,
            chunk_index: Some(chunk.chunk_index),
            created_at: Some(chunk.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
            highlights: None,
        })
        .collect();

    // 语义结果在前，关键词结果在后
    filtered.extend(keyword_items);

    // 5. 计算句子级别语义高亮（仅对前3个结果）
    for item in filtered.iter_mut().take(3) {
        item.highlights = compute_sentence_highlights(&state, &params.query, &item.content, 3).await;
    }

    let total = filtered.len();
    Ok(Json(SearchResponse {
        results: filtered,
        query: params.query,
        total,
    }))
}

///
/// POST /api/search
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> ServiceResult<Json<SearchResponse>> {
    let top_k = req.top_k.unwrap_or(5);

    tracing::info!("General search: query='{}', top_k={}", req.query, top_k);

    // 1. 增强搜索（带文档元信息）
    let results = state.search_with_document_info(&req.query, top_k, req.project_id).await?;

    // 2. 语义搜索结果（全部保留）
    let mut filtered: Vec<_> = results.into_iter()
        .map(|r| SearchResultItem {
            id: r.id,
            score: r.score,
            content: r.content,
            chunk_id: r.chunk_id,
            document_id: r.document_id,
            document_topic: r.document_topic,
            chunk_index: r.chunk_index,
            created_at: r.created_at,
            highlights: None,
        })
        .collect();

    // 3. 关键词搜索，拼在语义结果后面（限制5条）
    let pattern = format!("%{}%", req.query);
    let repos = state.repos.clone();
    let project_id = req.project_id;
    let keyword_results = tokio::task::spawn_blocking(move || {
        repos.chunks.search_by_keyword(&pattern, project_id, 5)
    })
    .await
    .map_err(|e| ServiceError::InternalError)?
    .map_err(|e| ServiceError::InternalError)?;

    let keyword_items: Vec<SearchResultItem> = keyword_results.into_iter()
        .map(|(chunk, doc)| SearchResultItem {
            id: chunk.id,
            score: 0.4,
            content: chunk.chunk_text.unwrap_or_default(),
            chunk_id: Some(chunk.id),
            document_id: Some(doc.id),
            document_topic: doc.topic,
            chunk_index: Some(chunk.chunk_index),
            created_at: Some(chunk.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
            highlights: None,
        })
        .collect();

    // 语义结果在前，关键词结果在后
    filtered.extend(keyword_items);

    // 4. 计算句子级别语义高亮（仅对前3个结果）
    for item in filtered.iter_mut().take(3) {
        item.highlights = compute_sentence_highlights(&state, &req.query, &item.content, 3).await;
    }

    let total = filtered.len();
    Ok(Json(SearchResponse {
        results: filtered,
        query: req.query,
        total,
    }))
}

/// 
/// GET /api/search/suggest?q=xxx
#[derive(Debug, Deserialize)]
pub struct SuggestRequest {
    pub q: String,
    pub project_id: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SuggestResponse {
    pub suggestions: Vec<String>,
    pub categories: Vec<String>,
    pub entities: Vec<String>,
}

pub async fn suggest(
    State(state): State<Arc<AppState>>,
    Query(req): Query<SuggestRequest>,
) -> ServiceResult<Json<SuggestResponse>> {
    let limit = req.limit.unwrap_or(10);
    let result = state.search_suggestion_service
        .get_suggestions(&req.q, req.project_id, limit)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(SuggestResponse {
        suggestions: result.suggestions.into_iter().map(|s| s.text).collect(),
        categories: result.categories,
        entities: result.entities,
    }))
}

pub async fn autocomplete(
    State(state): State<Arc<AppState>>,
    Query(req): Query<SuggestRequest>,
) -> ServiceResult<Json<Vec<String>>> {
    let limit = req.limit.unwrap_or(10);
    let suggestions = state.search_suggestion_service
        .autocomplete(&req.q, req.project_id, limit)
        .await
        .map_err(|e| ServiceError::InternalError)?;

    Ok(Json(suggestions))
}
