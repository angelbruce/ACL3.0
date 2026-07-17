// use futures::{Stream, StreamExt};
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use std::pin::Pin;
// use shared::errors::{ServiceError, ServiceResult};
// use shared::models::{ChatMessage, LlmTool,LlmToolFunction};
// use crate::tool::ToolExecutor;
// use crate::model::UserSession;

// #[derive(Debug, Serialize, Deserialize,Default)]
// pub struct StreamResponse {
//     pub content: String,
//     pub reasoning_content: Option<String>,
//     pub tool_calls: Option<Vec<ToolCallInfo>>,
//     pub finish_reason: Option<String>,
//     pub done: bool,
// }

// pub struct LlmClient {
//     base_url: String,
//     api_key: String,
//     model_name: String,
//     client: Client,
// }

// impl LlmClient {
//     pub fn new(base_url: &str, api_key: &str, model_name: &str) -> Self {
//         LlmClient {
//             base_url: base_url.to_string(),
//             api_key: api_key.to_string(),
//             model_name: model_name.to_string(),
//             client: Client::new(),
//         }
//     }

//     pub async fn chat_stream(
//         &self,
//         tool_executor: &ToolExecutor,
//         messages: &mut Vec<ChatMessage>,
//         tool_opts: Option<&[shared::models::MCPTool]>,
//     ) -> ServiceResult<Pin<Box<dyn Stream<Item = Result<StreamResponse, ServiceError>> + Send>>> {
//         let llm_tools = tool_opts.clone().map(|mcp_tools| {
//                 mcp_tools.iter().map(|tool| {
//                     LlmTool {
//                         r#type: "function".to_string(),
//                         function: LlmToolFunction {
//                             name: tool.name.clone(),
//                             description: tool.description.clone(),
//                             parameters: tool.input_schema.clone(),
//                         },
//                     }
//                 }).collect::<Vec<LlmTool>>()
//             });
//         let mut responses: Vec<Result<StreamResponse, ServiceError>> = vec![];
//         loop {
//             //构建请求体
//             let body = ChatCompletionRequest {
//                 model: self.model_name.clone(),
//                 messages: messages.iter().cloned().collect(),
//                 tools: llm_tools.clone(),
//                 stream: true,
//                 max_tokens: Some(4096),
//                 temperature: Some(0.7),
//             };

//             //获取本次请求响应流
//             let response = self.client
//                 .post(format!("{}/chat/completions", self.base_url))
//                 .header("Authorization", format!("Bearer {}", self.api_key))
//                 .header("Content-Type", "application/json")
//                 .json(&body)
//                 .send()
//                 .await
//                 .map_err(|e| ServiceError::LlmError(e.to_string()))?;

//             // 输出内容
//             let mut current_content = String::new();
//             // 推理过程
//             let mut current_reasoning_content = String::new();
//             // 调用的工具集合
//             let mut accumulated_tool_calls: Vec<ToolCallInfo> = Vec::new();
//             // 结束原因
//             let mut finish_reason: Option<String> = None;
//             // 是否包含函数调用，如果有函数调用，需要继续执行，否则本次执行结束
//             let mut has_tool_call_chunk = false;

//             let mut stream = response.bytes_stream();
            
//             let mut call_over = false;
//             while let Some(result) = stream.next().await {
//                 let bytes = result.map_err(|e| ServiceError::LlmError(e.to_string()))?;
//                 let lines = String::from_utf8_lossy(bytes.as_ref());
                   
//                 //逐行拼接获取内容、推理、工具
//                 for line in lines.lines() {
//                     let line = line.trim();
//                     if line.starts_with("data: ") {
//                         let json_str = line.strip_prefix("data: ").unwrap_or(line);
//                         if json_str == "[DONE]" {
//                             responses.push(Ok(StreamResponse {
//                                 content: String::new(),
//                                 reasoning_content: None,
//                                 tool_calls: None,
//                                 finish_reason: Some("stop".to_string()),
//                                 done: true,
//                             }));
//                         } else if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
//                             if let Some(choice) = chunk.choices.first() {
//                                 if let Some(ref content) = choice.delta.content {
//                                     current_content.push_str(content);
//                                 }
//                                 if let Some(ref reasoning) = choice.delta.reasoning_content {
//                                     current_reasoning_content.push_str(reasoning);
//                                 }

