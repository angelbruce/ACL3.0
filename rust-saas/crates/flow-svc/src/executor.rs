use shared::errors::{ServiceError, ServiceResult};
use shared::models::FlowRuntimeNode;
use crate::repository::FlowRepository;
use crate::state_machine::FlowStateMachine;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;

#[derive(Clone)]
pub struct FlowExecutor {
    flow_id: i64,
    runtime_id: i64,
    repo: Arc<FlowRepository>,
    state_machine: Arc<FlowStateMachine>,
    output_tx: Arc<RwLock<Option<mpsc::Sender<String>>>>,
    node_agents: Arc<RwLock<HashMap<i64, mpsc::Sender<String>>>>,
    running: Arc<RwLock<bool>>,
}

impl FlowExecutor {
    pub fn new(flow_id: i64, runtime_id: i64) -> Self {
        let repo = FlowRepository::new();
        let state_machine = FlowStateMachine::new(repo.clone());
        
        FlowExecutor {
            flow_id,
            runtime_id,
            repo: Arc::new(repo),
            state_machine: Arc::new(state_machine),
            output_tx: Arc::new(RwLock::new(None)),
            node_agents: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> ServiceResult<()> {
        let mut is_running = self.running.write().await;
        if *is_running {
            return Err(ServiceError::Conflict("Flow is already running".to_string()));
        }
        *is_running = true;
        drop(is_running);

        let nodes = self.repo.get_flow_runtime_nodes(self.runtime_id).await?;
        let running_nodes: Vec<_> = nodes.into_iter()
            .filter(|n| n.status == NodeStatus::Running.to_string())
            .collect();

        if running_nodes.is_empty() {
            self.stop_internal().await?;
            return Ok(());
        }

        let (tx, mut rx) = mpsc::channel::<String>(100);
        *self.output_tx.write().await = Some(tx);

        let node_agents = self.node_agents.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Some(tx) = node_agents.read().await.values().next() {
                    let _ = tx.send(msg).await;
                }
            }
        });

        for node in running_nodes {
            self.spawn_node_agent(&node).await?;
        }

        Ok(())
    }

    async fn spawn_node_agent(&self, node: &FlowRuntimeNode) -> ServiceResult<()> {
        let (input_tx, input_rx) = mpsc::channel::<String>(100);
        
        let node_id = node.id;
        self.node_agents.write().await.insert(node_id, input_tx);

        let runtime_id = self.runtime_id;
        let node = node.clone();
        let repo = self.repo.clone();
        let state_machine = self.state_machine.clone();
        let running = self.running.clone();
        let output_tx = self.output_tx.clone();

        tokio::spawn(async move {
            let agent = NodeAgent::new(
                runtime_id,
                node.id,
                node.action_id,
                node.action.clone(),
                node.prompt.clone(),
            );

            if let Err(e) = agent.run(input_rx, output_tx).await {
                tracing::error!("Node agent {} error: {:?}", node_id, e);
            }

            if let Err(e) = state_machine.complete_node(runtime_id, node_id).await {
                tracing::error!("Failed to complete node {}: {:?}", node_id, e);
            }

            if let Err(e) = repo.update_flow_runtime_node(node_id, NodeStatus::Stop).await {
                tracing::error!("Failed to stop node {}: {:?}", node_id, e);
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
            let _ = tx.send("__STOP__".to_string()).await;
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

pub struct NodeAgent {
    runtime_id: i64,
    node_id: i64,
    action_id: i64,
    action: String,
    prompt: Option<String>,
}

impl NodeAgent {
    pub fn new(
        runtime_id: i64,
        node_id: i64,
        action_id: i64,
        action: String,
        prompt: Option<String>,
    ) -> Self {
        NodeAgent {
            runtime_id,
            node_id,
            action_id,
            action,
            prompt,
        }
    }

    pub async fn run(
        &self,
        mut input_rx: mpsc::Receiver<String>,
        output_tx: Arc<RwLock<Option<mpsc::Sender<String>>>>,
    ) -> ServiceResult<()> {
        let mut messages: Vec<shared::models::ChatMessage> = Vec::new();

        if let Some(ref prompt) = self.prompt {
            messages.push(shared::models::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt.clone()),
                ..Default::default()
            });
        }

        messages.push(shared::models::ChatMessage {
            role: "user".to_string(),
            content: Some(self.action.clone()),
            ..Default::default()
        });

        loop {
            tokio::select! {
                Some(msg) = input_rx.recv() => {
                    if msg == "__STOP__" {
                        break;
                    }

                    messages.push(shared::models::ChatMessage {
                        role: "user".to_string(),
                        content: Some(msg),
                        ..Default::default()
                    });

                    let _ = self.call_llm(&messages, &output_tx).await;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn call_llm(
        &self,
        messages: &[shared::models::ChatMessage],
        output_tx: &Arc<RwLock<Option<mpsc::Sender<String>>>>,
    ) -> ServiceResult<()> {
        let model_repo = crate::repository::ModelRepository::new();
        let model = model_repo.get_default_model().await?;

        let client = crate::llm_client::LlmClient::new(
            &model.access_url,
            &model.api_key,
            &model.name,
        );

        let mut full_response = String::new();

        let stream = client.chat_stream(messages, None).await?;

        tokio::pin!(stream);

        while let Some(response) = stream.next().await {
            match response {
                Ok(resp) => {
                    full_response.push_str(&resp.content);

                    if let Some(tx) = output_tx.read().await.as_ref() {
                        let _ = tx.send(resp.content.clone()).await;
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
