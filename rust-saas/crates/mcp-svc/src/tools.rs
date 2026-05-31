use serde_json::Value;
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{MCPTool, MCPToolCallResult};
use std::collections::HashMap;
use std::sync::RwLock;

type ToolHandler = fn(&Value) -> ServiceResult<String>;

pub struct ToolDefinition {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
    handler: ToolHandler,
}

lazy_static::lazy_static! {
    pub static ref TOOL_REGISTRY: RwLock<HashMap<String, ToolDefinition>> = RwLock::new({
        let mut map = HashMap::new();
        map.insert("file_read".to_string(), ToolDefinition {
            name: "file_read".to_string(),
            description: "Read content from a file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file" }
                },
                "required": ["path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "File content" }
                }
            }),
            handler: file_read_handler,
        });
        map.insert("file_write".to_string(), ToolDefinition {
            name: "file_write".to_string(),
            description: "Write content to a file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean", "description": "Whether the write was successful" }
                }
            }),
            handler: file_write_handler,
        });
        map.insert("bash_exec".to_string(), ToolDefinition {
            name: "bash_exec".to_string(),
            description: "Execute a bash command".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Command to execute" }
                },
                "required": ["command"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "stdout": { "type": "string", "description": "Command output" },
                    "stderr": { "type": "string", "description": "Command error output" },
                    "exit_code": { "type": "integer", "description": "Exit code" }
                }
            }),
            handler: bash_exec_handler,
        });
        map.insert("web_search".to_string(), ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["query"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "results": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string" },
                                "url": { "type": "string" },
                                "snippet": { "type": "string" }
                            }
                        }
                    }
                }
            }),
            handler: web_search_handler,
        });
        map
    });
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn get_all_tools() -> Vec<MCPTool> {
        TOOL_REGISTRY.read().unwrap()
            .values()
            .map(|td| MCPTool {
                name: td.name.clone(),
                description: td.description.clone(),
                input_schema: td.input_schema.clone(),
                output_schema: td.output_schema.clone(),
                server_id: None,
            })
            .collect()
    }

    pub async fn call_tool(name: &str, args: &Value) -> ServiceResult<MCPToolCallResult> {
        let registry = TOOL_REGISTRY.read().unwrap();
        let tool = registry.get(name)
            .ok_or(ServiceError::McpError(format!("Tool '{}' not found", name)))?;
        
        match (tool.handler)(args) {
            Ok(content) => Ok(MCPToolCallResult {
                success: true,
                content,
                error: None,
            }),
            Err(e) => Ok(MCPToolCallResult {
                success: false,
                content: "".to_string(),
                error: Some(e.to_string()),
            }),
        }
    }
}

fn file_read_handler(args: &Value) -> ServiceResult<String> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .ok_or(ServiceError::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;
    
    let content = std::fs::read_to_string(path)
        .map_err(|e| ServiceError::McpError(e.to_string()))?;
    
    Ok(serde_json::json!({ "content": content }).to_string())
}

fn file_write_handler(args: &Value) -> ServiceResult<String> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .ok_or(ServiceError::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;
    
    let content = args.get("content")
        .and_then(|v| v.as_str())
        .ok_or(ServiceError::InvalidInput("Missing or invalid 'content' parameter".to_string()))?;
    
    std::fs::write(path, content)
        .map_err(|e| ServiceError::McpError(e.to_string()))?;
    
    Ok(serde_json::json!({ "success": true }).to_string())
}

fn bash_exec_handler(args: &Value) -> ServiceResult<String> {
    let command = args.get("command")
        .and_then(|v| v.as_str())
        .ok_or(ServiceError::InvalidInput("Missing or invalid 'command' parameter".to_string()))?;
    
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| ServiceError::McpError(e.to_string()))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    Ok(serde_json::json!({
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": output.status.code().unwrap_or(-1)
    }).to_string())
}

fn web_search_handler(args: &Value) -> ServiceResult<String> {
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or(ServiceError::InvalidInput("Missing or invalid 'query' parameter".to_string()))?;
    
    let results = vec![
        serde_json::json!({
            "title": format!("Search results for: {}", query),
            "url": "https://example.com",
            "snippet": "Sample search result snippet for demonstration purposes."
        })
    ];
    
    Ok(serde_json::json!({ "results": results }).to_string())
}