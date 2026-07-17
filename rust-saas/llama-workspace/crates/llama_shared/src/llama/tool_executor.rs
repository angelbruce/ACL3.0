use std::collections::HashMap;
use std::pin::Pin;
use futures::Stream;
use std::sync::Arc;
use core::marker::Send;
use reqwest::Client;

use crate::llama::gemma4_local::*;
use shared::errors::*;
use shared::models::{MCPTool, LlmToolFunction, LlmTool,ToolCallInfo,ToolCallFunction, 
    ChatMessage, StreamChunk,StreamResponse,ChatCompletionRequest,UserSession
};


pub struct ToolExecutor {
    servers: HashMap<i64, String>,
    default_server_url: String,
    client: Client,
}

impl ToolExecutor {
    pub fn new(servers: HashMap<i64, String>, default_server_url: &str) -> Self {
        Self {
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
                    // 格式: {"result": {"content": [{"type":"text","text":"..."}]}}
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
                        // 检查是否有 error
                        if let Some(error) = result.get("isError") {
                            if error.as_bool() == Some(true) {
                                return Err(ServiceError::McpError(
                                    format!("Tool execution returned error: {}", content)
                                ));
                            }
                        }
                    }
                    // 也检查顶层 error
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
    pub async fn list_tools(&self, server_url: Option<String>) -> ServiceResult<Vec<MCPTool>> {
        if server_url == None {
            return Ok(Vec::new());
        }

        let server_url = server_url.unwrap();
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
                                        server_id: None,
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