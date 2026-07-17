use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use core::marker::Send;
use std::thread;
use reqwest::Client;
use futures_util::stream::StreamExt;
use futures_core::stream::Stream;

use crate::llama::gemma4_local::*;
use shared::errors::*;
use shared::models::{MCPTool, LlmToolFunction, LlmTool,ToolCallInfo,ToolCallFunction, 
    ChatMessage, StreamChunk,StreamResponse,ChatCompletionRequest,UserSession
};


use crate::llama::common::LlamaChat;
use crate::llama::tool_executor::ToolExecutor;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use log::info;

pub struct LlmGemma4Local {
    model_path : Option<String>,
    keep_kv_cache: Option<bool>,
    wrapper: Arc<Mutex<LLMLlamaWrapper>>,
}

unsafe impl Send for LlmGemma4Local  {} 

impl LlmGemma4Local {
    pub fn new (model_path:Option<String>, keep_kv_cache: Option<bool>) ->Self {
        Self {
            model_path : model_path.clone(),
            keep_kv_cache,
            wrapper : Arc::new(Mutex::new(LLMLlamaWrapper::new (model_path.clone(),keep_kv_cache)))
        }
    }

    
    pub fn get_model_path(&self)-> Option<String> {
        self.model_path.clone()
    }

    pub fn get_keep_kv_cache(&self)-> Option<bool> {
        self.keep_kv_cache.clone()
    }
}


impl LlamaChat for LlmGemma4Local {

   async fn chat_stream(&mut self, 
        tool_executor: &ToolExecutor,
        chat_messages: &mut Vec<ChatMessage>, 
        tool_opts: Option<&[MCPTool]>) -> impl Stream<Item = Result<StreamResponse, String>> {

        let llm_tools: Option<Vec<LlmTool>> = tool_opts.clone().map(|mcp_tools| {
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

        let def_session = UserSession::new(0, 0, 0, 0);
        let tools_desc = serde_json::to_string(&llm_tools);
        // info!("---------------tool_desc:{}----",tools_desc.unwrap().clone());
        let messages = chat_messages.iter()
        .filter(|e| match e.content {
            Some(_) => true,
            None=> false
        })
        .map(|e|
            match &e.session {
                Some(d) => {
                    ChatMsg::new(
                        d.user_id,
                        d.project_id,
                        d.flow_id,
                        d.workspace_id,
                        e.role.clone(),
                        e.content.clone().unwrap()
                    )
                },
                None => {
                    ChatMsg::new(
                        def_session.user_id,
                        def_session.project_id,
                        def_session.flow_id,
                        def_session.workspace_id,
                        e.role.clone(),
                        e.content.clone().unwrap()
                    )
                }
            }
         ).collect();

        let (tx,mut rx) = std::sync::mpsc::channel::<ChatMsgPiece>();
        let mut wrapper = self.wrapper.clone();
        thread::spawn(move ||{
            let mut wrapper = wrapper.clone();
            let mut wrapper = wrapper.lock().unwrap();
            let mut wrapper = &mut *wrapper;
            let mut ret = wrapper.chat_message_stream(messages,llm_tools.clone(),tx);

        });

         async_stream::stream! {
            while let data = rx.recv() {
                // info!("----------===============---------------");
                if let Ok(piece) = data {
                    if piece.done {
                        yield Ok(
                            StreamResponse {
                                content: piece.token,
                                reasoning_content : None,
                                tool_calls: None,
                                finish_reason: None,
                                done: true,
                            }
                        );
                        break;
                    } else {
                        yield Ok(
                            StreamResponse {
                                content: piece.token,
                                reasoning_content : None,
                                tool_calls: None,
                                finish_reason: None,
                                done: false,
                            }
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_llm_gemma4 {
    use super::*;
    use async_stream::stream;
    use futures_util::pin_mut;
    use futures_util::stream::StreamExt;
    use futures_core::stream::Stream;
    use crate::{llama::{common::*, tool_executor}, models::{MCPTool, UserSession}};
    use crate::models::StreamResponse;
    use log::info;

       // 💡 重要的辅助函数：在测试开始前初始化 logger
    fn setup_logging() {
        // 尝试初始化 logger。如果已初始化，则忽略。
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info) // 设置最低显示级别为 INFO
            .try_init();
    }


    #[tokio::test]
    pub async fn test_llm_gemma4_local() {
        setup_logging();
        let stdin = std::io::stdin();

        let session  = UserSession::new(0, 0, 0, 0);
        let mut llama = LlmGemma4Local::new(None, Some(true));
        let mut tool_map = HashMap::new();
        tool_map.insert(0, "http://192.168.0.108:5000".to_string());
        let tool_executor = ToolExecutor::new(tool_map, "http:://localhost:5000");
        let mut chat_messages:Vec<ChatMessage> = vec![];
        chat_messages.push(ChatMessage{
                role:"system".to_string(),
                content:Some("你是以为PLC高级工程师，请阐述如何为机器变成的流程以及代码，内容简短不废话。".to_string()),
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
                            info!("{}",response.content);
                            if response.done {
                                break;
                            }
                        },
                        Err(e) => {
                            info!("error:{}",e.to_string());
                            break;
                        }
                    }
                },
                None => {
                    info!("21");
                }
            }
        }
    
    
        info!("..done..");
    }
}