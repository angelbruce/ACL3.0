// 该文件是从 llm-svc/src/client.rs 复制过来的逻辑，
// 避免修改 llm-svc 已有的代码。
// 这里将 LlmClient、ToolExecutor、StreamResponse 等类型复制到 workspace-svc，
// 以便在容器部署流程中直接调用 LLM 并执行 MCP 工具。

use futures::{Stream, StreamExt};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;
use std::pin::Pin;
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{MCPTool, LlmTool, LlmToolFunction, ToolCallInfo, ToolCallFunction, ChatMessage};
use crate::model::UserSession;

/// 工具执行结果（与 llm-svc 保持一致）
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// ToolExecutor：负责将 LLM 返回的工具调用转发到对应的 MCP-SERVER
pub struct ToolExecutor {
    servers: HashMap<i64, String>,
    default_server_url: String,
    client: Client,
}

impl ToolExecutor {
    pub fn new(servers: HashMap<i64, String>, default_server_url: &str) -> Self {
        ToolExecutor {
            servers,
            default_server_url: default_server_url.to_string(),
            client: Client::new(),
        }
    }

    pub fn get_server_url(&self, server_id: Option<i64>) -> String {
        match server_id {
            Some(id) => self.servers.get(&id)
                .cloned()
                .unwrap_or_else(|| self.default_server_url.clone()),
            None => self.default_server_url.clone(),
        }
    }

    pub async fn execute_tool(&self, name: &str, arguments: &serde_json::Value, server_id: Option<i64>) -> ServiceResult<String> {
        let server_url = self.get_server_url(server_id);
        let url = format!("{}", server_url);
        let request = serde_json::json!({
            "id": 1,
            "method": "tools/call",
            "jsonrpc": "2.0",
            "params" : {
                "name": name.to_string(),
                "arguments": arguments.clone(),
            },
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ServiceError::McpError(e.to_string()))?;

        // MCP 返回的是 SSE 流式格式，需要逐行解析
        let body_bytes = response.bytes()
            .await
            .map_err(|e| ServiceError::McpError(e.to_string()))?;
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("[ExecuteTool] Raw response body: {}", body_str);

        // 解析 SSE 格式：每行以 "data: " 开头
        let mut content = String::new();
        for line in body_str.lines() {
            let trimmed = line.trim();
            if let Some(data) = trimmed.strip_prefix("data: ") {
                // 跳过 [DONE] 标记
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    // 从 MCP JSON-RPC 响应中提取 content
                    if let Some(result) = value.get("result") {
                        if let Some(content_list) = result.get("content") {
                            if let Some(arr) = content_list.as_array() {
                                for item in arr {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        if !content.is_empty() {
                                            content.push('\n');
                                        }
                                        content.push_str(text);
                                    }
                                }
                            }
                        }
                        if let Some(error) = result.get("isError") {
                            if error.as_bool() == Some(true) {
                                return Err(ServiceError::McpError(
                                    format!("Tool execution returned error: {}", content)
                                ));
                            }
                        }
                    }
                    if let Some(err) = value.get("error") {
                        let err_msg = err.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        return Err(ServiceError::McpError(err_msg.to_string()));
                    }
                }
            }
        }

        if content.is_empty() {
            content = "Tool execution completed".to_string();
        }
        Ok(content)
    }

    /// 从 MCP-SSE 服务获取工具列表
    pub async fn list_tools(&self, server_id: Option<i64>) -> ServiceResult<Vec<MCPTool>> {
        let server_url = self.get_server_url(server_id);
        let request = serde_json::json!({
            "id": 1,
            "method": "tools/list",
            "jsonrpc": "2.0",
        });

        let response = self.client
            .post(&server_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ServiceError::McpError(e.to_string()))?;

        let body_bytes = response.bytes()
            .await
            .map_err(|e| ServiceError::McpError(e.to_string()))?;
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("[ListTools] Raw response body: {}", body_str);

        // 解析 SSE 格式响应
        let mut tools: Vec<MCPTool> = Vec::new();
        for line in body_str.lines() {
            let trimmed = line.trim();
            if let Some(data) = trimmed.strip_prefix("data: ") {
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(result) = value.get("result") {
                        if let Some(tool_list) = result.get("tools") {
                            if let Some(arr) = tool_list.as_array() {
                                for tool_item in arr {
                                    let name = tool_item.get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let description = tool_item.get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let input_schema = tool_item.get("inputSchema")
                                        .cloned()
                                        .unwrap_or_else(|| serde_json::json!({}));
                                    let output_schema = tool_item.get("outputSchema")
                                        .cloned()
                                        .unwrap_or_else(|| serde_json::json!({}));
                                    
                                    tools.push(MCPTool {
                                        name,
                                        description,
                                        input_schema,
                                        output_schema,
                                        server_id,
                                    });
                                }
                            }
                        }
                    }
                    if let Some(err) = value.get("error") {
                        let err_msg = err.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        return Err(ServiceError::McpError(err_msg.to_string()));
                    }
                }
            }
        }

        println!("[ListTools] Found {} tools", tools.len());
        Ok(tools)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    pub finish_reason: Option<String>,
}

