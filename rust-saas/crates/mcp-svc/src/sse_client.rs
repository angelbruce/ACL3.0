use reqwest::{Client, header::{HeaderMap, HeaderValue, HeaderName}};
use serde_json::{json, Value};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{MCPTool, MCPToolCallResult, McpServer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

fn parse_sse_response(response_text: &str) -> ServiceResult<Value> {
    for line in response_text.lines() {
        if line.starts_with("data: ") {
            let data = line.strip_prefix("data: ").unwrap_or(line);
            return serde_json::from_str(data)
                .map_err(|e| ServiceError::McpError(format!("Failed to parse SSE data: {}", e)));
        }
    }
    
    serde_json::from_str(response_text)
        .map_err(|e| ServiceError::McpError(format!("Failed to parse response: {}", e)))
}

pub struct McpSseClient {
    client: Client,
    server: McpServer,
    initialized: Arc<RwLock<bool>>,
}

impl McpSseClient {
    pub fn new(server: McpServer) -> Self {
        let mut headers = HeaderMap::new();
        
        if let Some(custom_headers) = server.headers.clone() {
            if let Some(obj) = custom_headers.as_object() {
                for (key, value) in obj {
                    if let Some(v) = value.as_str() {
                        if let Ok(header_name) = key.parse::<HeaderName>() {
                            if let Ok(header_value) = HeaderValue::from_str(v) {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }
            }
        }
        
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
        
        McpSseClient { 
            client, 
            server,
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    async fn ensure_initialized(&self) -> ServiceResult<()> {
        if self.server.stateless {
            let mut initialized = self.initialized.write().await;
            if !*initialized {
                if let Err(e) = self.initialize_stateless().await {
                    tracing::warn!("Failed to initialize stateless server: {}", e);
                }
                *initialized = true;
            }
        }
        Ok(())
    }

    async fn initialize_stateless(&self) -> ServiceResult<()> {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-02-15",
                "capabilities": {},
                "clientInfo": {
                    "name": "acl-mcp-client",
                    "version": "1.0.0"
                }
            }
        });

        println!("Request body: {:?}", request_body);
        println!("self.server.url: {:?}", self.server.url);

        let response = self.client
            .post(&self.server.url)
            // .header("Content-Type", "application/event-stream")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to initialize stateless server: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::McpError(format!("Initialization failed with status: {}", response.status())));
        }
        
        tracing::info!("Stateless server initialized successfully");
        Ok(())
    }

    pub async fn list_tools(&self) -> ServiceResult<Vec<MCPTool>> {
        if self.server.stateless {
            self.ensure_initialized().await?;
            return self.list_tools_stateless().await;
        }
        
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/list",
            "params": {}
        });

        let response = self.client
            .post(&self.server.url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to connect to MCP server: {}", e)))?;

            println!("Response status: {:?}", response.status());
            let bytes = response.bytes().await.map_err(|e| ServiceError::McpError(format!("Failed to read response: {}", e)))?;
            let response_text = String::from_utf8_lossy(&bytes);
        
        println!("Raw MCP server response: {}", response_text);
        let response_json = parse_sse_response(&response_text)?;

        println!("Parsed MCP response: {}", response_json);


        if let Some(error) = response_json.get("error") {
            return Err(ServiceError::McpError(format!("MCP error: {}", error)));
        }

        let mut tools: Vec<MCPTool> = response_json
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        Ok(tools)
    }

    async fn list_tools_stateless(&self) -> ServiceResult<Vec<MCPTool>> {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/list",
            "params": {}
        });

        let headers = self.server.headers.clone();
        let content_type = match headers {
            Some(headers) => {
                let obj = headers.as_object();
                match obj {
                    Some(obj) => {
                        let content_type = obj.get("Content-Type").cloned();
                        if let Some(data) = content_type {
                            data.to_string()
                        } else {
                        "".to_string()
                        }
                    }
                    None => {
                        "".to_string()
                    }
                }
                
            }
            None => {
                "".to_string()
            }
        };


        println!("Content-Type: {:?}", content_type);

        let response = self.client
            .post(&self.server.url)
            .json(&request_body)
            .header("Content-Type", &content_type)
            .header("Accept", "*/*")
            .send()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to connect to MCP server: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::McpError(format!("Failed to list tools: {}", response.status())));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to read response: {}", e)))?;

        tracing::info!("Raw MCP server response: {}", response_text);

        let response_json = parse_sse_response(&response_text)?;

        tracing::info!("Parsed MCP response: {}", response_json);

        if let Some(error) = response_json.get("error") {
            return Err(ServiceError::McpError(format!("MCP error: {}", error)));
        }

        let mut tools: Vec<MCPTool> = response_json
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        tracing::info!("Parsed {} tools from MCP server", tools.len());

        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, arguments: &Value) -> ServiceResult<MCPToolCallResult> {
        if self.server.stateless {
            self.ensure_initialized().await?;
            return self.call_tool_stateless(name, arguments).await;
        }
        
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let response = self.client
            .post(&self.server.url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to call tool: {}", e)))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response_json.get("error") {
            return Ok(MCPToolCallResult {
                success: false,
                content: "".to_string(),
                error: Some(error.to_string()),
            });
        }

        let content = match response_json.get("result").and_then(|r| r.get("content")) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(arr)) => {
                arr.iter()
                    .filter_map(|item| {
                        item.get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Some(v) => v.to_string(),
            None => response_json["result"].to_string(),
        };

        Ok(MCPToolCallResult {
            success: true,
            content,
            error: None,
        })
    }

    async fn call_tool_stateless(&self, name: &str, arguments: &Value) -> ServiceResult<MCPToolCallResult> {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let response = self.client
            .post(&self.server.url)
            .json(&request_body)
            .header("Accept", "*/*")
            .send()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to call tool: {}", e)))?;

        if !response.status().is_success() {
            return Ok(MCPToolCallResult {
                success: false,
                content: "".to_string(),
                error: Some(format!("HTTP error: {}", response.status())),
            });
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| ServiceError::McpError(format!("Failed to read response: {}", e)))?;

        tracing::info!("Tool call response: {}", response_text);

        let response_json = parse_sse_response(&response_text)?;

        tracing::info!("Parsed tool call response: {}", response_json);

        if let Some(error) = response_json.get("error") {
            return Ok(MCPToolCallResult {
                success: false,
                content: "".to_string(),
                error: Some(error.to_string()),
            });
        }

        let is_error = response_json.get("result").and_then(|r| r.get("isError")).and_then(|e| e.as_bool()).unwrap_or(false);
        
        let content = match response_json.get("result").and_then(|r| r.get("content")) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(arr)) => {
                arr.iter()
                    .filter_map(|item| {
                        item.get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Some(v) => v.to_string(),
            None => response_json["result"].to_string(),
        };

        Ok(MCPToolCallResult {
            success: !is_error,
            content: content.clone(),
            error: if is_error { Some(content) } else { None },
        })
    }
}

pub struct McpClientRegistry {
    clients: Arc<RwLock<HashMap<i64, McpSseClient>>>,
}

impl McpClientRegistry {
    pub fn new() -> Self {
        McpClientRegistry {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_server(&self, server: McpServer) {
        let client = McpSseClient::new(server.clone());
        let mut clients = self.clients.write().await;
        clients.insert(server.id, client);
    }

    pub async fn unregister_server(&self, server_id: i64) {
        let mut clients = self.clients.write().await;
        clients.remove(&server_id);
    }

    pub async fn get_client(&self, server_id: i64) -> Option<McpSseClient> {
        let clients = self.clients.read().await;
        clients.get(&server_id).map(|c| McpSseClient {
            client: c.client.clone(),
            server: c.server.clone(),
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn list_all_tools(&self) -> ServiceResult<Vec<(i64, String, Vec<MCPTool>)>> {
        let clients = self.clients.read().await;
        let mut results = Vec::new();

        for (server_id, client) in clients.iter() {
            let server_name = client.server.name.clone();
            match client.list_tools().await {
                Ok(tools) => results.push((*server_id, server_name, tools)),
                Err(e) => tracing::error!("Failed to list tools from server {}: {}", server_name, e),
            }
        }

        Ok(results)
    }
}

lazy_static::lazy_static! {
    pub static ref MCP_CLIENT_REGISTRY: McpClientRegistry = McpClientRegistry::new();
}
