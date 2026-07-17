use std::collections::HashMap;
use std::env;
use chrono::{Utc, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::models::{Agent, CreateFlowRequest, Flow, FlowRuntime, FlowRuntimeNode, LlmModel, NodeStatus};
use shared::schema::{flows, flow_runtimes, flow_runtime_nodes, agents};
use shared::repository::DalDataList;
use shared::errors::{ServiceError, ServiceResult};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable,AsChangeset)]
#[diesel(table_name = crate::schema::flow_runtime_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlowRuntimeSession {
    pub id: i64,
    pub flow_id: i64,
    pub flow_runtime_id: String,
    pub creator_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


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
    pub created_at: NaiveDateTime,
    pub creator_id: i64,
}


//      -> 总管 任务目标是什么，理解用户意图与需求 -> 检测意图理解是否正确（与人沟通？上网扒拉？）
//      -> 总监 需要哪些团队，为团队赋予蓝图、任务目标 确定交付组（有人给我产品），配置组（在哪儿提交过程文档）， 验证组（干的对不对）、实施组（谁来干活）、沟通组（问题有人反馈、沟通、相应）。 管理过程纲要：需求有人沟通、干哪些事情、活有人干、干后有人检、问题有人改、成果可生产应用。
//      -> 分解任务目标 归属到团队 （需要谁来干（定义属性））（成果要什么（给什么）） 
//      -> 团队任务 将任务目标分解为工作任务（干什么），定义产品输出：要求、约束、格式、条件、边界、管理活动过程记录、结果、验证方法、结果验证（输出、验证、过程），做到事情有人干，过程有人管，事情能落地，质量有保障。
//      -> 招募团队 -> 基于团队的要求为团队招募AGENT，设定AGENT职能（干什么的） 技能（如何干） 工具集 （手脚）
//      -> 团队协作 -> 团队成员领取任务（可以是团队管理人员分发任务），沟通、验证与工作分开为不同的AGENT,等待AGENT干完后，交给检验人员进行检测，形成结论，如有问题，反馈循环。
//      -> 管理交付口验证 针对任务

