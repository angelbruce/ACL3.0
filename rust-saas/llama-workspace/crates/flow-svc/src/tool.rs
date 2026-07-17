// use futures::{Stream, StreamExt};
// use reqwest::{Client, RequestBuilder};
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::pin::Pin;
// use shared::errors::{ServiceError, ServiceResult};
// use shared::models::{MCPTool, LlmTool, LlmToolFunction, ToolCallInfo, ToolCallFunction};
// use crate::agent_repository::AgentRepository;
// use serde_json::*;

// pub struct ToolExecutor {
//     servers: HashMap<i64, String>,
//     default_server_url: String,
//     client: Client,
//     repo: AgentRepository,
//     agent_id: i64,
//     user_id: i64,
// }

// impl ToolExecutor {
//     pub fn new(user_id:i64, agent_id:i64, repo:AgentRepository, servers: HashMap<i64, String>, default_server_url: &str) -> Self {
//         ToolExecutor {
//             user_id:user_id,
//             agent_id:agent_id,
//             repo:repo,
//             servers:servers,
//             default_server_url: default_server_url.to_string(),
//             client: Client::new(),
//         }
//     }

//     pub fn get_user_id(&self) -> i64 {
//         self.user_id
//     }

//     pub fn get_agent_id(&self) -> i64 {
//         self.agent_id
//     }

//     pub fn get_server_url(&self, server_id: Option<i64>) -> String {
//         match server_id {
//             Some(id) => self.servers.get(&id)
//                 .cloned()
//                 .unwrap_or_else(|| self.default_server_url.clone()),
//             None => self.default_server_url.clone(),
//         }
//     }

//     pub async fn list_tools(&self)-> ServiceResult<Vec<MCPTool>> {
//         let tools =  self.repo.get_agent_tools(self.agent_id).await?;

//         return Ok(tools);
//     }

//     pub async fn execute_tool(&self, name: &str, arguments: &serde_json::Value, server_id: Option<i64>) -> ServiceResult<String> {
//         let server_url = self.get_server_url(server_id);
//         let url = format!("{}", server_url);
//         let request = serde_json::json!({
//             "id": 1,
//             "method": "tools/call",
//             "jsonrpc": "2.0",
//             "params" : {
//                 "name": name.to_string(),
//                 "arguments": arguments.clone(),
//             },
//         });
      
//         let response = self.client
//             .post(&url)
//             .json(&request)
//             .send()
//             .await
//             .map_err(|e| ServiceError::McpError(e.to_string()))?;

//         // MCP 返回的是 SSE 流式格式，需要逐行解析
//         let body_bytes = response.bytes()
//             .await
//             .map_err(|e| ServiceError::McpError(e.to_string()))?;
//         let body_str = String::from_utf8_lossy(&body_bytes);

//         println!("[ExecuteTool] Raw response body: {}", body_str);

//         // 解析 SSE 格式：每行以 "data: " 开头
//         let mut content = String::new();
//         for line in body_str.lines() {
//             let trimmed = line.trim();
//             if let Some(data) = trimmed.strip_prefix("data: ") {
//                 // 跳过 [DONE] 标记
//                 if data == "[DONE]" {
//                     continue;
//                 }
//                 if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
//                     // 从 MCP JSON-RPC 响应中提取 content
//                     // 格式: {"result": {"content": [{"type":"text","text":"..."}]}}
//                     if let Some(result) = value.get("result") {
//                         if let Some(content_list) = result.get("content") {
//                             if let Some(arr) = content_list.as_array() {
//                                 for item in arr {
//                                     if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
//                                         if !content.is_empty() {
//                                             content.push('\n');
//                                         }
//                                         content.push_str(text);
//                                     }
//                                 }
//                             }
//                         }
//                         // 检查是否有 error
//                         if let Some(error) = result.get("isError") {
//                             if error.as_bool() == Some(true) {
//                                 return Err(ServiceError::McpError(
//                                     format!("Tool execution returned error: {}", content)
//                                 ));
//                             }
//                         }
//                     }
//                     // 也检查顶层 error
//                     if let Some(err) = value.get("error") {
//                         let err_msg = err.get("message")
//                             .and_then(|m| m.as_str())
//                             .unwrap_or("Unknown error");
//                         return Err(ServiceError::McpError(err_msg.to_string()));
//                     }
//                 }
//             }
//         }

//         if content.is_empty() {
//             content = "Tool execution completed".to_string();
//         }
//         Ok(content)
//     }
// }