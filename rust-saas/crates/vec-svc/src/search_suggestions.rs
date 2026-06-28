//! Search suggestions service
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, TextExpressionMethods, RunQueryDsl};

use crate::rdb_repository::{DbPool};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub weight: f64,
    pub source_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuggestionResult {
    pub suggestions: Vec<SearchSuggestion>,
    pub categories: Vec<String>,
    pub entities: Vec<String>,
}

pub struct SearchSuggestionService {
    db_pool: Arc<DbPool>,
    cache: std::sync::RwLock<HashMap<String, Vec<SearchSuggestion>>>,
}

impl SearchSuggestionService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self {
            db_pool,
            cache: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_suggestions(&self, query: &str, project_id: Option<i64>, limit: usize) -> Result<SuggestionResult, String> {
        let lower_query = query.to_lowercase();
        
        let mut suggestions = Vec::new();
        let mut categories = HashSet::new();
        let mut entities = HashSet::new();

        suggestions.extend(self.suggest_from_documents(&lower_query, project_id).await?);
        suggestions.extend(self.suggest_from_knowledge_points(&lower_query, project_id).await?);
        suggestions.extend(self.suggest_from_entities(&lower_query, project_id).await?);
        suggestions.extend(self.suggest_from_categories(&lower_query, project_id).await?);

        for s in &suggestions {
            if s.source_type == "category" {
                categories.insert(s.text.clone());
            } else if s.source_type == "entity" {
                entities.insert(s.text.clone());
            }
        }

        suggestions.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit);

        Ok(SuggestionResult {
            suggestions,
            categories: categories.into_iter().take(5).collect(),
            entities: entities.into_iter().take(5).collect(),
        })
    }

    async fn suggest_from_documents(&self, query: &str, project_id: Option<i64>) -> Result<Vec<SearchSuggestion>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let results = crate::schema::documents::table
            .filter(crate::schema::documents::topic.like(format!("%{}%", query)))
            .filter(crate::schema::documents::project_id.eq(project_id))
            .limit(10)
            .load::<crate::model::Document>(&mut conn)
            .map_err(|e| format!("Failed to search documents: {}", e))?;

        Ok(results
            .into_iter()
            .filter_map(|d| d.topic)
            .map(|t| SearchSuggestion {
                text: t,
                weight: 0.8,
                source_type: "document".to_string(),
            })
            .collect())
    }

    async fn suggest_from_knowledge_points(&self, query: &str, project_id: Option<i64>) -> Result<Vec<SearchSuggestion>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let results = crate::schema::knowledge_points::table
            .filter(crate::schema::knowledge_points::point_content.like(format!("%{}%", query)))
            .limit(10)
            .load::<crate::model::KnowledgePoint>(&mut conn)
            .map_err(|e| format!("Failed to search knowledge points: {}", e))?;

        Ok(results
            .into_iter()
            .filter_map(|k| k.point_content)
            .map(|c| {
                let snippet = if c.len() > 50 {
                    format!("{}...", &c[..50])
                } else {
                    c
                };
                SearchSuggestion {
                    text: snippet,
                    weight: 0.7,
                    source_type: "knowledge_point".to_string(),
                }
            })
            .collect())
    }

    async fn suggest_from_entities(&self, query: &str, project_id: Option<i64>) -> Result<Vec<SearchSuggestion>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let results = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.like(format!("%{}%", query)))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .limit(10)
            .load::<crate::model::KnowledgeEntity>(&mut conn)
            .map_err(|e| format!("Failed to search entities: {}", e))?;

        Ok(results
            .into_iter()
            .map(|e| SearchSuggestion {
                text: e.name.unwrap_or_default(),
                weight: 0.9,
                source_type: "entity".to_string(),
            })
            .collect())
    }

    async fn suggest_from_categories(&self, query: &str, project_id: Option<i64>) -> Result<Vec<SearchSuggestion>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let results = crate::schema::document_categories::table
            .filter(crate::schema::document_categories::category_name.like(format!("%{}%", query)))
            .limit(5)
            .load::<crate::model::DocumentCategory>(&mut conn)
            .map_err(|e| format!("Failed to search categories: {}", e))?;

        Ok(results
            .into_iter()
            .filter_map(|c| c.category_name.map(|name| SearchSuggestion {
                text: name,
                weight: 0.75,
                source_type: "category".to_string(),
            }))
            .collect())
    }

    pub async fn autocomplete(&self, query: &str, project_id: Option<i64>, limit: usize) -> Result<Vec<String>, String> {
        let result = self.get_suggestions(query, project_id, limit).await?;
        Ok(result.suggestions.into_iter().map(|s| s.text).collect())
    }
}