//                                 if let Some(ref tool_calls) = choice.delta.tool_calls {
//                                     for tc in tool_calls {
//                                         let idx = tc.index.unwrap_or(0) as usize;

//                                         if idx >= accumulated_tool_calls.len() {
//                                             accumulated_tool_calls.push(ToolCallInfo {
//                                                 id: None,
//                                                 tool_type: Some("function".to_string()),
//                                                 function: Some(ToolCallFunction {
//                                                     name: None,
//                                                     arguments: None,
//                                                 }),
//                                                 name: None,
//                                                 arguments: None,
//                                                 index: Some(idx as i32),
//                                             });
//                                         }


//                                         if !tc.id.is_none() {
//                                             accumulated_tool_calls[idx].id = tc.id.clone();
//                                         }
//                                         if let Some(ref tt) = tc.tool_type {
//                                             if !tt.is_empty() {
//                                                 accumulated_tool_calls[idx].tool_type = Some(tt.clone());
//                                             }
//                                         } else {
//                                             accumulated_tool_calls[idx].tool_type = Some("function".to_string());
//                                         }

//                                         if let Some(ref func) = tc.function {
//                                             if let Some(ref name) = func.name {
//                                                 if !name.is_empty() {
//                                                     if let Some(ref mut f) = accumulated_tool_calls[idx].function {
//                                                         f.name = Some(name.clone());
//                                                     }
//                                                 }
//                                             }
//                                             if let Some(ref args) = func.arguments {
//                                                 if !args.is_empty() {
//                                                     if let Some(ref mut f) = accumulated_tool_calls[idx].function {
//                                                         let before = f.arguments.clone().unwrap_or_default();
//                                                         f.arguments = Some(format!("{}{}", before, args));
//                                                         // println!("-- args: {}", f.arguments.clone().unwrap_or_default());
//                                                     }
//                                                 }
//                                             }
//                                         }

//                                         if let Some(ref args) = tc.arguments {
//                                             let args_str = args.to_string();
//                                             if !args_str.is_empty() {
//                                                 if let Some(ref mut f) = accumulated_tool_calls[idx].function {
//                                                     if f.arguments.clone().unwrap_or_default().is_empty() {
//                                                         f.arguments = Some(args_str);
//                                                     }
//                                                 }
//                                             }
//                                         }
//                                     }
//                                     has_tool_call_chunk = true;
//                                 }

//                                 if let Some(ref reason) = choice.finish_reason {
//                                     if reason == "tool_calls" {
//                                         finish_reason = Some(reason.clone());
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 //存在工具调用
//                 let has_tool_calls = has_tool_call_chunk && accumulated_tool_calls.iter().any(|tc| {
//                     tc.function.as_ref()
//                         .and_then(|f| f.name.as_ref())
//                         .map(|n| !n.is_empty())
//                         .unwrap_or(false) ||
//                     tc.name.as_ref().map(|n| !n.is_empty()).unwrap_or(false)
//                 });

//                 //没有工具，调用结束了，返回stream
//                 if !has_tool_calls {
            
//                     let reason_content = current_content.clone();
//                     let current_reasoning_content = current_reasoning_content.clone();
//                     let response = StreamResponse {
//                         content: current_content.clone(),
//                         reasoning_content: if current_reasoning_content.is_empty() {
//                             None
//                         } else {
//                             Some(current_reasoning_content.clone())
//                         },
//                         tool_calls: None,
//                         finish_reason: Some("stop".to_string()),
//                         done:true,
//                     };

//                     responses.push(Ok(response));
//                     //结束循环调用工具的过程。
//                     call_over = true;
//                     break;
//                 }


//                 for tc in &mut accumulated_tool_calls {
//                     if let Some(ref mut f) = tc.function {
//                         if f.arguments.clone().unwrap_or_default().is_empty() {
//                             f.arguments = Some("{}".to_string());
//                         }
//                     }
//                 }

//                 // begin tool call
//                 for tool_call in &accumulated_tool_calls {
//                     let tool_name = tool_call.function.as_ref()
//                         .and_then(|f| f.name.clone())
//                         .filter(|n| !n.is_empty())
//                         .or_else(|| tool_call.name.clone())
//                         .unwrap_or_default();

//                     let func_args_str = tool_call.function.as_ref()
//                         .and_then(|f| f.arguments.clone())
//                         .unwrap_or_default();
//                     println!("[ExecuteTool] name={}, func.arguments='{:?}'", tool_name, func_args_str);

