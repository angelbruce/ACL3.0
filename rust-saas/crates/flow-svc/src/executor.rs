use shared::errors::{ServiceError, ServiceResult};
use shared::models::FlowRuntimeNode;
use crate::repository::FlowRepository;
use crate::agent_repository::AgentRepository;
use crate::state_machine::FlowStateMachine;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;

#[derive(Clone)]
pub struct FlowExecutor {
    /// the flow id
    flow_id: i64,
    /// the runtime id
    runtime_id: i64,
    /// the flow repository
    repo: Arc<FlowRepository>,
    /// the flow state machine
    state_machine: Arc<FlowStateMachine>,
    /// the output channel for node agents
    output_tx: Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>,
    /// the node agents
    node_agents: Arc<RwLock<HashMap<i64, mpsc::Sender<NodeAgentMsg>>>>,
    /// the human node channels
    human_node_channels : Arc<RwLock<HashMap<i64, mpsc::Sender<NodeAgentMsg>>>>,
    /// the running flag
    running: Arc<RwLock<bool>>,
}

impl FlowExecutor {
    pub fn new(flow_id: i64, runtime_id: i64,repo: FlowRepository) -> Self {
        let repo = repo.clone();
        let state_machine = FlowStateMachine::new(repo.clone());
        
        FlowExecutor {
            flow_id,
            runtime_id,
            repo: Arc::new(repo),
            state_machine: Arc::new(state_machine),
            output_tx: Arc::new(RwLock::new(None)),
            node_agents: Arc::new(RwLock::new(HashMap::new())),
            human_node_channels : Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> ServiceResult<()> {
        let mut is_running = self.running.write().await;
        if *is_running {
            return Err(ServiceError::Conflict("Flow is already running".to_string()));
        }
        *is_running = true;
        //release the write lock
        drop(is_running);

        let nodes = self.repo.get_flow_runtime_nodes(self.runtime_id).await?;
        let running_nodes: Vec<_> = nodes.into_iter()
            .filter(|n| n.status == NodeStatus::Running.to_string())
            .collect();

            //empty running nodes, stop the flow
        if running_nodes.is_empty() {
            self.stop_internal().await?;
            return Ok(());
        }

        //tx: output channel for node agents
        //rx: input channel for node agents
        let (tx, mut rx) = mpsc::channel::<NodeAgentMsg>(100);
        *self.output_tx.write().await = Some(tx);

        let node_agents = self.node_agents.clone();
        let running = self.running.clone();

        //spawn a task to handle the output channel
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let id = msg.runtime_node_id;
                if id < 0 {
                    continue;
                }

                //broadcast to all node agents
                if id == 0 {
                    for (_, tx) in node_agents.read().await.iter() {
                        let _ = tx.send(msg.clone()).await;
                    }
                    continue;
                }

                //send to the node agent
                if !node_agents.read().await.contains_key(&id) {
                    continue;
                }
                
                if let Some(tx) = node_agents.read().await.get(&id) {   
                    let _ = tx.send(msg.clone()).await;
                }
            }
        });

        for node in running_nodes {
            self.spawn_node_agent(&node).await?;
        }

