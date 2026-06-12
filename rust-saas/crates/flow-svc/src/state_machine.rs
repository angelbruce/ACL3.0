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

        println!("21");
        let flow = self.repo.get_flow(flow_id).await?;
        println!("22");
        let config: FlowConfigModel = parse_flow_config(&flow.config)?;
        
        println!("23");
        let runtime = self.repo.create_flow_runtime(flow_id).await?;
        println!("24");
        
        let head_nodes = extract_head_nodes(&config);
        println!("25");
        //创建启动节点集合
        let node_creates = head_nodes.into_iter()
            .map(|v| FlowRuntimeNodeCreate {
                flow_node_id: v.id.clone(),
                action_id: v.agent.unwrap_or(0),
                action: v.value.clone(),
                prompt: v.prompt.clone(),
                status: NodeStatus::Running,
                next_choice: None,
            })
            .collect();
        println!("26");
        
        self.repo.create_flow_runtime_nodes(runtime.id, flow_id, node_creates).await?;
        println!("27");
        Ok(runtime)
    }

    /// complete current node and navigate the next target node or finish the flow
    pub async fn complete_node(&self, runtime_id: i64, runtime_node_id: i64) -> ServiceResult<()> {
        let nodes = self.repo.get_flow_runtime_nodes(runtime_id).await?;
        let current_node = nodes.iter().find(|n| n.id == runtime_node_id)
            .ok_or(ServiceError::NotFound)?;

        //错误的更新方式，必须通过id进行更新，不能通过node_id进行更新，一个流程中的节点可以存在多个实例在运行的时候。
        self.repo.update_flow_runtime_node(runtime_node_id, NodeStatus::RunningOver).await?;

        let flow = self.repo.get_flow(current_node.flow_id).await?;
        let config = parse_flow_config(&flow.config)?;

        let next_nodes = find_next_nodes(&config, &current_node, &nodes);
        match(next_nodes) {
            Some(next_nodes) => {
                for next_node in next_nodes {
                    let node_create = FlowRuntimeNodeCreate {
                        flow_node_id: next_node.flow_node_id.clone(),
                        action_id: next_node.action_id,
                        action: next_node.action.clone(),
                        prompt: next_node.prompt.clone(),
                        status: NodeStatus::Running,
                        next_choice: None,
                    };
                    
                    let created = self.repo.create_flow_runtime_nodes(runtime_id, current_node.flow_id,vec![node_create]).await?;
                    let mut choice = vec![];
                    for created_node in created.iter() {
                        choice.push(created_node.id.to_string());
                    }
                    
                    let choice = choice.join(",");  
                    self.repo.update_flow_runtime_node_next_choice(runtime_node_id, &choice).await?;
                }
            },
            None => {
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

fn find_next_nodes(config: &FlowConfigModel, current_node: &FlowRuntimeNode, existing_nodes: &[FlowRuntimeNode]) -> Option<Vec<FlowRuntimeNode>> {
    let mut next_nodes = Vec::new();
    //perhaps we should check the status of the current node, if it's not running, then we should not find the next nodes.
    for edge in &config.edges {
        //find the target nodes which pointed from current_node.
        //if the target vertex is terminate or over, then we should not find the next nodes.
        if edge.src == current_node.flow_node_id.clone() {
            //find the node's definition of the target vertex included by flow vertices.
            if let Some(target_vertex) = config.vertices.iter().find(|v| v.id == edge.target) {
                // not end node [!= terminate, != over]
                if target_vertex.r#type != "terminate" && target_vertex.r#type != "over" {
                    // find the node's degree to judge can move next or not.
                    let degree = target_vertex.degree.unwrap_or(1);
                    //any pointed node is complete
                    if degree == 1 {
                        // current_node is over, directly jump to the next target node  
                        // if existing_nodes.iter().all(|n| n.flow_node_id != target_vertex.id.clone() && n.status != NodeStatus::Running.to_string()) {
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
                                human: 0,
                            });
                        // }
                    } else if degree == 100 {
                        let from_nodes: Vec<_> = config.edges.iter()
                            .filter(|e| e.target == target_vertex.id)
                            .collect();
                        
                        let all_completed = from_nodes.iter().all(|e| {
                            existing_nodes.iter().any(|n| {
                                n.id != current_node.id && 
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
                                human: 0,
                            });
                        }
                    }
                }
                else {
                    return None;
                }
            }
        }
    }
    
    return Some(next_nodes);
}