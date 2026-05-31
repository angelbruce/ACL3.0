use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{ChatMessage};

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub content: String,
    pub done: bool,
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

    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        _tools: Option<&[shared::models::MCPTool]>,
    ) -> ServiceResult<Pin<Box<dyn Stream<Item = Result<StreamResponse, ServiceError>> + Send>>> {
        let body = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: messages.iter().cloned().collect(),
            stream: true,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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
                                let json_str = line.strip_prefix("data: ").unwrap_or(line);
                                if json_str == "[DONE]" {
                                    responses.push(Ok(StreamResponse {
                                        content: String::new(),
                                        done: true,
                                    }));
                                } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                                    let content = chunk.choices.first()
                                        .and_then(|c| c.delta.content.clone())
                                        .unwrap_or_default();
                                    let done = chunk.choices.first()
                                        .and_then(|c| c.finish_reason.as_ref())
                                        .map(|r| r == "stop")
                                        .unwrap_or(false);
                                    responses.push(Ok(StreamResponse {
                                        content,
                                        done,
                                    }));
                                }
                            }
                        }
                        
                        futures::stream::iter(responses)
                    }
                    Err(e) => futures::stream::iter(vec![Err(ServiceError::LlmError(e.to_string()))]),
                }
            });

        Ok(Box::pin(stream))
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
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
    content: Option<String>,
}