        Ok(())
    }

    async fn spawn_node_agent(&self, runtime_node: &FlowRuntimeNode) -> ServiceResult<()> {
        let (input_tx, input_rx) = mpsc::channel::<NodeAgentMsg>(1000);
        let (human_input_tx, human_input_rx) = mpsc::channel::<NodeAgentMsg>(1000);
        self.human_node_channels.write().await.insert(runtime_node.id, human_input_tx);
        
        let node_id = runtime_node.flow_node_id.clone();
        self.node_agents.write().await.insert(runtime_node.id, input_tx);

        let runtime_id = self.runtime_id;
        let runtime_node = runtime_node.clone();
        let repo = self.repo.clone();
        let state_machine = self.state_machine.clone();
        let running = self.running.clone();
        let output_tx = self.output_tx.clone();

        tokio::spawn(async move {
            let mut agent = NodeAgent::new(
                runtime_node.id,
                runtime_id,
                node_id,
                runtime_node.action_id,
                runtime_node.action.clone(),
                runtime_node.prompt.clone(),
                repo.clone(),
            );



            if let Err(e) = agent.run(input_rx, output_tx, human_input_rx).await {
                tracing::error!("Node agent {} error: {:?}", runtime_node.id, e);
            }

            if let Err(e) = state_machine.complete_node(runtime_id, runtime_node.id).await {
                tracing::error!("Failed to complete node {}: {:?}", runtime_node.id, e);
            }

            if let Err(e) = repo.update_flow_runtime_node(runtime_node.id, NodeStatus::Stop).await {
                tracing::error!("Failed to stop node {}: {:?}", runtime_node.id, e);
            }

            let is_still_running = *running.read().await;
            if is_still_running {
                if let Ok(nodes) = repo.get_flow_runtime_nodes(runtime_id).await {
                    let has_running = nodes.iter().any(|n| {
                        n.status == NodeStatus::Running.to_string()
                    });
                    if !has_running {
                        if let Err(e) = repo.stop_flow_runtime(runtime_id).await {
                            tracing::error!("Failed to stop flow runtime {}: {:?}", runtime_id, e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> ServiceResult<()> {
        let mut is_running = self.running.write().await;
        if !*is_running {
            return Ok(());
        }
        *is_running = false;
        drop(is_running);

        self.stop_internal().await
    }

    async fn stop_internal(&self) -> ServiceResult<()> {
        for (_, tx) in self.node_agents.write().await.drain() {
            let _ = tx.send(NodeAgentMsg::new(0, "__STOP__".to_string())).await;
        }
        
        let _ = self.repo.stop_flow_runtime(self.runtime_id).await;
        Ok(())
    }

    pub async fn can_run(&self) -> bool {
        if let Ok(Some(runtime)) = self.repo.get_running_flow_runtime(self.flow_id).await {
            runtime.id == self.runtime_id && !runtime.is_over
        } else {
            false
        }
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

use shared::models::NodeStatus;

#[derive(Clone)]
pub struct NodeAgentMsg {
    pub runtime_node_id: i64,
    pub msg: String,
    pub msg_type: NodeAgentMsgType,
}

#[derive(Clone)]
pub enum NodeAgentMsgType {
    AI,
    Human,
}

impl NodeAgentMsg {
    pub fn new(runtime_node_id: i64, msg: String) -> Self {
        Self {
            runtime_node_id,
            msg,
            msg_type: NodeAgentMsgType::AI,
        }
    }
}

pub struct NodeAgent {
    runtime_node_id: i64,
    runtime_id: i64,
    node_id: String,
    action_id: i64,
    action: String,
    prompt: Option<String>,
    human: bool,
    repo: Arc<FlowRepository>,
}

impl NodeAgent {
    pub fn new(
        runtime_node_id: i64,
        runtime_id: i64,
        node_id: String,
        action_id: i64,
        action: String,
        prompt: Option<String>,
        repo: Arc<FlowRepository>,
    ) -> Self {
        NodeAgent {
            runtime_node_id,
            runtime_id,
            node_id,
            action_id,
            action,
            prompt,
            human: false,
            repo,
        }
    }

    pub async fn run(
        &mut self,
        mut input_rx: mpsc::Receiver<NodeAgentMsg>,
        output_tx: Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>, 
        mut human_input_rx: mpsc::Receiver<NodeAgentMsg>,
    ) -> ServiceResult<()> {
        let mut messages: Vec<shared::models::ChatMessage> = Vec::new();

        let repo = crate::agent_repository::AgentRepository::new(self.repo.clone().get_pool().clone());
        let agent = repo.get_agent_by_id(self.action_id).await?;
        let human_data = self.repo.get_flow_runtime_node_human(self.runtime_node_id).await?;
        self.human = human_data == 1;
       
        if let Some(ref prompt) = self.prompt {
            messages.push(shared::models::ChatMessage {
                role: "system".to_string(),
                content: Some(prompt.clone()),
                ..Default::default()
            });
        }
        
        messages.push(shared::models::ChatMessage{
            role: "system".to_string(),
            content: agent.defination.clone(),
            ..Default::default()
        });

        messages.push(shared::models::ChatMessage {
            role: "system".to_string(),
            content: Some("**任务约束**：如果已经完成所有任务请输出````__stop__````，如何需要人参与，请输出````__human__````".to_string()), 
            ..Default::default()
        });

        messages.push(shared::models::ChatMessage {
            role: "system".to_string(),
            content: Some(format!("**任务目标**：{}", self.action)),
            ..Default::default()
        });

        loop {

            let mut recv_msg: Option<NodeAgentMsg> = None;
            if self.human {
                //进入阻塞模式，等待人参与
                if let Some(data) = human_input_rx.recv().await {
                    recv_msg = Some(data);
                }
                self.repo.update_flow_runtime_node_human(self.runtime_node_id, 0).await?;
                self.human = false;
                continue;
            } else {
                 loop {
                    tokio::select! {
                        Some(msg) = input_rx.recv() => {
                            recv_msg = Some(msg);
                            break;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {
                            break;
                        }
                    }
                }
            }

            if let Some(msg) = recv_msg {
                if msg.runtime_node_id == 0 
                    && msg.msg == "__STOP__" {
                    break;
                }

                //有注入风险，暂不考虑。
                if msg.msg.contains("````__stop__````") {
                    break;
                }

                //有注入风险，暂不考虑。
                if msg.msg.contains("````__human__````") {
                    self.human = true;
                    self.repo.update_flow_runtime_node_human(self.runtime_node_id, 1).await?;
                    continue;
                }
                
                let role =  match msg.msg_type {
                                        NodeAgentMsgType::AI => {
                                            "assistant".to_string()
                                        }
                                        NodeAgentMsgType::Human => {
                                            "user".to_string()
                                        }
                                    };

                messages.push(shared::models::ChatMessage {
                    role : role.clone(),
                    content: Some(msg.msg), 
                    ..Default::default()
                });

            } else {
                 messages.push(shared::models::ChatMessage {
                    role: "user".to_string(),
                    content: Some("请继续执行当前任务。".to_string()), 
                    ..Default::default()
                });
            }
        
            let result = self.call_llm(&messages, &output_tx).await;
            if let Err(e) = result {
                tracing::error!("LLM error: {:?}", e);
                //如果LLM调用失败，需要重试
                continue;
            }

        }

        Ok(())
    }

    async fn call_llm(
        &self,
        messages: &[shared::models::ChatMessage],
        output_tx: &Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>,
    ) -> ServiceResult<()> {
        let model_repo = crate::repository::ModelRepository::new();
        let model = model_repo.get_default_model().await?;

        let client = crate::llm_client::LlmClient::new(
            &model.access_url,
            &model.api_key,
            &model.name,
        );

        let mut full_response = String::new();

        //此处应该调用LLM-SERVICE进行，LLM-SERVICE会根据模型配置调用LLM-CLIENT
        //将来会根据情况进行部署（负载均衡、高可用）
        let stream = client.chat_stream(messages, None).await?;

        tokio::pin!(stream);

        while let Some(response) = stream.next().await {
            match response {
                Ok(resp) => {
                    full_response.push_str(&resp.content);

                    if let Some(tx) = output_tx.read().await.as_ref() {
                        let data = NodeAgentMsg::new(self.runtime_node_id, resp.content.clone());
                        let _ = tx.send(data).await;
                    }

                    if resp.done {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("LLM stream error: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
