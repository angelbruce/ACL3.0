use shared::errors::{ServiceError, ServiceResult};
use shared::models::*;
use llama_shared::llama::*;
use llama_shared::llama::common::*;
use async_stream::stream;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use futures_core::stream::Stream;
use shared::schema::agent_skills::agent_id;

use crate::repository::FlowRepository;
use crate::agent_repository::AgentRepository;
use crate::state_machine::FlowStateMachine;
// use crate::tool::ToolExecutor;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::sync::Mutex;

use chrono::Utc;

#[derive(Clone)]
pub struct FlowExecutor {
    user_id: i64,
    flow_id: i64,
    runtime_id: i64,
    repo: Arc<FlowRepository>,
    state_machine: Arc<FlowStateMachine>,
    output_tx: Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>,
    node_agents: Arc<RwLock<HashMap<i64, mpsc::Sender<NodeAgentMsg>>>>,
    human_node_channels: Arc<RwLock<HashMap<i64, mpsc::Sender<NodeAgentMsg>>>>,
    running: Arc<RwLock<bool>>,
    session_id: Arc<RwLock<Option<i64>>>,
}

impl FlowExecutor {
    pub fn new(user_id:i64, flow_id: i64, runtime_id: i64, repo: FlowRepository) -> Self {
        let repo = repo.clone();
        let state_machine = FlowStateMachine::new(repo.clone());
        
        FlowExecutor {
            user_id,
            flow_id,
            runtime_id,
            repo: Arc::new(repo),
            state_machine: Arc::new(state_machine),
            output_tx: Arc::new(RwLock::new(None)),
            node_agents: Arc::new(RwLock::new(HashMap::new())),
            human_node_channels: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            session_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> ServiceResult<()> {
        let mut is_running = self.running.write().await;
        if *is_running {
            return Err(ServiceError::Conflict("Flow is already running".to_string()));
        }
        *is_running = true;
        drop(is_running);

        let flow = self.repo.get_flow(self.flow_id).await?;
        let config: FlowConfigModel = match serde_json::from_value(flow.config.clone()) {
                                        Ok(c) => c,
                                        Err(e) => {
                                            tracing::error!("Failed to parse flow config: {:?}", e);
                                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                            return Err(ServiceError::InternalError);
                                        }
                                    };


        let over_nodes = config.vertices.iter().filter(|x| x.r#type == "end" || x.r#type == "over")
            .map(|x| (x.id.clone(),x.value.clone())).collect::<HashMap<String,String>>();

            let c = over_nodes.clone();
            println!("-------------------------------");
        for (k,v) in c.iter() {
            println!("{}\t{}",k.clone(),v.clone());
        }


        //启动会话
        self.create_session().await?;

        let nodes = self.repo.get_flow_runtime_nodes(self.runtime_id).await?;
        let running_nodes: Vec<_> = nodes.into_iter()
            .filter(|n| n.status == NodeStatus::Running.to_string())
            .collect();

        if running_nodes.is_empty() {
            self.stop_internal().await?;
            return Ok(());
        }

        // AI 管道，TX用来写入AI消息，RX与用来接收消息给NODE_AGENTS
        let (tx, mut rx) = mpsc::channel::<NodeAgentMsg>(100);
        *self.output_tx.write().await = Some(tx);

        let node_agents = self.node_agents.clone();
        // AGENT-NODES将消息通过PTX写入让PRX读取，就是AI消息中转，注意，PRX/PTX只属于当前运行节点
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let id = msg.runtime_node_id;
                if id < 0 {
                    continue;
                }

                if id == 0 {
                    for (_, tx) in node_agents.read().await.iter() {
                        let _ = tx.send(msg.clone()).await;
                    }
                    continue;
                }

                if !node_agents.read().await.contains_key(&id) {
                    continue;
                }
                
                if let Some(tx) = node_agents.read().await.get(&id) {   
                    let _ = tx.send(msg.clone()).await;
                }
            }
        });

        let runtime_id = self.runtime_id;
        let flow_id = self.flow_id;
        let repo = self.repo.clone();
        let state_machine = self.state_machine.clone();
        let running = self.running.clone();
        let output_tx = self.output_tx.clone();
        let node_agents_clone = self.node_agents.clone();
        let human_node_channels_clone = self.human_node_channels.clone();
        let session_id_clone = self.session_id.clone();
        let uid = self.user_id.clone();

        let ( shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let shutdown_sender = Arc::new(Mutex::new(Some(shutdown_tx)));
        //主循环

        let repo = repo.clone();
        tokio::spawn(async move {
            let mut consecutive_no_new = 0;
            let mut nodes =  match repo.get_flow_runtime_nodes(runtime_id).await {
                        Ok(t)=>t,
                        Err(_) => vec![]
                    };
            let mut running_nodes: Vec<_> = nodes.into_iter()
                    .filter(|n| n.status == NodeStatus::Running.to_string())
                    .collect();

            let repo = repo.clone();
            let mut changed = false;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let repo = repo.clone();
                if changed {
                    nodes = match repo.get_flow_runtime_nodes(runtime_id).await {
                        Ok(t)=>t,
                        Err(_) => vec![]
                    };

                    running_nodes = nodes.into_iter()
                        .filter(|n| n.status == NodeStatus::Running.to_string())
                        .collect();
                }

                
                let handle = async {
                    if !*running.read().await {
                        return false;
                    }

                    //存在运行节点，此处判断疑似存在问题
                    let mut has_running_agent = false;
                    for node in &running_nodes {
                        if node_agents_clone.read().await.contains_key(&node.id) {
                            has_running_agent = true;
                            break;
                        }
                    }

                    if has_running_agent {
                        println!("--has_running_agent--");
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        return true;
                    }
                    

                    // 得到运行时产生的node集合
                    let nodes = match repo.get_flow_runtime_nodes(runtime_id).await {
                        Ok(n) => n,
                        Err(e) => {
                            tracing::error!("Failed to get runtime nodes: {:?}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            return true;
                        }
                    };

                    //活动节点
                    println!("--running_nodes--");
                    let running_nodes: Vec<_> = nodes.into_iter()
                        .filter(|n| n.status == NodeStatus::Running.to_string())
                        .collect();

                    //检测5次（5*1s）后，依然没有产生新的节点，停止流程冰退出。
                    if running_nodes.is_empty() {
                        consecutive_no_new += 1;
                        if consecutive_no_new >= 5 {
                            let _ = repo.stop_flow_runtime(runtime_id).await;
                            println!("--running_nodes over--");
                            return false;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        return true;
                    }

                    println!("--running_nodes 111--");
                    consecutive_no_new = 0;

                    //流程退出条件判断
                    println!("--is_over--");
                    let is_over = running_nodes.iter().any(|x| over_nodes.contains_key(&x.flow_node_id));
                    if is_over {
                        println!("--is_over  1--");
                        if let Some(tx_option) = shutdown_sender.lock().unwrap().take() {
                            // 2. 发送信号
                            println!("3");
                            
                      
                            let send_rest = tx_option.send(());
                            match send_rest {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("send error ");
                                }
                            }

                        }

                        return false;
                    }

                    //为什么是第一个,应该是全部
                    for node in &running_nodes {
                        let node = node.clone();
                        tracing::info!("Starting node {} (flow_node_id={})", node.id, node.flow_node_id);

                        //为节点分配管道接收器和人交互管道
                        let (ptx, prx) = mpsc::channel::<NodeAgentMsg>(1000000);
                        let (hptx, hprx) = mpsc::channel::<NodeAgentMsg>(1000000);
                        human_node_channels_clone.write().await.insert(node.id, hptx);
                        node_agents_clone.write().await.insert(node.id, ptx);

                        let pn_id = node.flow_node_id.clone();
                        let pn = node.clone();
                        let rc = repo.clone();
                        let smc = state_machine.clone();
                        let runc = running.clone();
                        let otxc = output_tx.clone();
                        
                        //流程
                        let pflow = match rc.get_flow(flow_id).await {
                            Ok(f) => f,
                            Err(e) => {
                                tracing::error!("Failed to get flow: {:?}", e);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                continue;
                            }
                        };
                        
                        //流程配置
                        let pflow_config: FlowConfigModel = match serde_json::from_value(pflow.config.clone()) {
                            Ok(c) => c,
                            Err(e) => {
                                tracing::error!("Failed to parse flow config: {:?}", e);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                continue;
                            }
                        };

                        //会话
                        let session_id = match *session_id_clone.read().await {
                            Some(sid) => sid,
                            None => {
                                tracing::error!("Session not created for runtime {}", runtime_id);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                continue;
                            }
                        };

                        // 启动运行节点作业
                        let fu = tokio::spawn(async move {
                            let phuman = pn.human == 1;
                            let mut pagent = NodeAgent::new(
                                uid,
                                pn.id,
                                runtime_id,
                                flow_id,
                                pn_id,
                                pn.action_id,
                                pn.action.clone(),
                                pn.prompt.clone(),
                                rc.clone(),
                                pflow_config,
                                phuman,
                                session_id,
                            );

                            let mut p_next_choice: Option<String> = None;
                            if let Err(e) = pagent.run(prx, otxc, hprx, &mut p_next_choice).await {
                                tracing::error!("Node agent {} error: {:?}", pn.id, e);
                            }

                            if let Err(e) = smc.complete_node(runtime_id, pn.id, p_next_choice).await {
                                tracing::error!("Failed to complete node {}: {:?}", pn.id, e);
                            }

                            if let Err(e) = rc.update_flow_runtime_node(pn.id, NodeStatus::Stop).await {
                                tracing::error!("Failed to stop node {}: {:?}", pn.id, e);
                            }

                            node.id
                            // node_agents_clone.write().await.remove(node.id);
                        }); //END NODE WORK SPAWN

                        let data  = fu.await;
                        match data {
                            Ok(id) => {
                                node_agents_clone.write().await.remove(&id);
                                changed = true;
                            },
                            Err(_) => {}
                        }

                    } //END WHILE

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                    true
                }; // end async

                let should_continue = tokio::select! {
                // 等待关闭信号：如果外部调用者发送了信号，流程停止
                    _ = &mut shutdown_rx => {
                        tracing::info!("流程执行器接收到外部终止信号，停止运行。");
                       
                        false
                    }

                    result = handle => {
                        result
                    }
                };

                if !should_continue {
                    break;
                }
            }


            for (_, tx) in node_agents_clone.write().await.drain() {
                let _ = tx.send(NodeAgentMsg::new(0, "__stop__".to_string())).await;
            }


            let _ = repo.stop_flow_runtime(runtime_id).await;
        });

        tracing::info!("Flow executor {} started successfully", self.flow_id);

        Ok(())
    }

    
    /// 手动停止流程
    pub async fn stop(&self) -> ServiceResult<()> {
        let mut is_running = self.running.write().await;
        if !*is_running {
            return Ok(());
        }
        *is_running = false;
        drop(is_running);

        self.stop_internal().await
    }

    /// 停止流程、发送停止信号
    async fn stop_internal(&self) -> ServiceResult<()> {
        for (_, tx) in self.node_agents.write().await.drain() {
            let _ = tx.send(NodeAgentMsg::new(0, "__STOP__".to_string())).await;
        }
        
        let _ = self.repo.stop_flow_runtime(self.runtime_id).await;
        Ok(())
    }

    /// 创建会话
    async fn create_session(&self) -> ServiceResult<()> {
        let now = Utc::now().naive_utc();
        let session = crate::model::FlowRuntimeSession {
            id: 0,
            flow_id: self.flow_id,
            flow_runtime_id: self.runtime_id.to_string(),
            creator_id: 0,
            created_at: now,
            updated_at: now,
        };
        let created = self.repo.insert_flow_runtime_session(&session).await?;
        *self.session_id.write().await = Some(created.id);
        tracing::info!("Flow runtime {} session created: {}", self.runtime_id, created.id);
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

    pub async fn send_human_input(&self, node_id: i64, message: &str) -> ServiceResult<()> {
        if let Some(tx) = self.human_node_channels.read().await.get(&node_id) {
            let msg = NodeAgentMsg {
                runtime_node_id: node_id,
                msg: message.to_string(),
                msg_type: NodeAgentMsgType::Human,
            };
            tx.send(msg).await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;
            Ok(())
        } else {
            Err(ServiceError::NotFound)
        }
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

pub struct NodeEdge {
    pub from: String,
    pub to: String,
    pub value: String,
}

pub struct NodeAgent {
    user_id: i64,
    runtime_node_id: i64,
    runtime_id: i64,
    flow_id: i64,
    node_id: String,
    action_id: i64,
    action: String,
    prompt: Option<String>,
    human: bool,
    repo: Arc<FlowRepository>,
    edges: Vec<NodeEdge>,
    session_id: i64,
}

impl NodeAgent {
    pub fn new(
        user_id: i64,
        runtime_node_id: i64,
        runtime_id: i64,
        flow_id: i64,
        node_id: String,
        action_id: i64,
        action: String,
        prompt: Option<String>,
        repo: Arc<FlowRepository>,
        flow_config: FlowConfigModel,
        human: bool,
        session_id: i64,
    ) -> Self {
        let edges = flow_config.edges.iter()
            .filter(|e| e.src == node_id)
            .map(|e| NodeEdge {
                from: e.src.clone(),
                to: e.target.clone(),
                value: e.value.clone(),
            })
            .collect();

        NodeAgent {
            user_id,
            runtime_node_id,
            runtime_id,
            flow_id,
            node_id,
            action_id,
            action,
            prompt,
            human,
            repo,
            edges,
            session_id,
        }
    }

    /// 启动流程节点
    pub async fn run(
        &mut self,
        mut input_rx: mpsc::Receiver<NodeAgentMsg>,
        output_tx: Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>, 
        mut human_input_rx: mpsc::Receiver<NodeAgentMsg>,
        next_choice: &mut Option<String>,
    ) -> ServiceResult<()> {
 

        let mut messages: Vec<shared::models::ChatMessage> = Vec::new();

        let human_data = self.repo.get_flow_runtime_node_human(self.runtime_node_id).await?;
        self.human = human_data == 1;

        // TODO! important !!!!
        // 节点之间的输入信息会作为上下文输入到下一步流程中，很重要。
        // 所以要求节点执行的时候产生输出记录，下一步节点只读取产生出的关联记录即可,减少上下文量的大小。
        self.load_context(&mut messages).await?;

        messages.push(shared::models::ChatMessage{
            role: "system".to_string(),
            content: Some(format!("**UserId**：{}\n**FlowRuntimeId:**{}\n其他ID为0", self.user_id,self.runtime_id)),
            ..Default::default()
        });

        let repo = crate::agent_repository::AgentRepository::new(self.repo.get_pool().clone());
        let servers =  repo.get_mcp_servers().await?;
        let mut server_map = HashMap::new();
        for server in servers {
            server_map.insert(server.id,server.url);
        }

        let mut tool_executor = ToolExecutor::new(server_map,"");
        if self.action_id > 0 {
            if let Ok(agent) = repo.get_agent_by_id(self.action_id).await {
                if let Some(def) = agent.defination {
                    messages.push(shared::models::ChatMessage{
                        role: "system".to_string(),
                        content: Some(def),
                        ..Default::default()
                    });
                }
            }
        }

        if let Some(ref prompt) = self.prompt {
            messages.push(shared::models::ChatMessage {
                role: "system".to_string(),
                content: Some(prompt.clone()),
                ..Default::default()
            });
        }

        if !self.edges.is_empty() {
            let strategy = "**决策支撑**:你需要做出行动以支持以下决策，每个决策都有一个名称。\n".to_string();
            let edge_paths = self.edges.iter()
                .map(|e| format!("决策名称：`{}`", e.value))
                .collect::<Vec<String>>().join("\n");
            let prompts = format!("{}{}\n必须在输出结尾中从决策选项中选择一项进行输出，输出格式`next <决策名称>`或者`__stop__`，例如： `next 下一步`，**一定要用 `符号将决策输出包围起来，决策输出必须在输出末尾输出，并且末尾不能出现任何其它内容！**", strategy, edge_paths);

            println!("AGENT PROMPTS STRATEGY: {}", prompts.clone());
            messages.push(shared::models::ChatMessage {
                role: "system".to_string(),
                content: Some(prompts),
                ..Default::default()
            });
        }

        messages.push(shared::models::ChatMessage {
            role: "system".to_string(),
            // content: Some("**任务约束**：如果已经完成所有任务请输出````__stop__````，如果需要人参与，请输出````__human__````".to_string()), 
            content: Some("**任务约束**：如果已经完成所有任务请输出`__stop__`".to_string()), 
            ..Default::default()
        });

        messages.push(shared::models::ChatMessage {
            role: "system".to_string(),
            content: Some(format!("**任务目标**：{}", self.action)),
            ..Default::default()
        });

        if self.action_id == 0 && self.human {
            if let Some(data) = human_input_rx.recv().await {
                let role = match data.msg_type {
                    NodeAgentMsgType::AI => "assistant".to_string(),
                    NodeAgentMsgType::Human => "user".to_string(),
                };
                messages.push(shared::models::ChatMessage {
                    role: role.clone(),
                    content: Some(data.msg.clone()),
                    ..Default::default()
                });
                self.save_message(role, data.msg).await?;
            }
            self.repo.update_flow_runtime_node_human(self.runtime_node_id, 0).await?;
            return Ok(());
        }

        let mut first_iteration = true;
        
        loop {
            if !first_iteration {
                let mut recv_msg: Option<NodeAgentMsg> = None;
                if self.human {
                    //接收人的管道消息
                    if let Some(data) = human_input_rx.recv().await {
                        recv_msg = Some(data);
                    }
                    self.repo.update_flow_runtime_node_human(self.runtime_node_id, 0).await?;
                    self.human = false;
                } else {
                    //接送tx发送的管道消息

                    //统一收集管道消息，AI消息多为碎片，此处统一收集
                    let mut msg_buf = String::new();
                    while true {
                        tokio::select! {
                            Some(msg) = input_rx.recv() => {
                                msg_buf.push_str(msg.msg.as_str());
                                recv_msg = Some(msg);
                            }
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                                break;
                            }
                        }
                    }

                    match recv_msg {
                        Some(ref mut data) =>{
                            data.msg = msg_buf.clone();
                        }
                        None=>{}
                    }
                }

                //收到消息，执行存储。
                if let Some(msg) = recv_msg {
                    if msg.runtime_node_id == 0 || msg.msg == "__stop__" {
                        break;
                    }

                    if msg.msg.contains("__stop__") {
                        break;
                    }

                    if msg.msg.contains("__human__") {
                        self.human = true;
                        self.repo.update_flow_runtime_node_human(self.runtime_node_id, 1).await?;
                        self.save_message("system".to_string(), "请求人工参与".to_string()).await?;
                        continue;
                    }

                    let role = match msg.msg_type {
                        NodeAgentMsgType::AI => "assistant".to_string(),
                        NodeAgentMsgType::Human => "user".to_string(),
                    };

                    messages.push(shared::models::ChatMessage {
                        role: role.clone(),
                        content: Some(msg.msg.clone()), 
                        ..Default::default()
                    });

                    messages.push(shared::models::ChatMessage{
                        role: "user".to_string(),
                        content: Some("请继续执行任务，直到本次任务目标全部完成，同时必须做出下一步决策，决策输出必须严格按照**决策支撑**规则输出决策。".to_string()),
                        ..Default::default()
                    });

                    self.save_message(role, msg.msg).await?;
                }
            } else {
                first_iteration = false;
            }
        
            // 需要人交互，读取人管道中的消息
            if self.human {
                continue;
            }

            // 没有agent执行，也退出
            if self.action_id == 0 {
                break;
            }

            let mut full_response = String::new();

            // 采用工具执行作业
            let tools = repo.get_agent_tools(self.action_id).await?;
            let result = self.call_llm(&tool_executor, &mut messages, &output_tx, &mut full_response, tools).await;
            if let Err(e) = result {
                tracing::error!("LLM error: {:?}", e);
                break;
            }

            println!("LLM Response: {}", full_response.clone());

            let response_trimmed = full_response.trim();
            if response_trimmed.is_empty() {
                tracing::info!("LLM returned empty response, node completed");
                break;
            }

            //保存产生的会话信息
            self.save_message("assistant".to_string(), full_response.clone()).await?;

            // 如果已经产生了决策，则本节点执行完成，退出节点任务执行，进行流程下一环节
            if let Some(choice) = self.parse_decision(&full_response) {
                *next_choice = Some(choice);
                break;
            }

            // 如果流程节点完成，则直接退出
            if full_response.contains("__stop__") {
                break;
            }

            // 需要人来参与执行
            if full_response.contains("__human__") {
                self.human = true;
                self.repo.update_flow_runtime_node_human(self.runtime_node_id, 1).await?;
                self.save_message("system".to_string(), "请求人工参与".to_string()).await?;
                continue;
            }

        }

        Ok(())
    }

    /// 读取历史产生的会话消息
    /// TODO! important !!!!
    /// 节点之间的输入信息会作为上下文输入到下一步流程中，很重要。
    /// 所以要求节点执行的时候产生输出记录，下一步节点只读取产生出的关联记录即可,减少上下文量的大小。
    async fn load_context(&self, messages: &mut Vec<shared::models::ChatMessage>) -> ServiceResult<()> {
        let session_items = self.repo.get_flow_runtime_session_items_by_flow_runtime_id(self.runtime_id).await?;
        for item in session_items {
            messages.push(shared::models::ChatMessage {
                role: item.session_type.clone(),
                content: Some(item.content),
                ..Default::default()
            });
        }
        Ok(())
    }

    // 保存消息到会话中
    async fn save_message(&self, msg_type: String, content: String) -> ServiceResult<()> {
        tracing::info!("save_message called: session_id={}, msg_type={}, content_len={}", 
            self.session_id, msg_type, content.len());
        let now = Utc::now().naive_utc();
        let item = crate::model::FlowRuntimeSessionItem {
            id: 0,
            flow_id: self.flow_id,
            flow_runtime_id: self.runtime_id.to_string(),
            flow_runtime_session_id: self.session_id,
            flow_runtime_node_id: self.runtime_node_id.to_string(),
            session_type: msg_type,
            content,
            action_id: self.action_id,
            created_at: now,
            creator_id: 0,
        };

        match self.repo.insert_flow_runtime_session_item(&item).await {
            Ok(created) => {
                tracing::info!("save_message succeeded: session_item_id={}", created.id);
            }
            Err(e) => {
                tracing::error!("save_message failed: {:?}", e);
            }
        }

        Ok(())
    }

    /// 抽取决策信息
    fn parse_decision(&self, response: &str) -> Option<String> {
        let re = regex::Regex::new(r"`next\s+([^`]+)`").ok()?;
        if let Some(captures) = re.captures(response) {
            let decision = captures[1].trim().to_string();
            let matching_edges: Vec<_> = self.edges.iter()
                .filter(|e| e.value == decision)
                .collect();
            
            if matching_edges.is_empty() {
                return None;
            }
            
            if matching_edges.len() == 1 {
                return Some(matching_edges[0].to.clone());
            }

            let idx = rand::random::<usize>() % matching_edges.len();
            return Some(matching_edges[idx].to.clone());
        } else {
           for line in   response.lines().rev() {
                let trim_line = line.trim();
                if trim_line.is_empty() {
                    continue;
                }

                if let Some(data) = trim_line.strip_prefix("next") {
                    if  data.is_empty() {
                        return None;
                    } else {
                        return Some(data.to_string())
                    }
                }
           }
        }
        
        None
    }


    /// 调用AI模型，执行作业，产生并发送交互记录
    async fn call_llm(
        &self,
        tool_executor: &ToolExecutor,
        messages: &mut Vec<ChatMessage>,
        output_tx: &Arc<RwLock<Option<mpsc::Sender<NodeAgentMsg>>>>,
        full_response: &mut String,
        tools: Vec<MCPTool>
    ) -> ServiceResult<()> {
        let model_repo = crate::repository::ModelRepository::new();
        let model = model_repo.get_default_model().await?;
    
        let mut client = if model.access_url.clone().is_empty() {
            LlmProxy::for_local(None,None)
        } else {
            LlmProxy::for_openai(model.access_url.to_string(),model.api_key.to_string(),model.name.to_string())
        };
        
        let stream = client.chat_stream(tool_executor, messages, Some(&tools)).await;

        tokio::pin!(stream);

        while let Some(response) = stream.next().await {
            match response {
                Ok(resp) => {
                    full_response.push_str(&resp.content);
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
