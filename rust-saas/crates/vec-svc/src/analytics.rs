//! Analytics service
//!
//! Statistics and analytics functionality

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{Utc, NaiveDateTime, Duration};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, RunQueryDsl};

use crate::rdb_repository::DbPool;
use crate::schema;

pub struct AnalyticsService {
    db_pool: Arc<DbPool>,
    hot_cache: RwLock<HotCache>,
}

struct HotCache {
    popular_documents: Vec<PopularDocument>,
    hot_entities: Vec<HotEntity>,
    search_trends: Vec<SearchTrend>,
    last_updated: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularDocument {
    pub document_id: i64,
    pub title: String,
    pub view_count: i64,
    pub search_count: i64,
    pub trend_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotEntity {
    pub entity_id: i64,
    pub entity_name: String,
    pub entity_type: String,
    pub mention_count: i64,
    pub trend_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrend {
    pub query: String,
    pub count: i64,
    pub trend: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_documents: i64,
    pub total_views: i64,
    pub total_searches: i64,
    pub total_entities: i64,
    pub active_users: i64,
    pub popular_documents: Vec<PopularDocument>,
    pub hot_entities: Vec<HotEntity>,
    pub search_trends: Vec<SearchTrend>,
}

impl AnalyticsService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self {
            db_pool,
            hot_cache: RwLock::new(HotCache {
                popular_documents: Vec::new(),
                hot_entities: Vec::new(),
                search_trends: Vec::new(),
                last_updated: Utc::now().naive_utc(),
            }),
        }
    }

    pub async fn record_document_view(&self, document_id: i64, _user_id: Option<i64>) -> Result<(), String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let now = Utc::now().naive_utc();

        diesel::insert_into(schema::access_logs::table)
            .values((
                schema::access_logs::document_id.eq(document_id),
                schema::access_logs::access_type.eq("view"),
                schema::access_logs::created_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to record view: {}", e))?;

        Ok(())
    }

    pub async fn record_search(&self, query: &str, _project_id: Option<i64>, result_count: i32) -> Result<(), String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let now = Utc::now().naive_utc();

        diesel::insert_into(schema::search_logs::table)
            .values((
                schema::search_logs::query_text.eq(query),
                schema::search_logs::result_count.eq(result_count),
                schema::search_logs::created_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to record search: {}", e))?;

        Ok(())
    }

    pub async fn get_summary(&self, project_id: Option<i64>, days: i64) -> Result<AnalyticsSummary, String> {
        let _conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let now = Utc::now().naive_utc();
        let since = now - Duration::days(days);

        let total_documents = self.count_documents(project_id).unwrap_or(0);
        let total_entities = self.count_entities(project_id).unwrap_or(0);
        let total_views = self.count_access_logs(project_id, "view", since).unwrap_or(0);
        let total_searches = self.count_search_logs(project_id, since).unwrap_or(0);

        let popular_documents = self.get_popular_documents(project_id, 10, since).unwrap_or_default();
        let hot_entities = self.get_hot_entities(project_id, 10, since).unwrap_or_default();
        let search_trends = self.get_search_trends(project_id, 10, since).unwrap_or_default();

        Ok(AnalyticsSummary {
            total_documents,
            total_views,
            total_searches,
            total_entities,
            active_users: 0,
            popular_documents,
            hot_entities,
            search_trends,
        })
    }

    fn count_documents(&self, project_id: Option<i64>) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let mut query = schema::documents::table.into_boxed();
        if let Some(pid) = project_id {
            query = query.filter(schema::documents::project_id.eq(pid));
        }

        query.count().get_result(&mut conn)
            .map_err(|e| format!("Failed to count documents: {}", e))
    }

    fn count_entities(&self, project_id: Option<i64>) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let mut query = schema::knowledge_entities::table.into_boxed();
        if let Some(pid) = project_id {
            query = query.filter(schema::knowledge_entities::project_id.eq(pid));
        }

        query.count().get_result(&mut conn)
            .map_err(|e| format!("Failed to count entities: {}", e))
    }

    fn count_access_logs(&self, project_id: Option<i64>, access_type_filter: &str, since: NaiveDateTime) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        if let Some(pid) = project_id {
            let count: i64 = schema::access_logs::table
                .inner_join(schema::documents::table.on(schema::access_logs::document_id.eq(schema::documents::id.nullable())))
                .filter(schema::access_logs::created_at.ge(since))
                .filter(schema::access_logs::access_type.eq(access_type_filter))
                .filter(schema::documents::project_id.eq(pid))
                .count()
                .get_result(&mut conn)
                .map_err(|e| format!("Failed to count access logs: {}", e))?;
            Ok(count)
        } else {
            schema::access_logs::table
                .filter(schema::access_logs::created_at.ge(since))
                .filter(schema::access_logs::access_type.eq(access_type_filter))
                .count()
                .get_result(&mut conn)
                .map_err(|e| format!("Failed to count access logs: {}", e))
        }
    }

