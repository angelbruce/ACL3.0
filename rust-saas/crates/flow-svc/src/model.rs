use std::env;
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::models::{Agent, CreateFlowRequest, Flow, FlowRuntime, FlowRuntimeNode, LlmModel, NodeStatus};
use shared::schema::{flows, flow_runtimes, flow_runtime_nodes, agents};
use shared::repository::DalDataList;
use shared::errors::{ServiceError, ServiceResult};
use extmacros::Repository;
use serde::{Serialize, Deserialize};

#[derive(Repository)]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable,AsChangeset)]
#[diesel(table_name = crate::schema::flow_runtime_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlowRuntimeSession {
    pub id: i64,
    pub flow_id: i64,
    pub flow_runtime_id: String,
    pub creator_id: i64,
    pub created_at: String,
    pub updated_at: String,
}


#[derive(Repository)]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable,AsChangeset)]
#[diesel(table_name = crate::schema::flow_runtime_session_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlowRuntimeSessionItem {
    pub id: i64,
    pub flow_id: i64,
    pub flow_runtime_id: String,
    pub flow_runtime_session_id: i64,
    pub flow_runtime_node_id: String,
    pub session_type: String,
    pub content: String,
    pub action_id: i64,
    pub created_at: String,
    pub creator_id: i64,
}
