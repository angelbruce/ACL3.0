use axum::{extract::Path, Json, response::sse::{Event, Sse}};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use serde::{Serialize, Deserialize};
use shared::errors::ServiceResult;
use shared::models::{LlmModel, LlmRequest, CreateLlmModelRequest, ChatMessage, MCPTool};
use crate::repository::ModelRepository;
use crate::agent_repository::AgentRepository;
use crate::client::{LlmClient, StreamResponse};

type SSEStream = Pin<Box<dyn Stream<Item = Result<Event, std::convert::Infallible>> + Send>>;

pub async fn get_models() -> ServiceResult<Json<Vec<LlmModel>>> {
    let repo = ModelRepository::new();
    let models = repo.get_all_models().await?;
    Ok(Json(models))
}

pub async fn get_model(Path(id): Path<i64>) -> ServiceResult<Json<LlmModel>> {
    let repo = ModelRepository::new();
    let model = repo.get_model(id).await?;
    Ok(Json(model))
}

pub async fn create_model(Json(req): Json<CreateLlmModelRequest>) -> ServiceResult<Json<LlmModel>> {
    let repo = ModelRepository::new();
    let created = repo.create_model(req).await?;
    Ok(Json(created))
}

pub async fn update_model(Path(id): Path<i64>, Json(req): Json<CreateLlmModelRequest>) -> ServiceResult<Json<LlmModel>> {
    let repo = ModelRepository::new();
    let updated = repo.update_model(id, req).await?;
    Ok(Json(updated))
}

pub async fn delete_model(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = ModelRepository::new();
    repo.delete_model(id).await?;
    Ok(Json(()))
}

pub async fn chat(Json(req): Json<LlmRequest>) -> ServiceResult<Json<ChatResponse>> {
    let repo = ModelRepository::new();
    let model = repo.get_model(req.model_id).await?;
    
    let client = LlmClient::new(&model.access_url, &model.api_key, &model.name);
    
    let (messages, tools) = prepare_messages_and_tools(&req).await?;
    let tools_ref = tools.as_ref().map(|v| &**v);
    
    let response = client.chat(&messages, tools_ref).await?;
    
    Ok(Json(response))
}

pub async fn chat_stream(Json(req): Json<LlmRequest>) -> Sse<SSEStream> {
    let repo = ModelRepository::new();
    let agent_repo = AgentRepository::new();
    // Try to get the model
    let model = match repo.get_model(req.model_id).await {
        Ok(m) => m,
        Err(e) => {
            let error_msg = format!("{{\"error\": \"Model not found: {}\"}}", e);
            let stream: SSEStream = Box::pin(futures::stream::once(async move {
                Ok::<Event, std::convert::Infallible>(Event::default().data(error_msg))
            }));
            return Sse::new(stream);
        }
    };
    
    // Prepare messages and tools based on agent_id
    let (messages, tools) = match prepare_messages_and_tools(&req).await {
        Ok(result) => result,
        Err(e) => {
            let error_msg = format!("{{\"error\": \"Failed to prepare agent context: {}\"}}", e);
            let stream: SSEStream = Box::pin(futures::stream::once(async move {
                Ok::<Event, std::convert::Infallible>(Event::default().data(error_msg))
            }));
            return Sse::new(stream);
        }
    };
    
    // Create LLM client
    let client = LlmClient::new(&model.access_url, &model.api_key, &model.name);
    
    // Get all MCP servers from database
    let mcp_servers = match agent_repo.get_all_mcp_servers().await {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("{{\"error\": \"Failed to get MCP servers: {}\"}}", e);
            let stream: SSEStream = Box::pin(futures::stream::once(async move {
                Ok::<Event, std::convert::Infallible>(Event::default().data(error_msg))
            }));
            return Sse::new(stream);
        }
    };
    
    // Get default MCP URL from environment
    let mcp_default_url = std::env::var("MCP_BASE_URL")
        .unwrap_or_else(|_| "http://mcp-svc:8080".to_string());

    // Create the appropriate stream based on whether we have tools
    let stream: Pin<Box<dyn Stream<Item = StreamResponse> + Send>> = if tools.is_some() {
        match client.chat_with_tools(messages, tools.as_ref().map(|v| &**v), mcp_servers, &mcp_default_url, 30).await {
            Ok(s) => s,
            Err(e) => {
                let stream: SSEStream = Box::pin(futures::stream::once(async move {
                    Ok::<Event, std::convert::Infallible>(Event::default().data(format!("{{\"error\": \"LLM error: {}\"}}", e)))
                }));
                return Sse::new(stream);
            }
        }
    } else {
        // No tools, use regular chat_stream
        match client.chat_stream(&messages, None).await {
            Ok(s) => Box::pin(s) as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>,
            Err(e) => {
                let stream: SSEStream = Box::pin(futures::stream::once(async move {
                    Ok::<Event, std::convert::Infallible>(Event::default().data(format!("{{\"error\": \"LLM error: {}\"}}", e)))
                }));
                return Sse::new(stream);
            }
        }
    };
    
    // Map the stream to SSE events
    let sse_stream: SSEStream = Box::pin(stream.map(|res: StreamResponse| {
        match serde_json::to_string(&res) {
            Ok(json_str) => Ok::<Event, std::convert::Infallible>(Event::default().data(json_str)),
            Err(_) => Ok::<Event, std::convert::Infallible>(Event::default().data("{\"error\": \"Serialization error\"}")),
        }
    }));
    
    Sse::new(sse_stream)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<shared::models::ToolCallInfo>>,
}

async fn prepare_messages_and_tools(req: &LlmRequest) -> ServiceResult<(Vec<ChatMessage>, Option<Vec<MCPTool>>)> {
    let mut messages = req.messages.clone();
    let mut tools: Option<Vec<MCPTool>> = None;
    
    if let Some(agent_id) = req.agent_id {
        let agent_repo = AgentRepository::new();
        
        let system_prompt = agent_repo.get_agent_system_prompt(agent_id).await?;
        if let Some(prompt) = system_prompt {
            messages.insert(0, ChatMessage {
                role: "system".to_string(),
                content: Some(prompt),
                ..Default::default()
            });
        }
        
        let agent_tools = agent_repo.get_agent_tools(agent_id).await?;
        if !agent_tools.is_empty() {
            tools = Some(agent_tools);
        }
    }
    
    Ok((messages, tools))
}