//                     let tool_args = if !func_args_str.is_empty() {
//                         let trimmed = func_args_str.trim();
//                         if let Ok(value) = serde_json::from_str(trimmed) {
//                             println!("[ExecuteTool] Direct parse succeeded");
//                             value
//                         } else {
//                             if let Ok(inner_str) = serde_json::from_str::<String>(trimmed) {
//                                 println!("[ExecuteTool] Parsed as string first, inner value: '{}'", inner_str);
//                                 serde_json::from_str(&inner_str).unwrap_or_else(|e| {
//                                     println!("[ExecuteTool] Failed to parse inner string as JSON: {}, using empty object", e);
//                                     serde_json::json!({})
//                                 })
//                             } else {
//                                 println!("[ExecuteTool] Failed to parse arguments, using empty object");
//                                 serde_json::json!({})
//                             }
//                         }
//                     } else {
//                         println!("[ExecuteTool] function.arguments is empty, using arguments field or empty object");
//                         tool_call.arguments.clone().unwrap_or(serde_json::json!({}))
//                     };

//                     println!("[ExecuteTool] parsed arguments: {:?}", tool_args);

//                     if tool_name.is_empty() {
//                         continue;
//                     }


//                     let mut tool_args = tool_args.clone();
//                     println!("[ExecuteTool] original arguments: {:?}", tool_args.clone());
//                     let mut user_session = UserSession::new(tool_executor.get_user_id(),0,0);
//                     // TODO:
//                     // if(tool_name.contains("question") || tool_name.contains("task")) {
//                     //     let mut data =  tool_args.as_object_mut().unwrap();
//                     //     let mut session = Map::new();
//                     //     session.insert("session".to_string(), serde_json::json!(user_session));
//                     //     data.append( &mut session);
//                     //     tool_args = serde_json::json!(data);
                
//                     //     println!("[ExecuteTool] merged arguments: {:?}", tool_args);
//                     // }

//                     let mut server_id = None;
//                     if let Some(tools) = tool_opts {
//                         for tool in tools {
//                             if tool.name == tool_name {
//                                 server_id = tool.server_id;
//                                 break;
//                             }
//                         }
//                     }
                 

//                     let result = tool_executor.execute_tool(&tool_name, &tool_args, server_id).await;

//                     let tool_result = match result {
//                         Ok(content) => serde_json::json!({
//                             "success": true,
//                             "content": content
//                         }).to_string(),
//                         Err(e) => serde_json::json!({
//                             "success": false,
//                             "content": "",
//                             "error": e.to_string()
//                         }).to_string(),
//                     };

//                     messages.push(ChatMessage {
//                         role: "tool".to_string(),
//                         content: Some(tool_result),
//                         tool_call_id: tool_call.id.clone(),
//                         name: Some(tool_name),
//                         tool_calls: None,
//                     });

//                     responses.push(Ok(StreamResponse {
//                         content: current_content.clone(),
//                         reasoning_content: Some(current_reasoning_content.clone()),
//                         tool_calls: Some(vec![tool_call.clone()]),
//                         finish_reason: None,
//                         done: false
//                     }));
//                 }
//             }
       
//             if call_over {
//                 break;
//             }
//         }

//         let stream = Box::pin(futures::stream::iter(responses));// as Pin<Box<dyn Stream<Item = StreamResponse> + Send>>;
//         return Ok(stream);
//     }
// }

// #[derive(Debug, Serialize)]
// struct ChatCompletionRequest {
//     model: String,
//     messages: Vec<ChatMessage>,
//     stream: bool,
//     max_tokens: Option<i32>,
//     temperature: Option<f32>,
//     tools: Option<Vec<LlmTool>>,
// }

// #[derive(Clone,Default,Debug, Serialize, Deserialize)]
// pub struct ToolCallInfo {
//     pub id: Option<String>,
//     pub index : Option<i32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[serde(rename = "type")]
//     pub tool_type:Option<String>,
//     pub function:Option<ToolCallFunction>,
//     pub arguments:Option<serde_json::Value>,
//     pub name: Option<String>,
// }

// #[derive(Clone,Default,Debug, Serialize, Deserialize)]
// pub struct ToolCallFunction {
//     pub name: Option<String>,
//     pub arguments: Option<String>,
// }