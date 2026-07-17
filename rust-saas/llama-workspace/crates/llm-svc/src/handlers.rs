use axum::{extract::Path, Json, response::sse::{Event, Sse}};
use async_stream::stream;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use futures_core::stream::Stream;

use core::slice::Iter;
use std::{pin::Pin, thread};
use serde::{Serialize, Deserialize};
use shared::{errors::ServiceResult};
use shared::models::{LlmModel, LlmRequest, CreateLlmModelRequest, ChatMessage, MCPTool,StreamResponse};
use crate::repository::ModelRepository;
use crate::agent_repository::AgentRepository;
// use crate::client::{LlmClient, StreamResponse};
use tokio::runtime::Runtime;
use llama_shared::llama::common::*;
use llama_shared::llama::tool_executor::*;
use std::thread::JoinHandle;
use axum::response::IntoResponse;
use std::sync::Arc;

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

// pub async fn chat(Json(req): Json<LlmRequest>) -> ServiceResult<Json<ChatResponse>> {
//     let repo = ModelRepository::new();
//     let model = repo.get_model(req.model_id).await?;
    
//     let client = LlmClient::new(&model.access_url, &model.api_key, &model.name);
    
//     let (messages, tools) = prepare_messages_and_tools(&req).await?;
//     let tools_ref = tools.as_ref().map(|v| &**v);
    
//     let response = client.chat(&messages, tools_ref).await?;
    
//     Ok(Json(response))
// }

// pub async fn chat_stream(Json(req): Json<LlmRequest>) -> Sse<SSEStream> {
pub async fn chat_stream(Json(req): Json<LlmRequest>) ->impl IntoResponse {
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
    let (mut messages, tools) = match prepare_messages_and_tools(&req).await {
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
    
    let mcp_default_url = std::env::var("MCP_BASE_URL")
            .unwrap_or_else(|_| "http://mcp-svc:8080".to_string());


    let streamer = async_stream::stream! {
        let mut client = if model.access_url.is_empty()  {
            LlmProxy::for_local(None,None)
        } else  {
            LlmProxy::for_openai(model.access_url.clone(), model.api_key.clone(), model.name.clone())
        };
            
        let tool_executor = ToolExecutor::new(mcp_servers, mcp_default_url.as_str());

        let data_stream = client.chat_stream(&tool_executor
            ,&mut messages
            ,match tools {
                Some(ref ts) => Some(ts),
                None => None,
            }).await;


        pin_mut!(data_stream);

        while let Some(data) = data_stream.next().await {
            match data.clone() {
                Ok(d)=> {
                    yield data;
                    if d.done {
                        break;
                    }
                },
                Err(_)=>{
                    yield Ok(StreamResponse::done());
                    break ;
                }
            };
        }
    };
    

    let sse_stream = streamer.map(|res|{
                match res {
                    Ok(response) => {
                        match serde_json::to_string(&response) {
                            Ok(json_str) => Ok::<Event, std::convert::Infallible>(Event::default().data(json_str)),
                            Err(_) => Ok::<Event, std::convert::Infallible>(Event::default().data("{\"error\": \"Serialization error\"}")),
                        }
                    }
                    Err(_) => {
                        Ok::<Event, std::convert::Infallible>(Event::default().data(format!("{{\"error\": \"error found\"}}")))
                    }
                }
            });

    let sse_stream = Box::pin(sse_stream) as SSEStream;

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