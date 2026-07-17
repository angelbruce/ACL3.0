use axum::{extract::Path, Json};
use shared::errors::ServiceResult;
use shared::models::{MCPTool, MCPToolCallResult, McpServer, CreateMcpServerRequest, McpServerWithTools};
use crate::tools::ToolRegistry;
use crate::repository::McpServerRepository;
use crate::sse_client::MCP_CLIENT_REGISTRY;

pub async fn list_tools() -> ServiceResult<Json<Vec<MCPTool>>> {
    let mut all_tools: Vec<MCPTool> = ToolRegistry::get_all_tools()
        .into_iter()
        .map(|mut tool| {
            tool.server_id = None;
            tool
        })
        .collect();
    
    if let Ok(external_tools) = MCP_CLIENT_REGISTRY.list_all_tools().await {
        for (server_id, _server_name, mut tools) in external_tools {
            for tool in tools.iter_mut() {
                tool.server_id = Some(server_id);
            }
            all_tools.extend(tools);
        }
    }
    
    Ok(Json(all_tools))
}

pub async fn call_tool(Path(name): Path<String>, Json(args): Json<serde_json::Value>) -> ServiceResult<Json<MCPToolCallResult>> {
    tracing::info!("Calling tool: {}", name);
    
    if let Ok(result) = ToolRegistry::call_tool(&name, &args).await {
        if result.success {
            tracing::info!("Internal tool call successful");
            return Ok(Json(result));
        }
        tracing::info!("Internal tool call failed, trying external");
    } else {
        tracing::info!("Internal tool not found, trying external");
    }
    
    let external_tools = MCP_CLIENT_REGISTRY.list_all_tools().await.unwrap_or_default();
    tracing::info!("Found {} external tool sets", external_tools.len());
    
    for (server_id, server_name, tools) in external_tools {
        tracing::info!("Checking server {} with {} tools", server_name, tools.len());
        if tools.iter().any(|t| t.name == name) {
            tracing::info!("Found tool {} on server {}", name, server_name);
            if let Some(client) = MCP_CLIENT_REGISTRY.get_client(server_id).await {
                tracing::info!("Got client for server {}, calling tool", server_name);
                let result = client.call_tool(&name, &args).await?;
                return Ok(Json(result));
            } else {
                tracing::warn!("Failed to get client for server {}", server_id);
            }
        }
    }
    
    tracing::warn!("No external tool found, falling back to internal");
    let result = ToolRegistry::call_tool(&name, &args).await?;
    Ok(Json(result))
}

pub async fn list_mcp_servers() -> ServiceResult<Json<Vec<McpServer>>> {
    let repo = McpServerRepository::new();
    let servers = repo.get_all_servers().await?;
    Ok(Json(servers))
}

pub async fn get_mcp_server(Path(id): Path<i64>) -> ServiceResult<Json<McpServer>> {
    let repo = McpServerRepository::new();
    let server = repo.get_server(id).await?;
    Ok(Json(server))
}

pub async fn create_mcp_server(Json(req): Json<CreateMcpServerRequest>) -> ServiceResult<Json<McpServer>> {
    let repo = McpServerRepository::new();
    let server = repo.create_server(req).await?;
    
    if server.enabled {
        MCP_CLIENT_REGISTRY.register_server(server.clone()).await;
    }
    
    Ok(Json(server))
}

pub async fn update_mcp_server(Path(id): Path<i64>, Json(req): Json<CreateMcpServerRequest>) -> ServiceResult<Json<McpServer>> {
    let repo = McpServerRepository::new();
    let server = repo.update_server(id, req).await?;
    
    MCP_CLIENT_REGISTRY.unregister_server(id).await;
    if server.enabled {
        MCP_CLIENT_REGISTRY.register_server(server.clone()).await;
    }
    
    Ok(Json(server))
}

pub async fn delete_mcp_server(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = McpServerRepository::new();
    repo.delete_server(id).await?;
    
    MCP_CLIENT_REGISTRY.unregister_server(id).await;
    
    Ok(Json(()))
}

pub async fn toggle_mcp_server(Path((id, enabled)): Path<(i64, bool)>) -> ServiceResult<Json<McpServer>> {
    let repo = McpServerRepository::new();
    let server = repo.set_enabled(id, enabled).await?;
    
    if enabled {
        MCP_CLIENT_REGISTRY.register_server(server.clone()).await;
    } else {
        MCP_CLIENT_REGISTRY.unregister_server(id).await;
    }
    
    Ok(Json(server))
}

pub async fn get_mcp_server_tools(Path(id): Path<i64>) -> ServiceResult<Json<McpServerWithTools>> {
    let repo = McpServerRepository::new();
    let server = repo.get_server(id).await?;
    
    let tools = if let Some(client) = MCP_CLIENT_REGISTRY.get_client(id).await {
        client.list_tools().await.unwrap_or_default()
    } else {
        vec![]
    };
    
    Ok(Json(McpServerWithTools { server, tools }))
}

pub async fn refresh_mcp_servers() -> ServiceResult<Json<()>> {
    let repo = McpServerRepository::new();
    let servers = repo.get_enabled_servers().await?;
    
    for server in servers {
        MCP_CLIENT_REGISTRY.register_server(server).await;
    }
    
    Ok(Json(()))
}
