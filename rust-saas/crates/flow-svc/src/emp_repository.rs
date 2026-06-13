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
#[diesel(table_name = crate::schema::employees)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Employee {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
}


#[test]
pub fn test_create() {
    let repo = EmployeeRepository::new(None);
    let employee = Employee {
        id: 0,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
        phone: "1234567890".to_string(),
    };

    repo.insert(employee).unwrap();
}