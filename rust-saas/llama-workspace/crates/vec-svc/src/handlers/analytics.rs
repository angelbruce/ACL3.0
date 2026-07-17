//!  API 处理

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::analytics::{AnalyticsSummary, DocumentStats};

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub project_id: Option<i64>,
    pub days: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentStatsQuery {
    pub document_id: i64,
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<AnalyticsSummary>,
}

#[derive(Debug, Serialize)]
pub struct DocumentStatsResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<DocumentStats>,
}

pub async fn get_analytics_summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Json<AnalyticsResponse> {
    let days = query.days.unwrap_or(7);

    match state.analytics_service.get_summary(query.project_id, days).await {
        Ok(summary) => Json(AnalyticsResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(summary),
        }),
        Err(e) => Json(AnalyticsResponse {
            code: 500,
            message: format!("Failed to get analytics: {}", e),
            data: None,
        }),
    }
}

pub async fn get_document_stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DocumentStatsQuery>,
) -> Json<DocumentStatsResponse> {
    let days = query.days.unwrap_or(30);

    match state.analytics_service.get_document_stats(query.document_id, days).await {
        Ok(stats) => Json(DocumentStatsResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(stats),
        }),
        Err(e) => Json(DocumentStatsResponse {
            code: 500,
            message: format!("Failed to get document stats: {}", e),
            data: None,
        }),
    }
}
