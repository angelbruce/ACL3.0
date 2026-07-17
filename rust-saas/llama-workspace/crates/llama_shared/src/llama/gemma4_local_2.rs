use axum::{
    extract::State,
    response::{sse::{Event, Sse}, IntoResponse},
    routing::post,
    Json, Router,
};
use futures::{stream, Stream, StreamExt};
use std::{collections::HashMap, pin::Pin, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use serde::{Deserialize, Serialize};

// ==========================================
// 1. 模型定义 (Mocking your existing models)
// ==========================================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LlmRequest {
    pub model_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    pub session: UserSession,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserSession {
    pub user_id: i64,
    pub project_id: i64,
    pub flow_id: i64,
    pub workspace_id: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StreamResponse {
    pub content: String,
    pub done: bool,
}

// 用于在异步 Handler 和同步 Worker 之间传递指令
pub enum LlmCommand {
    ChatStream {
        messages: Vec<ChatMessage>,
        // 使用 oneshot 发送一个通道，让 Worker 把结果流塞进去
        response_tx: mpsc::Sender<StreamResponse>,
    },
}

// ==========================================
// 2. 底层同步 Wrapper (隔离 C 指针)
// ==========================================

// 模拟你的 LLMLlamaWrapper
pub struct LLMLlamaWrapper {
    // 这里模拟包含 *mut i8 的 LlamaBatch 等
    pub model_path: String,
}

impl LLMLlamaWrapper {
    pub fn new(path: String) -> Self {
        Self { model_path: path }
    }

    // 【关键】这是一个纯同步函数，没有任何 async 关键字
    // 它在专门的线程中运行，指针的生命周期只在这个函数内
    pub fn run_sync_chat(
        &mut self,
        messages: Vec<ChatMessage>,
        response_tx: mpsc::Sender<StreamResponse>,
    ) -> Result<(), String> {
        // 模拟 LlamaBatch 的使用过程
        // 在真实的实现中，这里会创建 LlamaBatch, 进行 decode 等
        // 由于这是同步函数，编译器不会在这里生成 async 状态机
        
        for i in 0..5 {
            // 模拟生成内容
            let res = StreamResponse {
                content: format!("Token chunk {} ", i),
                done: i == 4,
            };
            
            // 通过通道把数据发回异步世界
            // 注意：在同步线程中使用 tokio 的 mpsc 需要用 blocking_send
            if let Err(_) = response_tx.blocking_send(res) {
                return Err("Channel closed".to_string());
            }
            
            // 模拟耗时操作
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        Ok(())
    }
}

// ==========================================
// 3. Worker 线程 (Actor 模式的核心)
// ==========================================

pub struct LlmWorker {
    receiver: mpsc::Receiver<LlmCommand>,
    wrapper: LLMLlamaWrapper,
}

impl LlmWorker {
    pub fn spawn(path: String) -> mpsc::Sender<LlmCommand> {
        let (tx, rx) = mpsc::channel(32);
        let mut worker = LlmWorker {
            receiver: rx,
            wrapper: LLMLlamaWrapper::new(path),
        };

        // 启动一个专门的同步线程
        std::thread::spawn(move || {
            while let Some(cmd) = worker.receiver.blocking_recv() {
                match cmd {
                    LlmCommand::ChatStream { messages, response_tx } => {
                        // 在这个线程里，我们可以安全地使用所有非 Send 的 C 指针
                        let _ = worker.wrapper.run_sync_chat(messages, response_tx);
                    }
                }
            }
        });

        tx
    }
}

// ==========================================
// 4. Axum 服务层 (LlmGemma4Local)
// ==========================================

#[derive(Clone)]
pub struct LlmGemma4Local {
    // Handler 持有的不再是 Wrapper，而是一个指向 Worker 的指令通道
    // 这使得 LlmGemma4Local 是完全 Send + Sync 的
    worker_tx: mpsc::Sender<LlmCommand>,
}

impl LlmGemma4Local {
    pub fn new(path: String) -> Self {
        Self {
            worker_tx: LlmWorker::spawn(path),
        }
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> impl Stream<Item = Result<Event, std::convert::Infallible>> {
        let (res_tx, mut res_rx) = mpsc::channel(100);

        // 1. 向 Worker 发送请求
        let cmd = LlmCommand::ChatStream {
            messages,
            response_tx: res_tx,
        };
        
        if let Err(e) = self.worker_tx.send(cmd).await {
            eprintln!("Failed to send command to worker: {}", e);
        }

        // 2. 将接收到的消息转化为 SSE Event 流
        // 因为 res_rx 是在异步环境下监听的，所以这里是安全的
        async_stream::stream! {
            while let Some(res) = res_rx.recv().await {
                let json_str = serde_json::to_string(&res).unwrap_or_default();
                yield Ok(Event::default().data(json_str));
            }
        }
    }
}

// ==========================================
// 5. Axum Route & Handler
// ==========================================

pub async fn chat_handler(
    State(llm): State<Arc<LlmGemma4Local>>,
    Json(req): Json<LlmRequest>,
) -> impl IntoResponse {
    // 构造模拟消息
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: Some("Hello".to_string()),
        session: UserSession { user_id: 1, project_id: 1, flow_id: 1, workspace_id: 1 },
    }];

    // 调用 stream
    let stream = llm.chat_stream(messages).await;
    
    Sse::new(stream).into_response()
}

#[tokio::main]
async fn main() {
    let llm_service = Arc::new(LlmGemma4Local::new("path/to/model.gguf".to_string()));

    let app = Router::new()
        .route("/api/chat/stream", post(chat_handler))
        .with_state(llm_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
