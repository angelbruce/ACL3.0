use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Flow, FlowConfigModel, FlowRuntime, FlowRuntimeNode, NodeStatus, Vertex};
use crate::repository::{FlowRepository, FlowRuntimeNodeCreate};
use std::collections::{ HashSet};
use crate::executor::FlowExecutor;

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
                flow_node_id: v.id.clone(),
                action_id: v.agent.unwrap_or(0),
                action: v.value.clone(),
                prompt: v.prompt.clone(),
                status: NodeStatus::Running,
                next_choice: None,
                human: if v.r#type == "input" { 1 } else { 0 },
            })
            .collect();
        
        self.repo.create_flow_runtime_nodes(runtime.id, flow_id, node_creates).await?;
        Ok(runtime)
    }

    /// complete current node and navigate the next target node or finish the flow
    /// todo! r#type==input的节点，需要从前端界面上获取输入信息，方能继续执行。
    pub async fn complete_node(&self, runtime_id: i64, runtime_node_id: i64, next_choice: Option<String>) -> ServiceResult<()> {
        let nodes = self.repo.get_flow_runtime_nodes(runtime_id).await?;
        let current_node = nodes.iter().find(|n| n.id == runtime_node_id)
            .ok_or(ServiceError::NotFound)?;

        tracing::info!("complete_node: runtime_id={}, runtime_node_id={}, flow_node_id={}, next_choice={:?}", 
            runtime_id, runtime_node_id, current_node.flow_node_id, next_choice);

        self.repo.update_flow_runtime_node(runtime_node_id, NodeStatus::RunningOver).await?;

        let flow = self.repo.get_flow(current_node.flow_id).await?;
        let config = parse_flow_config(&flow.config)?;

        let next_nodes = find_next_nodes(&config, &current_node, &nodes, next_choice.as_deref());
        tracing::info!("complete_node: next_nodes={:?}", next_nodes.as_ref().map(|n| n.len()));
        
        match(next_nodes) {
            Some(next_nodes) => {
                if next_nodes.is_empty() {
                    tracing::info!("complete_node: next_nodes is empty, no new nodes to create");
                } else {
                    tracing::info!("complete_node: creating {} new nodes", next_nodes.len());
                    for next_node in next_nodes {
                        let node_create = FlowRuntimeNodeCreate {
                            flow_node_id: next_node.flow_node_id.clone(),
                            action_id: next_node.action_id,
                            action: next_node.action.clone(),
                            prompt: next_node.prompt.clone(),
                            status: NodeStatus::Running,
                            next_choice: None,
                            human: next_node.human,
                        };
                        
                        // 创建要执行的节点
                        let created = self.repo.create_flow_runtime_nodes(runtime_id, current_node.flow_id,vec![node_create]).await?;
                        let mut choice = vec![];
                        for created_node in created.iter() {
                            choice.push(created_node.id.to_string());
                        }
                        
                        let choice = choice.join(",");  
                        self.repo.update_flow_runtime_node_next_choice(runtime_node_id, &choice).await?;
                    }
                }
            },
            None => {
                tracing::info!("complete_node: next_nodes is None, flow is terminating");
                self.repo.update_flow_runtime_status(runtime_id, true).await?;
                self.repo.update_flow_runtime_node_next_choice(runtime_node_id, &current_node.id.to_string()).await?;
            }
        }

        self.repo.update_flow_runtime_node(runtime_node_id, NodeStatus::Stop).await?;
        
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
        //find data from vertices by edge.src
        if let Some(src) = config.vertices.iter().find(|v| v.id == edge.src) {
            //if the src vertex is start, then the target vertex is the head node. the type which value is 'start' is the judgement.
            if src.r#type == "start" {
                //find data from vertices by edge.target
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

/// 缺少r#type=='input'的处理，此为人类输入的节点，需要从前端界面上获取输入信息，方能继续执行。
fn find_next_nodes(config: &FlowConfigModel, current_node: &FlowRuntimeNode, existing_nodes: &[FlowRuntimeNode], next_choice: Option<&str>) -> Option<Vec<FlowRuntimeNode>> {
    let mut next_nodes = Vec::new();
    
    tracing::info!("find_next_nodes: current_node={}, flow_node_id={}, next_choice={:?}", 
        current_node.id, current_node.flow_node_id, next_choice);
    
    let mut filtered_edges: Vec<&shared::models::Edge> = config.edges.iter()
        .filter(|e| e.src == current_node.flow_node_id.clone())
        .collect();
    
    tracing::info!("find_next_nodes: found {} edges from current node", filtered_edges.len());
    
    // 如果做出选择
    if let Some(choice) = next_choice {
        let mut new_filtered_edges = Vec::new();
        let choice = choice.trim();
        for edge in &filtered_edges {
            if edge.value == choice || edge.value.trim() == choice || choice.contains(edge.value.trim()) {
                new_filtered_edges.push(*edge);
            }
        }
        
        if !new_filtered_edges.is_empty() {
            //有从边做出选择的节点，正确做出了选择，或者AI做出了正确的选择。
            filtered_edges = new_filtered_edges;
        } else {
            //从关联节点做出选择
            filtered_edges = filtered_edges.into_iter()
                .filter(|e| {
                    config.vertices.iter().any(|x| x.id == e.target && x.value.trim() == choice || choice.contains(x.value.trim()))
                })
                .collect();
        }
    }
    
    //条件判定，节点执行下一步的条件
    for edge in filtered_edges {
        if let Some(target_vertex) = config.vertices.iter().find(|v| v.id == edge.target) {
            if target_vertex.r#type != "terminate" && target_vertex.r#type != "over" {
                let degree = target_vertex.degree.unwrap_or(1);
                
                if degree == 1 {
                    next_nodes.push(FlowRuntimeNode {
                        id: 0,
                        flow_runtime_id: current_node.flow_runtime_id,
                        flow_id: current_node.flow_id,
                        flow_node_id: target_vertex.id.clone(),
                        action_id: target_vertex.agent.unwrap_or(0),
                        action: target_vertex.value.clone(),
                        prompt: target_vertex.prompt.clone(),
                        status: NodeStatus::Running.to_string(),
                        next_choice: None,
                        created_at: current_node.created_at.clone(),
                        human: if target_vertex.r#type == "input" { 1 } else { 0 },
                    });
                } else if degree == 100 {
                    let from_nodes: Vec<_> = config.edges.iter()
                        .filter(|e| e.target == target_vertex.id)
                        .collect();
                    
                    let all_completed = from_nodes.iter().all(|e| {
                        existing_nodes.iter().any(|n| {
                            n.flow_node_id == e.src && n.status == NodeStatus::RunningOver.to_string()
                        })
                    });
                    
                    if all_completed {
                        next_nodes.push(FlowRuntimeNode {
                            id: 0,
                            flow_runtime_id: current_node.flow_runtime_id,
                            flow_id: current_node.flow_id,
                            flow_node_id: target_vertex.id.clone(),
                            action_id: target_vertex.agent.unwrap_or(0),
                            action: target_vertex.value.clone(),
                            prompt: target_vertex.prompt.clone(),
                            status: NodeStatus::Running.to_string(),
                            next_choice: None,
                            created_at: current_node.created_at.clone(),
                            human: if target_vertex.r#type == "input" { 1 } else { 0 },
                        });
                    }
                }
            } else {
                return None;
            }
        }
    }
    
    return Some(next_nodes);
}