    fn count_search_logs(&self, project_id: Option<i64>, since: NaiveDateTime) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let mut query = schema::search_logs::table.into_boxed();
        query = query.filter(schema::search_logs::created_at.ge(since));
        if let Some(pid) = project_id {
            query = query.filter(schema::search_logs::project_id.eq(pid));
        }

        query.count().get_result(&mut conn)
            .map_err(|e| format!("Failed to count search logs: {}", e))
    }

    fn get_popular_documents(&self, project_id: Option<i64>, limit: usize, since: NaiveDateTime) -> Result<Vec<PopularDocument>, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        use schema::access_logs;
        use schema::documents;
        use diesel::dsl::count_star;

        let mut query = access_logs::table
            .inner_join(documents::table.on(access_logs::document_id.eq(documents::id.nullable())))
            .filter(access_logs::access_type.eq("view"))
            .filter(access_logs::created_at.ge(since))
            .group_by((documents::id, documents::topic))
            .select((documents::id, documents::topic, count_star()))
            .order_by(count_star().desc())
            .limit(limit as i64)
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(documents::project_id.eq(pid));
        }

        let results: Vec<(i64, Option<String>, i64)> = query
            .load(&mut conn)
            .map_err(|e| format!("Failed to get popular documents: {}", e))?;

        Ok(results.into_iter().map(|(doc_id, doc_topic, view_cnt)| PopularDocument {
            document_id: doc_id,
            title: doc_topic.unwrap_or_default(),
            view_count: view_cnt,
            search_count: 0,
            trend_score: 0.0,
        }).collect())
    }

    fn get_hot_entities(&self, project_id: Option<i64>, limit: usize, since: NaiveDateTime) -> Result<Vec<HotEntity>, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        use schema::knowledge_entities;
        use diesel::dsl::count_star;

        let mut query = knowledge_entities::table
            .filter(knowledge_entities::created_at.ge(since))
            .group_by((knowledge_entities::id, knowledge_entities::name, knowledge_entities::entity_type))
            .select((knowledge_entities::id, knowledge_entities::name, knowledge_entities::entity_type, count_star()))
            .order_by(count_star().desc())
            .limit(limit as i64)
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(knowledge_entities::project_id.eq(pid));
        }

        let results: Vec<(i64, Option<String>, Option<String>, i64)> = query
            .load(&mut conn)
            .map_err(|e| format!("Failed to get hot entities: {}", e))?;

        Ok(results.into_iter().map(|(eid, name, e_type, mention_cnt)| HotEntity {
            entity_id: eid,
            entity_name: name.unwrap_or_default(),
            entity_type: e_type.unwrap_or_default(),
            mention_count: mention_cnt,
            trend_score: 0.0,
        }).collect())
    }

    fn get_search_trends(&self, project_id: Option<i64>, limit: usize, since: NaiveDateTime) -> Result<Vec<SearchTrend>, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        use diesel::dsl::count_star;

        let mut query = schema::search_logs::table
            .filter(schema::search_logs::created_at.ge(since))
            .group_by(schema::search_logs::query_text)
            .select((schema::search_logs::query_text, count_star()))
            .order_by(count_star().desc())
            .limit(limit as i64)
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(schema::search_logs::project_id.eq(pid));
        }

        let results: Vec<(String, i64)> = query
            .load(&mut conn)
            .map_err(|e| format!("Failed to get search trends: {}", e))?;

        Ok(results.into_iter().map(|(q, cnt)| SearchTrend {
            query: q,
            count: cnt,
            trend: 0.0,
        }).collect())
    }

    pub async fn get_document_stats(&self, document_id: i64, days: i64) -> Result<DocumentStats, String> {
        let now = Utc::now().naive_utc();
        let since = now - Duration::days(days);

        let view_count = self.count_document_access(document_id, "view", since).unwrap_or(0);
        let search_count = self.count_document_access(document_id, "search", since).unwrap_or(0);

        Ok(DocumentStats {
            document_id,
            view_count,
            search_count,
            share_count: 0,
            average_read_time: 0,
        })
    }

    fn count_document_access(&self, _document_id: i64, _access_type: &str, _since: NaiveDateTime) -> Result<i64, String> {
        Ok(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    pub document_id: i64,
    pub view_count: i64,
    pub search_count: i64,
    pub share_count: i64,
    pub average_read_time: i64,
}
