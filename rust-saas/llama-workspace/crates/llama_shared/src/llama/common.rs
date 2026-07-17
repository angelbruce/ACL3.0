use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use core::marker::Send;
use std::result::Result;

use shared::errors::*;
use shared::models::{ToolCallInfo,ToolCallFunction,ChatMessage,StreamResponse,MCPTool};
use crate::llama::llm_web_openai::LlmWebOpenAi;
use crate::llama::llm_gemma4_local::LlmGemma4Local;
use crate::llama::tool_executor::ToolExecutor;
use async_stream::stream;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use futures_core::stream::Stream;

use log::info;
use tokio::sync::Mutex;

pub trait LlamaChat {
    async fn chat_stream(&mut self, 
        tool_executor: &ToolExecutor,
        chat_messages: &mut Vec<ChatMessage>, tools: Option<&[MCPTool]>) -> impl Stream<Item = Result<StreamResponse, String>>;
}

pub fn setup_logging() {
    // 尝试初始化 logger。如果已初始化，则忽略。
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info) // 设置最低显示级别为 INFO
        .try_init();
}


pub enum LlmType {
    Gemma4Local(LlmGemma4Local),
    OpenAI(LlmWebOpenAi),
}


unsafe impl Send for LlmType {} 


pub struct LlmProxy {
    pub options: HashMap<String, String>,
    llm_type: tokio::sync::Mutex<LlmType>
}

unsafe impl Send for LlmProxy {} 

impl LlmProxy {

    pub fn for_openai(base_url:String,api_key:String,model_name:String) -> Self {
          let open_ai_web = LlmWebOpenAi::new(base_url, api_key, model_name);
        Self {
            options: HashMap::new(),
            llm_type : Mutex::new(LlmType::OpenAI(open_ai_web))
        }
    }

    pub fn for_local(model_path:Option<String>, keep_kv_cache: Option<bool>) -> Self {
        let local = LlmGemma4Local::new(model_path, keep_kv_cache);
        Self {
            options: HashMap::new(),
            llm_type : Mutex::new(LlmType::Gemma4Local(local))
        }
    }
}

impl LlamaChat for LlmProxy {
    async fn chat_stream(&mut self, 
        tool_executor: &ToolExecutor,
        chat_messages: &mut Vec<ChatMessage>, 
        tools: Option<&[MCPTool]>) ->impl futures_core::stream::Stream<Item = Result<StreamResponse, String>> {
        let mut llm_type_guard = self.llm_type.lock().await; 
        async_stream::stream! {
            match &mut *llm_type_guard {
                LlmType::Gemma4Local(ref mut local) =>  {
                    let mut data_stream = local.chat_stream(tool_executor, chat_messages, tools).await;
                    pin_mut!(data_stream);
                    while let data = data_stream.next().await {
                        match data {
                            Some(dt) => {
                                match dt {
                                    Ok(resp) =>{
                                        yield Ok(resp.clone());
                                    }
                                    Err(e) => {
                                        yield Ok(StreamResponse::done())
                                    }
                                }
                            },
                            None => {
                                yield Ok(StreamResponse::done())
                            }
                        }
                    }
                },
                LlmType::OpenAI(ref mut web) =>  {
                    let mut data_stream = web.chat_stream(tool_executor, chat_messages, tools).await;
                    pin_mut!(data_stream);
                    while let data = data_stream.next().await {
                        match data {
                            Some(dt) => {
                                match dt {
                                    Ok(resp) =>{
                                        yield Ok(resp.clone());
                                    }
                                    Err(e) => {
                                        yield Ok(StreamResponse::done())
                                    }
                                }
                            },
                            None => {
                                yield Ok(StreamResponse::done())
                            }
                        }
                        
                    }
                } ,
            };

        }
    }
}

#[cfg(test)]
pub mod test_common {
    // use futures::StreamExt;
    use async_stream::stream;
    use futures_util::pin_mut;
    use futures_util::stream::StreamExt;
    use futures_core::stream::Stream;
    use crate::{llama::{common::*, tool_executor}, models::{MCPTool, UserSession}};
    use crate::models::StreamResponse;


    #[test]
    pub fn test_common_local () {
        super::setup_logging();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {

            let session  = Some(UserSession::new(0, 0, 0, 0));

            let mut llama = LlmProxy::for_local(None, Some(true));
            let mut tool_map = HashMap::new();
            tool_map.insert(0, "http://192.168.0.108:5000".to_string());
            let mut tool_executor = ToolExecutor::new(tool_map, "http:://localhost:5000");
            let mut chat_messages:Vec<ChatMessage> = vec![];
            chat_messages.push(ChatMessage{
                 role:"system".to_string(),
                 content:Some("你是一名英语老师".to_string()),
                 name:None,
                 tool_call_id:None,
                 tool_calls:None,
                 session : session.clone()
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
        });

    }



        #[test]
    pub fn test_common_open_ai () {
        super::setup_logging();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {

            let session  = UserSession::new(0, 0, 0, 0);

            let mut llama = LlmProxy::for_openai("http://localhost:18088".to_string(), "EMPTY".to_string(),"Gemma4-E4P".to_string());
            let mut tool_map = HashMap::new();
            tool_map.insert(0, "http://localhost:5000".to_string());
            let mut tool_executor = ToolExecutor::new(tool_map, "http:://localhost:5000");
            let mut chat_messages:Vec<ChatMessage> = vec![];
            chat_messages.push(ChatMessage{
                 role:"system".to_string(),
                 content:Some("你是一名数据分析师，阐述数据分析的全流程，以及使用的工具何理论。".to_string()),
                 name:None,
                 tool_call_id:None,
                 tool_calls:None,
                 session : session.clone()
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
                    }
                }
            }
        });

    }



}