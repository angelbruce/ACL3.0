// 该文件是从 llm-svc/src/client.rs 复制过来的逻辑，
// 避免修改 llm-svc 已有的代码。
// 这里将 LlmClient、ToolExecutor、StreamResponse 等类型复制到 workspace-svc，
// 以便在容器部署流程中直接调用 LLM 并执行 MCP 工具。

use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;
use std::pin::Pin;
use std::path::PathBuf;
use tokio::process::Command;
use shared::errors::*;
use shared::models::*;
use llama_shared::llama::*;
use llama_shared::llama::common::*;
use llama_shared::llama::tool_executor::*;

use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use futures_core::stream::Stream;

// /// 工具执行结果（与 llm-svc 保持一致）
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ChatResponse {
//     pub content: String,
//     pub tool_calls: Option<Vec<ToolCallInfo>>,
// }
// /// ToolExecutor：负责将 LLM 返回的工具调用转发到对应的 MCP-SERVER
// pub struct ToolExecutor {
//     servers: HashMap<i64, String>,
//     default_server_url: String,
//     client: Client,
//     debug_dir: Option<PathBuf>,
// }
// impl ToolExecutor {
//     pub fn new(servers: HashMap<i64, String>, default_server_url: &str, debug_dir: Option<PathBuf>) -> Self {
//         ToolExecutor {
//             servers,
//             default_server_url: default_server_url.to_string(),
//             client: Client::new(),
//             debug_dir,
//         }
//     }
//     pub fn get_server_url(&self, server_id: Option<i64>) -> String {
//         match server_id {
//             Some(id) => self.servers.get(&id)
//                 .cloned()
//                 .unwrap_or_else(|| self.default_server_url.clone()),
//             None => self.default_server_url.clone(),
//         }
//     }
//     pub async fn execute_tool(&self, name: &str, arguments: &serde_json::Value, server_id: Option<i64>) -> ServiceResult<String> {
//         if arguments.clone().to_string().contains("docker") {
//             if let Some(ref debug_dir) = self.debug_dir {
//                 let result = Command::new(arguments.to_string())
//                     .current_dir(debug_dir)
//                     // .arg("up")
//                     // .arg("-d")
//                     // .arg("--build")
//                     // .arg("--remove-orphans")
//                     // .arg("--force-recreate")
//                     .spawn()
//                     .map_err(|e| ServiceError::McpError(e.to_string()))?
//                     .wait()
//                     .await
//                     .map_err(|e| ServiceError::McpError(e.to_string()))?;
//                 if result.success() {
//                     return Ok("Tool execution completed".to_string());
//                 } else {
//                     return Err(ServiceError::McpError(format!("Command failed with exit code: {:?}", result.code())));
//                 }
//             } 
//         }
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
//                         if let Some(error) = result.get("isError") {
//                             if error.as_bool() == Some(true) {
//                                 return Err(ServiceError::McpError(
//                                     format!("Tool execution returned error: {}", content)
//                                 ));
//                             }
//                         }
//                     }
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
//     /// 从 MCP-SSE 服务获取工具列表
//     pub async fn list_tools(&self, server_id: Option<i64>) -> ServiceResult<Vec<MCPTool>> {
//         let server_url = self.get_server_url(server_id);
//         let request = serde_json::json!({
//             "id": 1,
//             "method": "tools/list",
//             "jsonrpc": "2.0",
//         });
//         let response = self.client
//             .post(&server_url)
//             .json(&request)
//             .send()
//             .await
//             .map_err(|e| ServiceError::McpError(e.to_string()))?;
//         let body_bytes = response.bytes()
//             .await
//             .map_err(|e| ServiceError::McpError(e.to_string()))?;
//         let body_str = String::from_utf8_lossy(&body_bytes);
//         println!("[ListTools] Raw response body: {}", body_str);
//         // 解析 SSE 格式响应
//         let mut tools: Vec<MCPTool> = Vec::new();
//         for line in body_str.lines() {
//             let trimmed = line.trim();
//             if let Some(data) = trimmed.strip_prefix("data: ") {
//                 if data == "[DONE]" {
//                     continue;
//                 }
//                 if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
//                     if let Some(result) = value.get("result") {
//                         if let Some(tool_list) = result.get("tools") {
//                             if let Some(arr) = tool_list.as_array() {
//                                 for tool_item in arr {
//                                     let name = tool_item.get("name")
//                                         .and_then(|n| n.as_str())
//                                         .unwrap_or("unknown")
//                                         .to_string();
//                                     let description = tool_item.get("description")
//                                         .and_then(|d| d.as_str())
//                                         .unwrap_or("")
//                                         .to_string();
//                                     let input_schema = tool_item.get("inputSchema")
//                                         .cloned()
//                                         .unwrap_or_else(|| serde_json::json!({}));
//                                     let output_schema = tool_item.get("outputSchema")
//                                         .cloned()
//                                         .unwrap_or_else(|| serde_json::json!({}));
//                                     tools.push(MCPTool {
//                                         name,
//                                         description,
//                                         input_schema,
//                                         output_schema,
//                                         server_id,
//                                     });
//                                 }
//                             }
//                         }
//                     }
//                     if let Some(err) = value.get("error") {
//                         let err_msg = err.get("message")
//                             .and_then(|m| m.as_str())
//                             .unwrap_or("Unknown error");
//                         return Err(ServiceError::McpError(err_msg.to_string()));
//                     }
//                 }
//             }
//         }
//         println!("[ListTools] Found {} tools", tools.len());
//         Ok(tools)
//     }
// }
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct StreamResponse {
//     pub content: String,
//     pub reasoning_content: Option<String>,
//     pub tool_calls: Option<Vec<ToolCallInfo>>,
//     pub finish_reason: Option<String>,
// }

pub struct LlmClient {
    base_url: String,
    api_key: String,
    model_name: String,
    client: Client,
    proxy: LlmProxy,
    tool_executor: ToolExecutor
}

impl LlmClient {
    pub fn new(servers: HashMap<i64, String>, base_url: &str, api_key: &str, model_name: &str) -> Self {
        let tool_executor = ToolExecutor::new(servers,base_url);

        let proxy = if base_url.clone().starts_with("127.0.0.1") {
            LlmProxy::for_local(None,None)
        } else {
            LlmProxy::for_openai(base_url.to_string(),api_key.to_string(),model_name.to_string())
        };

        LlmClient {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model_name: model_name.to_string(),
            client: Client::new(),
            proxy: proxy,
            tool_executor: tool_executor
        }
    }

    // pub async fn chat(&self, messages: &[ChatMessage], tools: Option<&[MCPTool]>) -> ServiceResult<ChatResponse> {
    //     let llm_tools = tools.map(|mcp_tools| {
    //         mcp_tools.iter().map(|tool| LlmTool {
    //             r#type: "function".to_string(),
    //             function: LlmToolFunction {
    //                 name: tool.name.clone(),
    //                 description: tool.description.clone(),
    //                 parameters: tool.input_schema.clone(),
    //             },
    //         }).collect::<Vec<LlmTool>>()
    //     });
    //     let body = ChatCompletionRequest {
    //         model: self.model_name.clone(),
    //         messages: messages.iter().cloned().collect(),
    //         tools: llm_tools,
    //         stream: Some(false),
    //         max_tokens: Some(4096),
    //         temperature: Some(0.7),
    //     };
    //     let response = self.build_request()
    //         .json(&body)
    //         .send()
    //         .await
    //         .map_err(|e| ServiceError::LlmError(e.to_string()))?;
    //     let result: ChatCompletionResponse = response.json()
    //         .await
    //         .map_err(|e| ServiceError::LlmError(e.to_string()))?;
    //     Ok(ChatResponse {
    //         content: result.choices.first()
    //             .and_then(|c| c.message.content.clone())
    //             .unwrap_or_default(),
    //         tool_calls: result.choices.first()
    //             .and_then(|c| c.message.tool_calls.clone()),
    //     })
    // }
    // pub async fn chat_stream(&self, messages: &[ChatMessage], tools: Option<&[MCPTool]>) -> ServiceResult<impl Stream<Item = StreamResponse>> {
    //     let llm_tools = tools.map(|mcp_tools| {
    //         mcp_tools.iter().map(|tool| LlmTool {
    //             r#type: "function".to_string(),
    //             function: LlmToolFunction {
    //                 name: tool.name.clone(),
    //                 description: tool.description.clone(),
    //                 parameters: tool.input_schema.clone(),
    //             },
    //         }).collect::<Vec<LlmTool>>()
    //     });
    //     println!("tools: {:?}", llm_tools.clone());
    //     let messages: Vec<ChatMessage> = messages.iter().filter(|d| {
    //         match d.content {
    //             Some(ref c) => !c.is_empty(),
    //             None => false,
    //         }
    //     }).cloned().collect();
    //     let body = ChatCompletionRequest {
    //         model: self.model_name.clone(),
    //         messages: messages.iter().cloned().collect(),
    //         tools: Some(Vec::new()),
    //         stream: Some(true),
    //         max_tokens: Some(4096),
    //         temperature: Some(0.3),
    //     };
    //     let response = self.build_request()
    //         .json(&body)
    //         .send()
    //         .await
    //         .map_err(|e| ServiceError::LlmError(e.to_string()))?;
    //     let stream = response.bytes_stream()
    //         .flat_map(|result| {
    //             match result {
    //                 Ok(bytes) => {
    //                     let lines = String::from_utf8_lossy(bytes.as_ref());
    //                     let mut responses = Vec::new();
    //                     for line in lines.lines() {
    //                         let line = line.trim();
    //                         if line.starts_with("data: ") {
    //                             println!("data: {}", line);
    //                             let json_str = line.strip_prefix("data: ").unwrap_or(line);
    //                             if json_str == "[DONE]" {
    //                                 responses.push(StreamResponse {
    //                                     content: "[DONE]".to_string(),
    //                                     reasoning_content: None,
    //                                     tool_calls: None,
    //                                     finish_reason: Some("stop".to_string()),
    //                                 });
    //                             } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
    //                                 let reasoning_content = chunk.choices.first()
    //                                     .and_then(|c| c.delta.reasoning_content.clone());
    //                                 let content = chunk.choices.first()
    //                                     .and_then(|c| c.delta.content.clone())
    //                                     .unwrap_or_default();
    //                                 responses.push(StreamResponse {
    //                                     content,
    //                                     reasoning_content,
    //                                     tool_calls: chunk.choices.first()
    //                                         .and_then(|c| c.delta.tool_calls.clone()),
    //                                     finish_reason: chunk.choices.first()
    //                                         .and_then(|c| c.finish_reason.clone()),
    //                                 });
    //                             }
    //                         }
    //                     }
    //                     futures::stream::iter(responses)
    //                 }
    //                 Err(_) => futures::stream::iter(Vec::new()),
    //             }
    //         });
    //     Ok(stream)
    // }
    // /// 完整的工具调用循环：流式响应 -> 累积工具调用 -> 执行工具 -> 继续对话
    // pub async fn chat_with_tools(
    //     &self,
    //     messages: Vec<ChatMessage>,
    //     tools: Option<&[MCPTool]>,
    //     mcp_servers: HashMap<i64, String>,
    //     mcp_default_url: &str,
    //     max_tool_calls: usize,
    //     user_id: i64,
    //     project_id: i64,
    //     config_id: Option<i64>,
    //     debug_dir: Option<PathBuf>,
    // ) -> ServiceResult<Pin<Box<dyn Stream<Item = StreamResponse> + Send>>> {
    //     let mut all_messages = messages;
    //     let mut tool_call_count = 0;
    //     let base_url = self.base_url.clone();
    //     let api_key = self.api_key.clone();
    //     let model_name = self.model_name.clone();
    //     let tool_executor = ToolExecutor::new(mcp_servers, mcp_default_url, debug_dir);
    //     let tool_server_map: HashMap<String, Option<i64>> = tools
    //         .map(|ts| {
    //             ts.iter()
    //                 .map(|t| (t.name.clone(), t.server_id))
    //                 .collect()
    //         })
    //         .unwrap_or_default();
    //     let send_request = |client: &LlmClient, msgs: &[ChatMessage], tool_opts: Option<&[MCPTool]>| {
    //         let llm_tools = tool_opts.map(|mcp_tools| {
    //             mcp_tools.iter().map(|tool| {
    //                 LlmTool {
    //                     r#type: "function".to_string(),
    //                     function: LlmToolFunction {
    //                         name: tool.name.clone(),
    //                         description: tool.description.clone(),
    //                         parameters: tool.input_schema.clone(),
    //                     },
    //                 }
    //             }).collect::<Vec<LlmTool>>()
    //         });
    //         let body = ChatCompletionRequest {
    //             model: model_name.clone(),
    //             messages: msgs.iter().cloned().collect(),
    //             tools: llm_tools,
    //             stream: Some(true),
    //             max_tokens: Some(4096),
    //             temperature: Some(0.7),
    //         };
    //         client.build_request()
    //             .json(&body)
    //             .send()
    //     };
    //     let mut accumulated_content = String::new();
    //     let mut accumulated_reasoning_content = String::new();
    //     loop {
    //         if tool_call_count >= max_tool_calls {
    //             let stream = Box::pin(futures::stream::once(async move {
    //                 Ok::<_, ServiceError>(StreamResponse {
    //                     content: accumulated_content.clone(),
    //                     reasoning_content: if accumulated_reasoning_content.is_empty() {
    //                         None
    //                     } else {
    //                         Some(accumulated_reasoning_content.clone())
    //                     },
    //                     tool_calls: None,
    //                     finish_reason: Some("tool_calls_limit".to_string()),
    //                 })
    //             }).map(|r| r.unwrap())) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>;
    //             return Ok(stream);
    //         }
    //         let response = send_request(self, &all_messages, tools.map(|t| &*t))
    //             .await
    //             .map_err(|e| ServiceError::LlmError(e.to_string()))?;
    //         let mut stream = response.bytes_stream();
    //         let mut current_content = String::new();
    //         let mut current_reasoning_content = String::new();
    //         let mut accumulated_tool_calls: Vec<ToolCallInfo> = Vec::new();
    //         let mut finish_reason: Option<String> = None;
    //         let mut has_tool_call_chunk = false;
    //         while let Some(result) = stream.next().await {
    //             let bytes = result.map_err(|e| ServiceError::LlmError(e.to_string()))?;
    //             let lines = String::from_utf8_lossy(bytes.as_ref());
    //             for line in lines.lines() {
    //                 let line = line.trim();
    //                 if line.starts_with("data: ") {
    //                     let json_str = line.strip_prefix("data: ").unwrap_or(line);
    //                     if json_str == "[DONE]" {
    //                         finish_reason = Some("stop".to_string());
    //                         break;
    //                     } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
    //                         if let Some(choice) = chunk.choices.first() {
    //                             if let Some(ref content) = choice.delta.content {
    //                                 current_content.push_str(content);
    //                             }
    //                             if let Some(ref reasoning) = choice.delta.reasoning_content {
    //                                 current_reasoning_content.push_str(reasoning);
    //                             }
    //                             if let Some(ref tool_calls) = choice.delta.tool_calls {
    //                                 for tc in tool_calls {
    //                                     let idx = tc.index.unwrap_or(0) as usize;
    //                                     if idx >= accumulated_tool_calls.len() {
    //                                         accumulated_tool_calls.push(ToolCallInfo {
    //                                             id: None,
    //                                             tool_type: Some("function".to_string()),
    //                                             function: Some(ToolCallFunction {
    //                                                 name: None,
    //                                                 arguments: None,
    //                                             }),
    //                                             name: None,
    //                                             arguments: None,
    //                                             index: Some(idx as i32),
    //                                         });
    //                                     }
    //                                     if !tc.id.is_none() {
    //                                         accumulated_tool_calls[idx].id = tc.id.clone();
    //                                     }
    //                                     if let Some(ref tt) = tc.tool_type {
    //                                         if !tt.is_empty() {
    //                                             accumulated_tool_calls[idx].tool_type = Some(tt.clone());
    //                                         }
    //                                     } else {
    //                                         accumulated_tool_calls[idx].tool_type = Some("function".to_string());
    //                                     }
    //                                     if let Some(ref func) = tc.function {
    //                                         if let Some(ref name) = func.name {
    //                                             if !name.is_empty() {
    //                                                 if let Some(ref mut f) = accumulated_tool_calls[idx].function {
    //                                                     f.name = Some(name.clone());
    //                                                 }
    //                                             }
    //                                         }
    //                                         if let Some(ref args) = func.arguments {
    //                                             if !args.is_empty() {
    //                                                 if let Some(ref mut f) = accumulated_tool_calls[idx].function {
    //                                                     let before = f.arguments.clone().unwrap_or_default();
    //                                                     f.arguments = Some(format!("{}{}", before, args));
    //                                                     // println!("-- args: {}", f.arguments.clone().unwrap_or_default());
    //                                                 }
    //                                             }
    //                                         }
    //                                     }
    //                                     if let Some(ref args) = tc.arguments {
    //                                         let args_str = args.to_string();
    //                                         if !args_str.is_empty() {
    //                                             if let Some(ref mut f) = accumulated_tool_calls[idx].function {
    //                                                 if f.arguments.clone().unwrap_or_default().is_empty() {
    //                                                     f.arguments = Some(args_str);
    //                                                 }
    //                                             }
    //                                         }
    //                                     }
    //                                 }
    //                                 has_tool_call_chunk = true;
    //                             }
    //                             if let Some(ref reason) = choice.finish_reason {
    //                                 if reason == "tool_calls" {
    //                                     finish_reason = Some(reason.clone());
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         let has_tool_calls = has_tool_call_chunk &&
    //             accumulated_tool_calls.iter().any(|tc| {
    //                 tc.function.as_ref()
    //                     .and_then(|f| f.name.as_ref())
    //                     .map(|n| !n.is_empty())
    //                     .unwrap_or(false) ||
    //                 tc.name.as_ref().map(|n| !n.is_empty()).unwrap_or(false)
    //             });
    //         if !has_tool_calls {
    //             accumulated_content.push_str(&current_content);
    //             accumulated_reasoning_content.push_str(&current_reasoning_content);
    //             let stream = Box::pin(futures::stream::once(async move {
    //                 Ok::<_, ServiceError>(StreamResponse {
    //                     content: accumulated_content,
    //                     reasoning_content: if accumulated_reasoning_content.is_empty() {
    //                         None
    //                     } else {
    //                         Some(accumulated_reasoning_content)
    //                     },
    //                     tool_calls: None,
    //                     finish_reason: Some("stop".to_string()),
    //                 })
    //             }).map(|r| r.unwrap())) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>;
    //             return Ok(stream);
    //         }
    //         accumulated_content.push_str(&current_content);
    //         accumulated_reasoning_content.push_str(&current_reasoning_content);
    //         tool_call_count += 1;
    //         for tc in &mut accumulated_tool_calls {
    //             if let Some(ref mut f) = tc.function {
    //                 if f.arguments.clone().unwrap_or_default().is_empty() {
    //                     f.arguments = Some("{}".to_string());
    //                 }
    //             }
    //         }
    //         all_messages.push(ChatMessage {
    //             role: "assistant".to_string(),
    //             content: Some(current_content),
    //             tool_calls: Some(accumulated_tool_calls.clone()),
    //             tool_call_id: None,
    //             name: None,
    //         });
    //         for tool_call in &accumulated_tool_calls {
    //             let tool_name = tool_call.function.as_ref()
    //                 .and_then(|f| f.name.clone())
    //                 .filter(|n| !n.is_empty())
    //                 .or_else(|| tool_call.name.clone())
    //                 .unwrap_or_default();
    //             let func_args_str = tool_call.function.as_ref()
    //                 .and_then(|f| f.arguments.clone())
    //                 .unwrap_or_default();
    //             println!("[ExecuteTool] name={}, func.arguments='{:?}'", tool_name, func_args_str);
    //             let tool_args = if !func_args_str.is_empty() {
    //                 let trimmed = func_args_str.trim();
    //                 if let Ok(value) = serde_json::from_str(trimmed) {
    //                     println!("[ExecuteTool] Direct parse succeeded");
    //                     value
    //                 } else {
    //                     if let Ok(inner_str) = serde_json::from_str::<String>(trimmed) {
    //                         println!("[ExecuteTool] Parsed as string first, inner value: '{}'", inner_str);
    //                         serde_json::from_str(&inner_str).unwrap_or_else(|e| {
    //                             println!("[ExecuteTool] Failed to parse inner string as JSON: {}, using empty object", e);
    //                             serde_json::json!({})
    //                         })
    //                     } else {
    //                         println!("[ExecuteTool] Failed to parse arguments, using empty object");
    //                         serde_json::json!({})
    //                     }
    //                 }
    //             } else {
    //                 println!("[ExecuteTool] function.arguments is empty, using arguments field or empty object");
    //                 tool_call.arguments.clone().unwrap_or(serde_json::json!({}))
    //             };
    //             println!("[ExecuteTool] parsed arguments: {:?}", tool_args);
    //             if tool_name.is_empty() {
    //                 continue;
    //             }
    //             let mut tool_args = tool_args.clone();
    //             println!("[ExecuteTool] original arguments: {:?}", tool_args.clone());
    //             let mut user_session = UserSession::new(user_id,project_id,(if let Some(config_id) = config_id { config_id } else { 0 }));
    //             if(tool_name.contains("question") || tool_name.contains("task")) {
    //                 let mut data =  tool_args.as_object_mut().unwrap();
    //                 let mut session = Map::new();
    //                 session.insert("session".to_string(), serde_json::json!(user_session));
    //                 data.append( &mut session);
    //                 tool_args = serde_json::json!(data);
    //                 println!("[ExecuteTool] merged arguments: {:?}", tool_args);
    //             }
    //             let server_id = tool_server_map.get(&tool_name).copied().flatten();
    //             let result = tool_executor.execute_tool(&tool_name, &tool_args, server_id).await;
    //             let tool_result = match result {
    //                 Ok(content) => serde_json::json!({
    //                     "success": true,
    //                     "content": content
    //                 }).to_string(),
    //                 Err(e) => serde_json::json!({
    //                     "success": false,
    //                     "content": "",
    //                     "error": e.to_string()
    //                 }).to_string(),
    //             };
    //             all_messages.push(ChatMessage {
    //                 role: "tool".to_string(),
    //                 content: Some(tool_result),
    //                 tool_call_id: tool_call.id.clone(),
    //                 name: Some(tool_name),
    //                 tool_calls: None,
    //             });
    //         }
    //     }
    // }
    // fn build_request(&self) -> RequestBuilder {
    //     self.client.post(format!("{}/chat/completions", self.base_url))
    //         .header("Authorization", format!("Bearer {}", self.api_key))
    //         .header("Content-Type", "application/json")
    // }
}

// #[derive(Debug, Serialize)]
// struct ChatCompletionRequest {
//     model: String,
//     messages: Vec<ChatMessage>,
//     tools: Option<Vec<LlmTool>>,
//     stream: Option<bool>,
//     max_tokens: Option<i32>,
//     temperature: Option<f32>,
// }
// #[derive(Debug, Deserialize)]
// struct ChatCompletionResponse {
//     choices: Vec<ChatChoice>,
// }
// #[derive(Debug, Deserialize)]
// struct ChatChoice {
//     message: ChatMessageResponse,
// }
// #[derive(Debug, Deserialize)]
// struct ChatMessageResponse {
//     pub content: Option<String>,
//     pub tool_calls: Option<Vec<ToolCallInfo>>,
// }
// #[derive(Debug, Deserialize)]
// struct StreamChunk {
//     choices: Vec<StreamChoice>,
// }
// #[derive(Debug, Deserialize)]
// struct StreamChoice {
//     delta: StreamDelta,
//     finish_reason: Option<String>,
// }
// #[derive(Debug, Deserialize)]
// struct StreamDelta {
//     pub content: Option<String>,
//     pub tool_calls: Option<Vec<ToolCallInfo>>,
//     pub reasoning_content: Option<String>,
// }

// ============================================
// 文件容器分配的LLM调用
// ============================================

impl LlmClient {

    pub fn get_proxy(&mut self) -> &mut LlmProxy{
        &mut self.proxy
    }

    // pub fn chat_message(
    //     &mut self,
    //     chat_messages: &mut Vec<ChatMessage>, 
    //     tools : Option<&[MCPTool]>
    // ) -> impl futures_core::stream::Stream<Item = Result<StreamResponse, String>> {
    //     self.proxy.chat_stream(
    //         & *&self.tool_executor,
    //         chat_messages,
    //         tools
    //     )
    // }

    /// dispatch files to different containers, based on the file content and container configs.
    /// some file may belong to multiple containers, which can not be assigned to a single container.
    /// in this case, the file will be assigned to all containers that it belongs to.
    /// 
    /// 调用LLM进行文件容器分配
    /// 根据文件内容和容器配置，智能分析文件应该归属到哪些容器
    /// container_config_id = 0 表示共有代码（所有容器都需要）
    pub async fn assign_files_to_containers(
        &mut self,
        files: Vec<FileAssignmentInfo>,
        container_configs: Vec<ContainerConfigInfo>,
        project_id: i64,
    ) -> ServiceResult<Vec<FileAssignmentResult>> {
        let files_json = serde_json::to_string(&files)
            .map_err(|e| ServiceError::LlmError(format!("Failed to serialize files: {}", e)))?;
        
        let configs_json = serde_json::to_string(&container_configs)
            .map_err(|e| ServiceError::LlmError(format!("Failed to serialize configs: {}", e)))?;

        let system_prompt = format!(
            r#"你是一个专业的代码部署助手，负责分析项目文件并将其分配到正确的容器中。

## 任务说明

请分析以下项目文件，根据文件内容和用途，将每个文件分配到一个或多个容器中。

## 容器配置

容器配置信息（JSON格式）：
{}

## 文件列表

项目文件列表（JSON格式）：
{}

## 分配规则

1. **共有代码（container_config_id = 0）**：如果文件是所有容器都需要的通用代码，如工具函数、配置文件、共享数据模型等，请分配到 container_config_id = 0。

2. **专属代码**：如果文件只属于特定容器，请分配到对应的容器配置ID。

3. **多容器归属**：一个文件可以同时属于多个容器（包括共有代码），例如用户模型可能同时被API服务和Worker服务使用。

4. **配置文件**：如 package.json, requirements.txt 等通常属于共有代码或所有需要它们的容器。

5. **入口文件**：如 main.py, server.js 等通常属于特定容器的专属代码。

6. **测试文件**：通常属于所有相关容器或共有代码。

## 输出格式

请严格按照以下JSON格式输出，不要包含其他内容：

{{
  "assignments": [
    {{
      "file_id": <文件ID>,
      "file_path": "<文件完整路径>",
      "container_config_ids": [<容器配置ID列表，0表示共有代码>],
      "confidence_score": <0-100的置信度分数>,
      "assignment_reason": "<分配原因说明>"
    }}
  ]
}}

## 示例
{{
  "assignments": [
    {{
      "file_id": 1,
      "file_path": "src/common/utils.py",
      "container_config_ids": [0],
      "confidence_score": 95,
      "assignment_reason": "通用工具函数，所有容器都需要"
    }},
    {{
      "file_id": 2,
      "file_path": "src/api/main.py",
      "container_config_ids": [1],
      "confidence_score": 90,
      "assignment_reason": "API服务入口文件，专属api-server容器"
    }},
    {{
      "file_id": 3,
      "file_path": "src/models/user.py",
      "container_config_ids": [0, 1, 2],
      "confidence_score": 85,
      "assignment_reason": "用户模型，api-server和worker容器都需要"
    }}
  ]
}}
"#,
            configs_json, files_json
        );

        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt),
                ..Default::default()
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some("请分析以上文件并进行容器分配。请严格按照指定的JSON格式输出结果，不要包含任何其他文字或解释。".to_string()),
                ..Default::default()
            },
        ];

        
        let stream = self.proxy.chat_stream(&self.tool_executor, &mut messages,None).await;
        pin_mut!(stream);
        let mut content = String::new();
        // merge the content of the stream.
        while let Some(data) = stream.next().await {
            match data {
                Ok(response) => {
                    let resp_content = response.content;
                    content.push_str(resp_content.as_str());
                },
                Err(_) => {
                }
            }
        }

        //analysis the content to extract the assignments from the JSON embedded in the content.
        let content_json: serde_json::Value = serde_json::from_str(content.as_str())
            .map_err(|e| ServiceError::LlmError(format!("Failed to parse content as JSON: {}", e)))?;

        let assignments_array = content_json.get("assignments")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ServiceError::LlmError("Invalid assignments format".to_string()))?;

        let mut assignments = Vec::new();
        for item in assignments_array {
            let file_id = item.get("file_id")
                .and_then(|f| f.as_i64())
                .ok_or_else(|| ServiceError::LlmError("Missing file_id in assignment".to_string()))?;

            let file_path = item.get("file_path")
                .and_then(|f| f.as_str())
                .ok_or_else(|| ServiceError::LlmError("Missing file_path in assignment".to_string()))?
                .to_string();

            let container_config_ids: Vec<i64> = item.get("container_config_ids")
                .and_then(|c| c.as_array())
                .ok_or_else(|| ServiceError::LlmError("Missing container_config_ids in assignment".to_string()))?
                .iter()
                .filter_map(|id| id.as_i64())
                .collect();

            let confidence_score = item.get("confidence_score")
                .and_then(|c| c.as_f64())
                .unwrap_or(80.0);

            let assignment_reason = item.get("assignment_reason")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string();

            assignments.push(FileAssignmentResult {
                file_id,
                file_path,
                container_config_ids,
                confidence_score,
                assignment_reason,
            });
        }

        Ok(assignments)
    }
}
