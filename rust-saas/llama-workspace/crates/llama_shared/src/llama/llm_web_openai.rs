use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use core::marker::Send;
use reqwest::Client;

use crate::llama::gemma4_local::*;
use shared::errors::*;
use shared::models::{MCPTool, LlmToolFunction, LlmTool,ToolCallInfo,ToolCallFunction, 
    ChatMessage, StreamChunk,StreamResponse,ChatCompletionRequest,UserSession
};
use crate::llama::tool_executor::ToolExecutor;

use futures_util::stream::StreamExt;
use futures_util::pin_mut;
use futures_core::stream::Stream;
use async_stream::stream;
use crate::llama::common::{LlamaChat};
use log::info;
use log::error;

pub struct LlmWebOpenAi {
    pub base_url: String,
    pub api_key: String,
    pub model_name: String,
    client: Client,
}

unsafe impl Send for LlmWebOpenAi  {} 

impl LlmWebOpenAi{
    pub fn new(base_url:String,api_key:String,model_name:String) -> Self {
        Self { 
            base_url, 
            api_key, 
            model_name,
            client: Client::new(),
         }
    }
}

impl LlamaChat for LlmWebOpenAi {

    async fn chat_stream(&mut self,
        tool_executor: &ToolExecutor,
        messages: &mut Vec<ChatMessage>, tool_opts: Option<&[MCPTool]>) -> impl Stream<Item = Result<StreamResponse, String>> {
                        println!("3!!");
        let llm_tools = tool_opts.clone().map(|mcp_tools| {
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

        let llm_tools = None;

        let session = messages[0].session.clone();

        async_stream::stream! {
            loop {
                //构建请求体
                let body = ChatCompletionRequest {
                    model: self.model_name.clone(),
                    messages: messages.clone(),
                    tools: llm_tools.clone(),
                    stream: Some(true),
                    max_tokens: Some(4096),
                    temperature: Some(0.7),
                };

                info!("body: {}", serde_json::to_string(&body).unwrap());

                //获取本次请求响应流
                let response_ret = self.client
                    .post(format!("{}/chat/completions", self.base_url))
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| ServiceError::LlmError(e.to_string()));

                let response = match response_ret {
                    Ok(response) => response,
                    Err(_) =>{ 
                        info!("222");
                        return;
                    }
                };

                // let response = response_ret.unwrap();

                // 输出内容
                let mut current_content = String::new();
                // 推理过程
                let mut current_reasoning_content = String::new();
                // 调用的工具集合
                let mut accumulated_tool_calls: Vec<ToolCallInfo> = Vec::new();
                // 结束原因
                let mut finish_reason: Option<String> = None;
                // 是否包含函数调用，如果有函数调用，需要继续执行，否则本次执行结束
                let mut has_tool_call_chunk = false;

                let mut stream = response.bytes_stream();
                
                // let mut call_over = false;

                while let Some(result) = stream.next().await {
                    current_content.clear();
                    current_reasoning_content.clear();
                    accumulated_tool_calls.clear();
                    finish_reason = None;

                    let bytes_result = result.map_err(|e| ServiceError::LlmError(e.to_string()));
                    let bytes = match bytes_result {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            return;
                        }
                    };

                    let mut lines = String::from_utf8_lossy(bytes.as_ref());
                    
                    //逐行拼接获取内容、推理、工具
                    for line in lines.lines() {
                        // info!("line:{}",line.clone());
                        let line = line.trim();
                       
                        if line.starts_with("data: ") {
                            let json_str = line.strip_prefix("data: ").unwrap_or(line);
                            if json_str == "[DONE]" {
                                yield Ok(StreamResponse {
                                    content: String::new(),
                                    reasoning_content: None,
                                    tool_calls: None,
                                    finish_reason: Some("stop".to_string()),
                                    done: true,
                                });
                            } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(ref content) = choice.delta.content {
                                        current_content.push_str(content);
                                    }

                                    if let Some(ref reasoning) = choice.delta.reasoning_content {
                                        current_reasoning_content.push_str(reasoning);
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


                    //存在工具调用
                    let has_tool_calls = has_tool_call_chunk && accumulated_tool_calls.iter().any(|tc| {
                        tc.function.as_ref()
                            .and_then(|f| f.name.as_ref())
                            .map(|n| !n.is_empty())
                            .unwrap_or(false) ||
                        tc.name.as_ref().map(|n| !n.is_empty()).unwrap_or(false)
                    });

                    //没有工具，调用结束了，返回stream
                    if !has_tool_calls {
                        // let reason_content = current_content.clone();
                        let current_reasoning_content = current_reasoning_content.clone();
                        let response = StreamResponse {
                            content: current_content.clone(),
                            reasoning_content: if current_reasoning_content.is_empty() {
                                None
                            } else {
                                Some(current_reasoning_content.clone())
                            },
                            tool_calls: None,
                            finish_reason: None,
                            done:false,
                        };

                        // info!("response:{}",serde_json::to_string(&response).unwrap());
                        yield Ok(response);
                        //结束循环调用工具的过程。
                        // call_over = true;
                        // break;
                    }


                    for tc in &mut accumulated_tool_calls {
                        if let Some(ref mut f) = tc.function {
                            if f.arguments.clone().unwrap_or_default().is_empty() {
                                f.arguments = Some("{}".to_string());
                            }
                        }
                    }

                    // begin tool call
                    for tool_call in &accumulated_tool_calls {
                        let tool_name = tool_call.function.as_ref()
                            .and_then(|f| f.name.clone())
                            .filter(|n| !n.is_empty())
                            .or_else(|| tool_call.name.clone())
                            .unwrap_or_default();

                        let func_args_str = tool_call.function.as_ref()
                            .and_then(|f| f.arguments.clone())
                            .unwrap_or_default();
                        info!("[ExecuteTool] name={}, func.arguments='{:?}'", tool_name, func_args_str);

                        let tool_args = if !func_args_str.is_empty() {
                            let trimmed = func_args_str.trim();
                            if let Ok(value) = serde_json::from_str(trimmed) {
                                info!("[ExecuteTool] Direct parse succeeded");
                                value
                            } else {
                                if let Ok(inner_str) = serde_json::from_str::<String>(trimmed) {
                                    info!("[ExecuteTool] Parsed as string first, inner value: '{}'", inner_str);
                                    serde_json::from_str(&inner_str).unwrap_or_else(|e| {
                                        info!("[ExecuteTool] Failed to parse inner string as JSON: {}, using empty object", e);
                                        serde_json::json!({})
                                    })
                                } else {
                                    info!("[ExecuteTool] Failed to parse arguments, using empty object");
                                    serde_json::json!({})
                                }
                            }
                        } else {
                            info!("[ExecuteTool] function.arguments is empty, using arguments field or empty object");
                            tool_call.arguments.clone().unwrap_or(serde_json::json!({}))
                        };

                        info!("[ExecuteTool] parsed arguments: {:?}", tool_args);

                        if tool_name.is_empty() {
                            continue;
                        }


                        let mut tool_args = tool_args.clone();
                        info!("[ExecuteTool] original arguments: {:?}", tool_args.clone());
                        let mut user_session = session.clone();
                        // TODO:
                        // if(tool_name.contains("question") || tool_name.contains("task")) {
                        //     let mut data =  tool_args.as_object_mut().unwrap();
                        //     let mut session = Map::new();
                        //     session.insert("session".to_string(), serde_json::json!(user_session));
                        //     data.append( &mut session);
                        //     tool_args = serde_json::json!(data);
                    
                        //     println!("[ExecuteTool] merged arguments: {:?}", tool_args);
                        // }

                        let mut server_id = None;
                        if let Some(tools) = tool_opts {
                            for tool in tools {
                                if tool.name == tool_name {
                                    server_id = tool.server_id;
                                    break;
                                }
                            }
                        }
                    

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

                        messages.push(ChatMessage {
                            role: "tool".to_string(),
                            content: Some(tool_result),
                            tool_call_id: tool_call.id.clone(),
                            name: Some(tool_name),
                            tool_calls: None,
                            session: session.clone()
                        });

                        yield Ok(StreamResponse {
                            content: current_content.clone(),
                            reasoning_content: Some(current_reasoning_content.clone()),
                            tool_calls: Some(vec![tool_call.clone()]),
                            finish_reason: None,
                            done: false
                        });
                    }
                }
            }

        }
    }


}

#[cfg(test)]
mod llm_web_openai_test {

    use super::*;
    use log::error;
    use log::info;

    use futures_util::stream::StreamExt;
    use futures_util::pin_mut;
    use futures_core::stream::Stream;
    use async_stream::stream;

    pub fn setup_logging() {
    // 尝试初始化 logger。如果已初始化，则忽略。
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info) // 设置最低显示级别为 INFO
        .try_init();
}

    #[tokio::test]
    pub async fn test_llm_web_open_ai_chat() {
        setup_logging();
        info!("** test test_llm_web_open_ai_chat ** begin..");


        let session  = UserSession::new(0, 0, 0, 0);
        let mut llama = LlmWebOpenAi::new("http://localhost:18088".to_string(), "EMPTY".to_string(),"Gemma4-E4P".to_string());
        let mut tool_map = HashMap::new();
        tool_map.insert(0, "http://localhost:5000".to_string());
        let tool_executor = ToolExecutor::new(tool_map, "http:://localhost:5000");
        let mut chat_messages:Vec<ChatMessage> = vec![];
        chat_messages.push(ChatMessage{
            role:"user".to_string(),
            content:Some("你是一名航天科学家，阐述如何制作火箭的详细步骤以及注意事项。".to_string()),
            name:None,
            tool_call_id:None,
            tool_calls:None,
            session : Some(session.clone())
        });
        let tools:Vec<MCPTool> = vec![];
        let mut stream = llama.chat_stream(&tool_executor, &mut chat_messages, Some(&tools)).await;
        pin_mut!(stream);
        while let data = stream.next().await {
            match data {
                Some( msg) => {
                    match msg {
                        Ok( response) => {
                            info!("{}",response.content.trim());
                            if response.done {
                                break;
                            }
                        },
                        Err(e) => {
                                info!(" error received. ");
                            error!("error:{}",e.to_string());
                            break;
                        }
                    }
                },
                None => {
                    info!(" none received. ");
                }
            }
        }
    
    
        info!("..done..");

    }
}
