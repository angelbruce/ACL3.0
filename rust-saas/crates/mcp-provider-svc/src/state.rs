use std::sync::Arc;
use tokio::sync::broadcast;
use crate::tools_handler::McpToolsHandler;

#[derive(Clone)]
pub struct AppState {
    /// 广播通道发送端：POST 收到的消息由此发出，SSE 连接订阅此通道
    pub tx: broadcast::Sender<String>,
    /// MCP Handler 实例
    pub handler: Arc<McpToolsHandler>,
}

impl AppState {
    pub fn new() -> Self {
        // buffer size 1024，可根据并发调整
        let (tx, _rx) = broadcast::channel::<String>(1024);
        Self {
            tx,
            handler: Arc::new(McpToolsHandler::new()),
        }
    }
}
