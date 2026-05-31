use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Flow, FlowConfigModel, FlowRuntime, FlowRuntimeNode, NodeStatus, Vertex};
use crate::repository::{FlowRepository, FlowRuntimeNodeCreate};
use std::collections::{ HashSet};

pub struct FlowStateMachine {
    repo: FlowRepository,
}

impl FlowStateMachine {
    pub fn new(repo: FlowRepository) -> Self {
        FlowStateMachine { repo }
    }

    /// 启动流程
    /// 需要考虑流程运行到一半的特殊情况，数据没有提交， 如何处理？
    pub async fn start_flow(&self, flow_id: i64) -> ServiceResult<FlowRuntime> {
        //如果启动后，程序停止了，怎么办？
        if let Some(running) = self.repo.get_running_flow_runtime(flow_id).await? {
            return Err(ServiceError::Conflict("Flow is already running".to_string()));
        }

        let flow = self.repo.get_flow(flow_id).await?;
        let config: FlowConfigModel = parse_flow_config(&flow.config)?;
        
        let runtime = self.repo.create_flow_runtime(flow_id).await?;
        
        let head_nodes = extract_head_nodes(&config);
        //创建启动节点集合
        let node_creates = head_nodes.into_iter()
            .map(|v| FlowRuntimeNodeCreate {
                action_id: v.agent.unwrap_or(0),
                action: v.value.clone(),
                prompt: v.prompt.clone(),
                status: NodeStatus::Running,
                next_choice: None,
            })
            .collect();
        
        self.repo.create_flow_runtime_nodes(runtime.id, flow_id, node_creates).await?;
        
        Ok(runtime)
    }

    pub async fn complete_node(&self, runtime_id: i64, node_id: i64) -> ServiceResult<()> {
        let nodes = self.repo.get_flow_runtime_nodes(runtime_id).await?;
        let current_node = nodes.iter().find(|n| n.id == node_id)
            .ok_or(ServiceError::NotFound)?;

        self.repo.update_flow_runtime_node(node_id, NodeStatus::RunningOver).await?;

        let flow = self.repo.get_flow(current_node.flow_id).await?;
        let config = parse_flow_config(&flow.config)?;

        let next_nodes = find_next_nodes(&config, &current_node, &nodes);
        
        for next_node in next_nodes {
            if let Some(existing) = nodes.iter().find(|n| n.action_id == next_node.action_id && n.status == NodeStatus::Running.to_string()) {
                self.repo.update_flow_runtime_node_next_choice(node_id, &existing.id.to_string()).await?;
            } else {
                let node_create = FlowRuntimeNodeCreate {
                    action_id: next_node.action_id,
                    action: next_node.action.clone(),
                    prompt: next_node.prompt.clone(),
                    status: NodeStatus::Running,
                    next_choice: None,
                };
                
                let created = self.repo.create_flow_runtime_nodes(runtime_id, current_node.flow_id, vec![node_create]).await?;
                if let Some(created_node) = created.first() {
                    self.repo.update_flow_runtime_node_next_choice(node_id, &created_node.id.to_string()).await?;
                }
            }
        }

        self.repo.update_flow_runtime_node(node_id, NodeStatus::Stop).await?;
        
        Ok(())
    }
}

fn parse_flow_config(config: &serde_json::Value) -> ServiceResult<FlowConfigModel> {
    serde_json::from_value(config.clone()).map_err(|e| ServiceError::InvalidInput(e.to_string()))
}

fn extract_head_nodes(config: &FlowConfigModel) -> Vec<&Vertex> {
    let mut start_ids = HashSet::new();
    
    //从边中提取开始节点是start的节点，其结束节点就是启动节点。
    //start_ids中存储的是启动节点的id集合
    for edge in &config.edges {
        if let Some(src) = config.vertices.iter().find(|v| v.id == edge.src) {
            if src.r#type == "start" {
                if let Some(target) = config.vertices.iter().find(|v| v.id == edge.target) {
                    start_ids.insert(target.id.clone());
                }
            }
        }
    }
    
    //从顶点中提取启动节点集合
    config.vertices.iter()
        .filter(|v| start_ids.contains(&v.id))
        .collect()
}

fn find_next_nodes(config: &FlowConfigModel, current_node: &FlowRuntimeNode, existing_nodes: &[FlowRuntimeNode]) -> Vec<FlowRuntimeNode> {
    let mut next_nodes = Vec::new();
    
    for edge in &config.edges {
        if edge.src == current_node.action.to_string() {
            if let Some(target_vertex) = config.vertices.iter().find(|v| v.id == edge.target) {
                if target_vertex.r#type != "terminate" && target_vertex.r#type != "over" {
                    let degree = target_vertex.degree.unwrap_or(1);
                    
                    if degree == 1 {
                        if existing_nodes.iter().all(|n| n.action_id != target_vertex.agent.unwrap_or(0) || n.status != NodeStatus::Running.to_string()) {
                            next_nodes.push(FlowRuntimeNode {
                                id: 0,
                                flow_runtime_id: current_node.flow_runtime_id,
                                flow_id: current_node.flow_id,
                                action_id: target_vertex.agent.unwrap_or(0),
                                action: target_vertex.value.clone(),
                                prompt: target_vertex.prompt.clone(),
                                status: NodeStatus::Running.to_string(),
                                next_choice: None,
                                created_at: current_node.created_at.clone(),
                            });
                        }
                    } else if degree == 100 {
                        let from_nodes: Vec<_> = config.edges.iter()
                            .filter(|e| e.target == target_vertex.id)
                            .collect();
                        
                        let all_completed = from_nodes.iter().all(|e| {
                            existing_nodes.iter().any(|n| {
                                n.action == e.src && n.status == NodeStatus::RunningOver.to_string()
                            })
                        });
                        
                        if all_completed {
                            next_nodes.push(FlowRuntimeNode {
                                id: 0,
                                flow_runtime_id: current_node.flow_runtime_id,
                                flow_id: current_node.flow_id,
                                action_id: target_vertex.agent.unwrap_or(0),
                                action: target_vertex.value.clone(),
                                prompt: target_vertex.prompt.clone(),
                                status: NodeStatus::Running.to_string(),
                                next_choice: None,
                                created_at: current_node.created_at.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    next_nodes
}