pub struct LlmClient {
    base_url: String,
    api_key: String,
    model_name: String,
    client: Client,
}

impl LlmClient {
    pub fn new(base_url: &str, api_key: &str, model_name: &str) -> Self {
        LlmClient {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model_name: model_name.to_string(),
            client: Client::new(),
        }
    }

    pub async fn chat(&self, messages: &[ChatMessage], tools: Option<&[MCPTool]>) -> ServiceResult<ChatResponse> {
        let llm_tools = tools.map(|mcp_tools| {
            mcp_tools.iter().map(|tool| LlmTool {
                r#type: "function".to_string(),
                function: LlmToolFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            }).collect::<Vec<LlmTool>>()
        });

        let body = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: messages.iter().cloned().collect(),
            tools: llm_tools,
            stream: Some(false),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        let response = self.build_request()
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::LlmError(e.to_string()))?;

        let result: ChatCompletionResponse = response.json()
            .await
            .map_err(|e| ServiceError::LlmError(e.to_string()))?;

        Ok(ChatResponse {
            content: result.choices.first()
                .and_then(|c| c.message.content.clone())
                .unwrap_or_default(),
            tool_calls: result.choices.first()
                .and_then(|c| c.message.tool_calls.clone()),
        })
    }

    pub async fn chat_stream(&self, messages: &[ChatMessage], tools: Option<&[MCPTool]>) -> ServiceResult<impl Stream<Item = StreamResponse>> {
        let llm_tools = tools.map(|mcp_tools| {
            mcp_tools.iter().map(|tool| LlmTool {
                r#type: "function".to_string(),
                function: LlmToolFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            }).collect::<Vec<LlmTool>>()
        });

        println!("tools: {:?}", llm_tools.clone());

        let messages: Vec<ChatMessage> = messages.iter().filter(|d| {
            match d.content {
                Some(ref c) => !c.is_empty(),
                None => false,
            }
        }).cloned().collect();

        let body = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: messages.iter().cloned().collect(),
            tools: Some(Vec::new()),
            stream: Some(true),
            max_tokens: Some(4096),
            temperature: Some(0.3),
        };

        let response = self.build_request()
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::LlmError(e.to_string()))?;

        let stream = response.bytes_stream()
            .flat_map(|result| {
                match result {
                    Ok(bytes) => {
                        let lines = String::from_utf8_lossy(bytes.as_ref());
                        let mut responses = Vec::new();

                        for line in lines.lines() {
                            let line = line.trim();
                            if line.starts_with("data: ") {
                                println!("data: {}", line);
                                let json_str = line.strip_prefix("data: ").unwrap_or(line);
                                if json_str == "[DONE]" {
                                    responses.push(StreamResponse {
                                        content: "[DONE]".to_string(),
                                        tool_calls: None,
                                        finish_reason: Some("stop".to_string()),
                                    });
                                } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                                    let content = chunk.choices.first()
                                        .and_then(|c| c.delta.reasoning_content.clone())
                                        .or_else(|| chunk.choices.first()
                                            .and_then(|c| c.delta.content.clone()))
                                        .unwrap_or_default();
                                    responses.push(StreamResponse {
                                        content,
                                        tool_calls: chunk.choices.first()
                                            .and_then(|c| c.delta.tool_calls.clone()),
                                        finish_reason: chunk.choices.first()
                                            .and_then(|c| c.finish_reason.clone()),
                                    });
                                }
                            }
                        }

                        futures::stream::iter(responses)
                    }
                    Err(_) => futures::stream::iter(Vec::new()),
                }
            });

        Ok(stream)
    }

    /// 完整的工具调用循环：流式响应 -> 累积工具调用 -> 执行工具 -> 继续对话
    pub async fn chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<&[MCPTool]>,
        mcp_servers: HashMap<i64, String>,
        mcp_default_url: &str,
        max_tool_calls: usize,
        user_id: i64,
        project_id: i64,
        config_id: Option<i64>,
    ) -> ServiceResult<Pin<Box<dyn Stream<Item = StreamResponse> + Send>>> {
        let mut all_messages = messages;
        let mut tool_call_count = 0;
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let model_name = self.model_name.clone();

        let tool_executor = ToolExecutor::new(mcp_servers, mcp_default_url);

        let tool_server_map: HashMap<String, Option<i64>> = tools
            .map(|ts| {
                ts.iter()
                    .map(|t| (t.name.clone(), t.server_id))
                    .collect()
            })
            .unwrap_or_default();

        let send_request = |client: &LlmClient, msgs: &[ChatMessage], tool_opts: Option<&[MCPTool]>| {
            let llm_tools = tool_opts.map(|mcp_tools| {
                mcp_tools.iter().map(|tool| {
                    LlmTool {
                        r#type: "function".to_string(),
                        function: LlmToolFunction {
                            name: tool.name.clone(),
                            description: tool.description.clone(),
                            parameters: tool.input_schema.clone(),
                        },
                    }
                }).collect::<Vec<LlmTool>>()
            });

            let body = ChatCompletionRequest {
                model: model_name.clone(),
                messages: msgs.iter().cloned().collect(),
                tools: llm_tools,
                stream: Some(true),
                max_tokens: Some(4096),
                temperature: Some(0.7),
            };

            client.build_request()
                .json(&body)
                .send()
        };

        let mut accumulated_content = String::new();

        loop {
            if tool_call_count >= max_tool_calls {
                let stream = Box::pin(futures::stream::once(async move {
                    Ok::<_, ServiceError>(StreamResponse {
                        content: accumulated_content.clone(),
                        tool_calls: None,
                        finish_reason: Some("tool_calls_limit".to_string()),
                    })
                }).map(|r| r.unwrap())) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>;
                return Ok(stream);
            }

            let response = send_request(self, &all_messages, tools.map(|t| &*t))
                .await
                .map_err(|e| ServiceError::LlmError(e.to_string()))?;

            let mut stream = response.bytes_stream();
            let mut current_content = String::new();
            let mut reasoning_content = String::new();
            let mut accumulated_tool_calls: Vec<ToolCallInfo> = Vec::new();
            let mut finish_reason: Option<String> = None;
            let mut has_tool_call_chunk = false;

            while let Some(result) = stream.next().await {
                let bytes = result.map_err(|e| ServiceError::LlmError(e.to_string()))?;
                let lines = String::from_utf8_lossy(bytes.as_ref());

                for line in lines.lines() {
                    let line = line.trim();
                    if line.starts_with("data: ") {
                        let json_str = line.strip_prefix("data: ").unwrap_or(line);
                        if json_str == "[DONE]" {
                            finish_reason = Some("stop".to_string());
                            break;
                        } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(ref content) = choice.delta.content {
                                    current_content.push_str(content);
                                }
                                if let Some(ref reasoning) = choice.delta.reasoning_content {
                                    current_content.push_str(reasoning.as_str().clone());
                                    reasoning_content.push_str(reasoning.as_str().clone());
                                }

                                if let Some(ref tool_calls) = choice.delta.tool_calls {
                                    for tc in tool_calls {
                                        let idx = tc.index.unwrap_or(0) as usize;

                                        if idx >= accumulated_tool_calls.len() {
                                            accumulated_tool_calls.push(ToolCallInfo {
                                                id: None,
                                                tool_type: Some("function".to_string()),
                                                function: Some(ToolCallFunction {
                                                    name: None,
                                                    arguments: None,
                                                }),
                                                name: None,
                                                arguments: None,
                                                index: Some(idx as i32),
                                            });
                                        }


                                        if !tc.id.is_none() {
                                            accumulated_tool_calls[idx].id = tc.id.clone();
                                        }
                                        if let Some(ref tt) = tc.tool_type {
                                            if !tt.is_empty() {
                                                accumulated_tool_calls[idx].tool_type = Some(tt.clone());
                                            }
                                        } else {
                                            accumulated_tool_calls[idx].tool_type = Some("function".to_string());
                                        }

                                        if let Some(ref func) = tc.function {
                                            if let Some(ref name) = func.name {
                                                if !name.is_empty() {
                                                    if let Some(ref mut f) = accumulated_tool_calls[idx].function {
                                                        f.name = Some(name.clone());
                                                    }
                                                }
                                            }
                                            if let Some(ref args) = func.arguments {
                                                if !args.is_empty() {
                                                    if let Some(ref mut f) = accumulated_tool_calls[idx].function {
                                                        let before = f.arguments.clone().unwrap_or_default();
                                                        f.arguments = Some(format!("{}{}", before, args));
                                                        // println!("-- args: {}", f.arguments.clone().unwrap_or_default());
                                                    }
                                                }
                                            }
                                        }

                                        if let Some(ref args) = tc.arguments {
                                            let args_str = args.to_string();
                                            if !args_str.is_empty() {
                                                if let Some(ref mut f) = accumulated_tool_calls[idx].function {
                                                    if f.arguments.clone().unwrap_or_default().is_empty() {
                                                        f.arguments = Some(args_str);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    has_tool_call_chunk = true;
                                }

                                if let Some(ref reason) = choice.finish_reason {
                                    if reason == "tool_calls" {
                                        finish_reason = Some(reason.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let has_tool_calls = has_tool_call_chunk &&
                accumulated_tool_calls.iter().any(|tc| {
                    tc.function.as_ref()
                        .and_then(|f| f.name.as_ref())
                        .map(|n| !n.is_empty())
                        .unwrap_or(false) ||
                    tc.name.as_ref().map(|n| !n.is_empty()).unwrap_or(false)
                });

            if !has_tool_calls {
                accumulated_content.push_str(&current_content);
                let stream = Box::pin(futures::stream::once(async move {
                    Ok::<_, ServiceError>(StreamResponse {
                        content: accumulated_content,
                        tool_calls: None,
                        finish_reason: Some("stop".to_string()),
                    })
                }).map(|r| r.unwrap())) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>;
                return Ok(stream);
            }

            accumulated_content.push_str(&current_content);
            tool_call_count += 1;

            for tc in &mut accumulated_tool_calls {
                if let Some(ref mut f) = tc.function {
                    if f.arguments.clone().unwrap_or_default().is_empty() {
                        f.arguments = Some("{}".to_string());
                    }
                }
            }

            all_messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: Some(current_content),
                tool_calls: Some(accumulated_tool_calls.clone()),
                tool_call_id: None,
                name: None,
            });

            for tool_call in &accumulated_tool_calls {
                let tool_name = tool_call.function.as_ref()
                    .and_then(|f| f.name.clone())
                    .filter(|n| !n.is_empty())
                    .or_else(|| tool_call.name.clone())
                    .unwrap_or_default();

                let func_args_str = tool_call.function.as_ref()
                    .and_then(|f| f.arguments.clone())
                    .unwrap_or_default();
                println!("[ExecuteTool] name={}, func.arguments='{:?}'", tool_name, func_args_str);

                let tool_args = if !func_args_str.is_empty() {
                    let trimmed = func_args_str.trim();
                    if let Ok(value) = serde_json::from_str(trimmed) {
                        println!("[ExecuteTool] Direct parse succeeded");
                        value
                    } else {
                        if let Ok(inner_str) = serde_json::from_str::<String>(trimmed) {
                            println!("[ExecuteTool] Parsed as string first, inner value: '{}'", inner_str);
                            serde_json::from_str(&inner_str).unwrap_or_else(|e| {
                                println!("[ExecuteTool] Failed to parse inner string as JSON: {}, using empty object", e);
                                serde_json::json!({})
                            })
                        } else {
                            println!("[ExecuteTool] Failed to parse arguments, using empty object");
                            serde_json::json!({})
                        }
                    }
                } else {
                    println!("[ExecuteTool] function.arguments is empty, using arguments field or empty object");
                    tool_call.arguments.clone().unwrap_or(serde_json::json!({}))
                };

                println!("[ExecuteTool] parsed arguments: {:?}", tool_args);

                if tool_name.is_empty() {
                    continue;
                }


                let mut tool_args = tool_args.clone();
                println!("[ExecuteTool] original arguments: {:?}", tool_args.clone());
                let mut user_session = UserSession::new(user_id,project_id,(if let Some(config_id) = config_id { config_id } else { 0 }));
                if(tool_name.contains("question") || tool_name.contains("task")) {
                    let mut data =  tool_args.as_object_mut().unwrap();
                    let mut session = Map::new();
                    session.insert("session".to_string(), serde_json::json!(user_session));
                    data.append( &mut session);
                    tool_args = serde_json::json!(data);
               
                    println!("[ExecuteTool] merged arguments: {:?}", tool_args);
                }

                let server_id = tool_server_map.get(&tool_name).copied().flatten();

                let result = tool_executor.execute_tool(&tool_name, &tool_args, server_id).await;

                let tool_result = match result {
                    Ok(content) => serde_json::json!({
                        "success": true,
                        "content": content
                    }).to_string(),
                    Err(e) => serde_json::json!({
                        "success": false,
                        "content": "",
                        "error": e.to_string()
                    }).to_string(),
                };

                all_messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: Some(tool_result),
                    tool_call_id: tool_call.id.clone(),
                    name: Some(tool_name),
                    tool_calls: None,
                });
            }
        }
    }

    fn build_request(&self) -> RequestBuilder {
        self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<LlmTool>>,
    stream: Option<bool>,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    pub reasoning_content: Option<String>,
}
