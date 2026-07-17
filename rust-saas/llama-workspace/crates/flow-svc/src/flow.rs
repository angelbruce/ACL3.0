use shared::models::{Flow, FlowRuntime,FlowDefinition,Agent,Vertex,Edge};
use chrono::Utc;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::executor::FlowExecutor;

pub static EXECUTOR_MANAGER: once_cell::sync::Lazy<Arc<RwLock<ExecutorManager>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(ExecutorManager::new())));



pub struct ExecutorManager {
    executors: HashMap<i64, Arc<FlowExecutor>>,
}

impl ExecutorManager {
    pub fn new() -> Self {
        ExecutorManager {
            executors: HashMap::new(),
        }
    }

    pub fn get(&self, flow_id: i64) -> Option<Arc<FlowExecutor>> {
        self.executors.get(&flow_id).cloned()
    }

    pub fn insert(&mut self, flow_id: i64, executor: Arc<FlowExecutor>) {
        self.executors.insert(flow_id, executor);
    }

    pub fn remove(&mut self, flow_id: i64) {
        self.executors.remove(&flow_id);
    }
}
