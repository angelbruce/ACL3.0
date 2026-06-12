use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Flow, FlowRuntime, FlowRuntimeNode, NodeStatus, CreateFlowRequest, LlmModel};
use shared::schema::{flows, flow_runtimes, flow_runtime_nodes, llm_models};
use std::env;

pub struct FlowRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl FlowRepository {
    pub fn get_pool(&self) -> r2d2::Pool<ConnectionManager<PgConnection>> {
        self.pool.clone()
    }
}

impl Clone for FlowRepository {
    fn clone(&self) -> Self {
        FlowRepository {
            pool: self.pool.clone(),
        }
    }
}

impl FlowRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        FlowRepository { pool }
    }

    pub async fn get_all_flows(&self) -> ServiceResult<Vec<Flow>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let flows = flows::table
            .order(flows::created_at.desc())
            .load::<Flow>(&mut conn)?;
        
        Ok(flows)
    }

    pub async fn get_flow(&self, id: i64) -> ServiceResult<Flow> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let flow = flows::table
            .filter(flows::id.eq(id))
            .first::<Flow>(&mut conn)?;
        
        Ok(flow)
    }

    pub async fn create_flow(&self, req: CreateFlowRequest) -> ServiceResult<Flow> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let flow = diesel::insert_into(flows::table)
            .values((
                flows::name.eq(&req.name),
                flows::config.eq(&req.config),
                flows::created_at.eq(now),
            ))
            .returning(Flow::as_select())
            .get_result(&mut conn)?;
        
        Ok(flow)
    }

    pub async fn update_flow(&self, id: i64, req: CreateFlowRequest) -> ServiceResult<Flow> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let flow = diesel::update(flows::table)
            .filter(flows::id.eq(id))
            .set((
                flows::name.eq(&req.name),
                flows::config.eq(&req.config),
            ))
            .returning(Flow::as_select())
            .get_result(&mut conn)?;
        
        Ok(flow)
    }


    pub async fn delete_flow(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::delete(flow_runtime_nodes::table.filter(flow_runtime_nodes::flow_id.eq(id))).execute(&mut conn)?;
        diesel::delete(flow_runtimes::table.filter(flow_runtimes::flow_id.eq(id))).execute(&mut conn)?;
        diesel::delete(flows::table.filter(flows::id.eq(id))).execute(&mut conn)?;
        
        Ok(())
    }

    pub async fn create_flow_runtime(&self, flow_id: i64) -> ServiceResult<FlowRuntime> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        
        let runtime = diesel::insert_into(flow_runtimes::table)
            .values((
                flow_runtimes::flow_id.eq(flow_id),
                flow_runtimes::is_over.eq(false),
                flow_runtimes::created_at.eq(now),
            ))
            .returning(FlowRuntime::as_select())
            .get_result(&mut conn)?;
        
        Ok(runtime)
    }

    pub async fn get_flow_runtimes(&self, flow_id: i64) -> ServiceResult<Vec<FlowRuntime>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let runtimes = flow_runtimes::table
            .filter(flow_runtimes::flow_id.eq(flow_id))
            .order(flow_runtimes::created_at.desc())
            .load::<FlowRuntime>(&mut conn)?;
        
        Ok(runtimes)
    }

    pub async fn get_flow_runtime_with_nodes(&self, runtime_id: i64) -> ServiceResult<(FlowRuntime, Vec<FlowRuntimeNode>)> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let runtime = flow_runtimes::table
            .filter(flow_runtimes::id.eq(runtime_id))
            .first::<FlowRuntime>(&mut conn)?;

        let nodes = flow_runtime_nodes::table
            .filter(flow_runtime_nodes::flow_runtime_id.eq(runtime_id))
            .load::<FlowRuntimeNode>(&mut conn)?;
        
        Ok((runtime, nodes))
    }

    pub async fn stop_flow_runtime(&self, runtime_id: i64) -> ServiceResult<FlowRuntime> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let runtime = diesel::update(flow_runtimes::table)
            .filter(flow_runtimes::id.eq(runtime_id))
            .set(flow_runtimes::is_over.eq(true))
            .returning(FlowRuntime::as_select())
            .get_result(&mut conn)?;
        
        Ok(runtime)
    }
   

    pub async fn create_flow_runtime_nodes(&self, runtime_id: i64, flow_id: i64, nodes: Vec<FlowRuntimeNodeCreate>) -> ServiceResult<Vec<FlowRuntimeNode>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        let mut result_nodes = Vec::new();
        
        for node in nodes {
            let new_node = diesel::insert_into(flow_runtime_nodes::table)
                .values((
                    flow_runtime_nodes::flow_runtime_id.eq(runtime_id),
                    flow_runtime_nodes::flow_id.eq(flow_id),
                    flow_runtime_nodes::flow_node_id.eq(node.flow_node_id),
                    flow_runtime_nodes::action_id.eq(node.action_id),
                    flow_runtime_nodes::action.eq(&node.action),
                    flow_runtime_nodes::prompt.eq(&node.prompt),
                    flow_runtime_nodes::status.eq(node.status.to_string()),
                    flow_runtime_nodes::next_choice.eq(&node.next_choice),
                    flow_runtime_nodes::created_at.eq(now),
                    flow_runtime_nodes::human.eq(0),
                ))
                .returning(FlowRuntimeNode::as_select())
                .get_result(&mut conn)?;
            
            result_nodes.push(new_node);
        }
        
        Ok(result_nodes)
    }

    pub async fn update_flow_runtime_node(&self, runtime_node_id: i64, status: NodeStatus) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::update(flow_runtime_nodes::table)
            .filter(flow_runtime_nodes::id.eq(runtime_node_id))
            .set(flow_runtime_nodes::status.eq(status.to_string()))
            .execute(&mut conn)?;
        
        Ok(())
    }

    pub async fn update_flow_runtime_node_next_choice(&self, runtime_node_id: i64, next_choice: &str) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::update(flow_runtime_nodes::table)
            .filter(flow_runtime_nodes::id.eq(runtime_node_id))
            .set(flow_runtime_nodes::next_choice.eq(next_choice))
            .execute(&mut conn)?;
        
        Ok(())
    }

     pub async fn update_flow_runtime_node_human(&self, runtime_node_id: i64, human: i32) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let _ = diesel::update(flow_runtime_nodes::table)
            .filter(flow_runtime_nodes::id.eq(runtime_node_id))
            .set(flow_runtime_nodes::human.eq(human))
            .execute(&mut conn)?;
        
        Ok(())
    }


    pub async fn get_flow_runtime_node_human(&self, runtime_node_id: i64) -> ServiceResult<i32> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let human = flow_runtime_nodes::table
            .filter(flow_runtime_nodes::id.eq(runtime_node_id))
            .first::<FlowRuntimeNode>(&mut conn)?;
        
        Ok(human.human)
    }

    pub async fn get_flow_runtime_nodes(&self, runtime_id: i64) -> ServiceResult<Vec<FlowRuntimeNode>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let nodes = flow_runtime_nodes::table
            .filter(flow_runtime_nodes::flow_runtime_id.eq(runtime_id))
            .load::<FlowRuntimeNode>(&mut conn)?;
        
        Ok(nodes)
    }

    pub async fn get_running_flow_runtime(&self, flow_id: i64) -> ServiceResult<Option<FlowRuntime>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let runtime = flow_runtimes::table
            .filter(flow_runtimes::flow_id.eq(flow_id))
            .filter(flow_runtimes::is_over.eq(false))
            .first::<FlowRuntime>(&mut conn)
            .optional()?;
        
        Ok(runtime)
    }

    pub async fn update_flow_runtime_status(&self, runtime_id: i64, is_over: bool) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        diesel::update(flow_runtimes::table)
            .filter(flow_runtimes::id.eq(runtime_id))
            .set(flow_runtimes::is_over.eq(is_over))
            .execute(&mut conn)?;
        
        Ok(())
    }
}

pub struct FlowRuntimeNodeCreate {
    pub flow_node_id: String,
    pub action_id: i64,
    pub action: String,
    pub prompt: Option<String>,
    pub status: NodeStatus,
    pub next_choice: Option<String>,
}

pub struct ModelRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Clone for ModelRepository {
    fn clone(&self) -> Self {
        ModelRepository {
            pool: self.pool.clone(),
        }
    }
}

impl ModelRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        ModelRepository { pool }
    }

    pub async fn get_default_model(&self) -> ServiceResult<LlmModel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let model = llm_models::table
            .filter(llm_models::is_default.eq(true))
            .first::<LlmModel>(&mut conn)
            .optional()?
            .ok_or(ServiceError::NotFound)?;
        
        Ok(model)
    }

    pub async fn get_all_models(&self) -> ServiceResult<Vec<LlmModel>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let models = llm_models::table
            .load::<LlmModel>(&mut conn)?;
        
        Ok(models)
    }
}
