use llama_cpp_4::prelude::*;
use llama_cpp_4::StringFromModelError;
use llama_cpp_4::StringToTokenError;
use std::num::*;
use log::info;
use std::path::Path;
use std::env;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::mem;
use core::marker::Send;
use std::collections::*;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::pin_mut;


use shared::models::{MCPTool, LlmToolFunction, LlmTool,ToolCallInfo,ToolCallFunction, 
    ChatMessage, StreamChunk,StreamResponse,ChatCompletionRequest,UserSession
};

#[derive(Debug,Clone)]
pub struct ChatMemory {
    //历史记录-长期记忆，
    histories : Vec<ChatMsg>,
    //短期记忆，最多保留多少条
    last_to_keep: i64,
}

impl ChatMemory {

    pub fn new (last_to_keep: i64) -> Self{
        Self { histories: vec![] , last_to_keep: last_to_keep }
    }

    pub fn push(&mut self, chat_msg: ChatMsg) -> &Vec<ChatMsg>{
        self.histories.push(chat_msg);
        &self.histories
    }

    pub fn clear(&mut self) {
        self.histories.clear();
    }

    pub fn take(&self,cnt: usize) -> Vec<ChatMsg> {
        if cnt <=0 {
            self.histories.clone()
        } else {
            let t = self.histories.len();
            let datas = &self.histories[t-cnt..t];
            let mut v = vec![];
            for d in datas{
                 v.push(d.clone())
            }
            v
        }
    }

    pub fn is_empty(&self) -> bool  {
        self.histories.is_empty()
    }   
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LlmError(LlmErrorKind);
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LlmErrorKind {
    Error,
    ModelCannotFound(String),
    IllegalRole,
    IllegalContent,
}

impl From<ApplyChatTemplateError> for LlmError {
    fn from(value: ApplyChatTemplateError) -> Self {
        Self(LlmErrorKind::Error)
    }
}

impl From<StringFromModelError> for LlmError {
    fn from(value: StringFromModelError) -> Self {
        Self(LlmErrorKind::Error)
    }
}

impl From<StringToTokenError> for LlmError {
    fn from(value: StringToTokenError) -> Self {
        Self(LlmErrorKind::Error)
    }
}

impl From<BatchAddError> for LlmError {
    fn from(value: BatchAddError) -> Self {
        Self(LlmErrorKind::Error)
    }
}

pub type LlmResult<T>=std::result::Result<T,LlmError>;

#[derive(Debug,Clone)]
pub struct ChatMemoryStore {
    memory : ChatMemory,
}

unsafe impl Send for ChatMemoryStore{}
unsafe impl Send for ChatMemory{}

impl ChatMemoryStore {
    pub fn new()->Self {
        Self{ memory: ChatMemory::new(64) }
    }

    pub fn push(&mut self,msg: ChatMsg) -> &ChatMemory {
        self.memory.push(msg);
        &self.memory
    }

    pub fn push_all(&mut self,msgs: &[ChatMsg]) -> &ChatMemory {
        for msg in msgs {
            self.memory.push(msg.clone());
        }

        &self.memory
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }

    pub fn get_memory(&self) -> &ChatMemory {
        &self.memory
    }
}

#[derive(Debug,Clone)]
pub struct ChatMsg {
    user_id: i64,
    project_id : i64,
    flow_id: i64,
    workspace_id: i64,
    role:String,
    content:String,
    store:bool    
}




impl ChatMsg {
    pub fn new(user_id:i64,project_id:i64,flow_id:i64,workspace_id:i64,role:String,content:String) -> Self {
        Self {
            user_id,
            project_id,
            flow_id,
            workspace_id,
            role,
            content,
            store:false,
        }
    }

    pub fn to_llama_msg(&self) -> LlamaChatMessage {
        let content =  format!("/{}/{}/{}/{}/:{}",
            self.user_id,
            self.project_id,
            self.flow_id,
            self.workspace_id,
            self.content);
        LlamaChatMessage::new(self.role.clone(),self.content.clone()).expect("allocat data space failed.")
    }
}


struct ModelInfo {
    pub model: LlamaModel,
    pub backend: LlamaBackend,
}

static MODEL_INSTANCE_GEMMA4:OnceCell<Arc<ModelInfo>> = OnceCell::new();

fn get_model_instance_gemma4(model_full_path:Option<String>) -> Arc<ModelInfo> {
    let model_path = match model_full_path {
        Some(path) => path.clone(),
        None => {
            dotenv::dotenv().ok();
            env::var("gemma4_model_path").unwrap_or_else(|_| "J:/gemma4/models--unsloth--gemma-4-E4B-it-GGUF/snapshots/ce152932ac27bc40bc9c727386760424d50bb456/gemma-4-E4B-it-Q4_0.gguf".to_string())
        },
    };
    MODEL_INSTANCE_GEMMA4.get_or_init(|| {
        let backend = LlamaBackend::init().expect("init backend failed");
        let mut model_params = LlamaModelParams::default();
        model_params = model_params.with_main_gpu(1).with_n_gpu_layers(99);
        let model = LlamaModel::load_from_file(&backend,  Path::new(&model_path), &model_params).expect("load model failed");
        Arc::new(ModelInfo { model, backend })
    }).clone()
}

pub struct LLMLlamaWrapper{
    store: ChatMemoryStore,
    model_info: Arc<ModelInfo>,
    //上下文
    // 1 上下文满时的自动处理：Context Shift , 当 KV Cache 的 token 数量达到 n_ctx 上限时，llama.cpp 会自动执行 context shift
    // 2 TurboQuant（编译期绑定机制） 内置了 TurboQuant（TQ） 动态 KV Cache 压缩机制，但它不是运行时开关，需要编译时开启 编译时需加 -DLLAMA_TURBOQUANT=O 模型文件必须包含 TURBO_SCALE 张量（TurboQuant-ready GGUF）
    context: LlamaContext<'static>,
    sampler: LlamaSampler,
    kv_cache_pos : i32,
    keep: bool,
}


unsafe impl Send for LLMLlamaWrapper{}

impl LLMLlamaWrapper {

    pub fn new (model:Option<String>, keep_kv_cache: Option<bool>) -> Self {
        let model_info = get_model_instance_gemma4(None).clone();
        let ctx_params = LlamaContextParams::default().with_n_threads(10).with_n_batch(4096).with_n_ctx(Some(NonZeroU32::new(4096).unwrap()));
        let ctx =  unsafe { 
            let model_ref : &'static LlamaModel = mem::transmute(&model_info.model);
            let backend_ref :&'static LlamaBackend = mem::transmute(&model_info.backend);
            model_ref.new_context(&backend_ref, ctx_params).expect("create context failed")
        };
        let store = ChatMemoryStore::new();
        let sampler = LlamaSampler::chain(vec![
            LlamaSampler::top_k(50),
            // LlamaSampler::top_p(0.9,1),
            LlamaSampler::temp(0.8),
            LlamaSampler::dist(30),

        ], false);

        Self {
            model_info: model_info.clone(),
            context: ctx,
            store,
            sampler,
            kv_cache_pos: 0,
            keep: keep_kv_cache.unwrap_or(false),
        }
    }


    pub  fn chat_message_stream(&mut self, msges : Vec<ChatMsg>, tools: Option<Vec<LlmTool>>, response_sender:std::sync::mpsc::Sender<ChatMsgPiece>) -> LlmResult<bool> {
        if ! self.keep {
            self.kv_cache_pos = 0;
            self.context.clear_kv_cache();
        }

        let msg = msges.first().unwrap().clone();
        let mut chat_messages: Vec<LlmResult<ChatMsgPiece>> =  vec![];

        info!("kv_cache_pos:{}",self.kv_cache_pos);
        let memory = if self.keep {
            //缓存模式，只在第一次加tool，可以设置事件或者开关，暂时先这样
            if let Some(llm_tools) = tools {
                if !self.store.get_memory().is_empty() {
                    let tools_desc = serde_json::to_string(&llm_tools).unwrap();
                    let tool_msg = ChatMsg::new (
                        msg.user_id.clone(),
                        msg.project_id.clone(),
                        msg.flow_id.clone(),
                        msg.workspace_id.clone(),
                        "system".to_string(), 
                        format!("你可以使用的工具如下:{},函数调用的结果只能输出json，输出json采用open ai的格式，json的schema的格式样例如下：{}"
                        , tools_desc
                        ,serde_json::json!({
                            "tool_call": {
                                "function_name": "get_weather",
                                "arguments": {
                                    "city": "London",
                                    "unit": "celsius"
                                }
                            }
                        }).to_string())
                    );
                    self.store.push(tool_msg);                
                }
            }
            
            let last = msges.last().clone();
            match last {
                Some(msg) => self.store.push(msg.clone()),
                None => self.store.get_memory()
            }
        } else {
            self.store.clear();
            self.store.push_all(&msges)
        };

        //接管所有权
        let session = memory.take(0);
        let llm_session: Vec<LlamaChatMessage> = if self.keep {
            vec![LlamaChatMessage::new(msg.role.clone(),msg.content.clone()).expect("allocate data space failed.")]
        } else {
            session.iter().map(|e| e.to_llama_msg()).collect()
        };

        let chat_template_message_ret = self.context.model.apply_chat_template(None, &llm_session,true);
        let chat_template_message = match chat_template_message_ret {
            Ok(m) => { info!("{}",m.clone()); m},
            Err(e) => {
                response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                return Err(LlmError(LlmErrorKind::Error));
            }
        };
        info!("chat_template_message:{}",chat_template_message.clone());
        let tokens_ret = self.context.model.str_to_token(chat_template_message.as_str(),  AddBos::Always);
        let tokens = match tokens_ret {
            Ok(t) => t,
            Err(e) => {
                response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                return Err(LlmError(LlmErrorKind::Error));
            }
        };

        let eos = self.context.model.token_eos();

        let mut generate_tokens = vec![];
        let mut generate_text = String::new();
        let mut next_token_id:i32 = self.context.model.token_eos().0;
        let cnt = tokens.len();
        // info!("cnt:{}",cnt);
        {
            let mut batch = LlamaBatch::new(cnt, 1);
            for (i,&token) in tokens.iter().enumerate() {
                let logits = (i+1) == cnt;
                let pos = self.kv_cache_pos + i as i32;
                let batch_add_ret = batch.add(token,pos,&[0],logits).map_err(|e| LlmError(LlmErrorKind::Error));
                match batch_add_ret {
                    Ok(_) => {},
                    Err(e) => {
                        response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                        return Err(LlmError(LlmErrorKind::Error));
                    }
                }
            }

            let decode_ret = self.context.decode(&mut batch).map_err(|e| LlmError(LlmErrorKind::Error));
            match decode_ret {
                Ok(_) => {},
                Err(e) => {
                    response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                    return Err(LlmError(LlmErrorKind::Error));
                }
            }

            let decode_pos = cnt as i32 - 1;
            // info!("decode_pos:{}",decode_pos);
            let next_token = self.sampler.sample(&self.context,decode_pos);
            self.sampler.accept(next_token);

            next_token_id = next_token.0;
            generate_tokens.push(next_token_id);
            let next_token_ret = self.context.model.detokenize(&[LlamaToken(next_token_id)], false, false);
            let next_token = match next_token_ret {
                Ok(t) => t,
                Err(e) => {
                    response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                    return Err(LlmError(LlmErrorKind::Error));
                }
            };

            generate_text.push_str(next_token.as_str());
            // info!("next_token_id:{}",next_token_id);
            // info!("next_token:{}",next_token.clone());
            // info!("output:{}",generate_text.clone());
            if next_token.ends_with("\n") || next_token.ends_with("\r") {
                // info!("send");
                response_sender.send(ChatMsgPiece::new(generate_tokens.clone(), generate_text.clone(), false, None));
                generate_tokens.clear();
                generate_text.clear();
            }
        }

        let max_tokens = 2147483640;
        let mut pos = self.kv_cache_pos + cnt as i32;
        for _ in 0..max_tokens {
            if next_token_id == eos.0 {
                if !generate_text.is_empty() {
                    let ret = response_sender.send(ChatMsgPiece::new(generate_tokens.clone(), generate_text.clone(), true, None));
                    match ret {
                        Ok(_) => {},
                        Err(e) => { info!("{}",e.to_string()); }
                    }
                    generate_tokens.clear();
                    generate_text.clear();
                }
                break;
            }

            self.sampler.reset();
            let mut batch = LlamaBatch::new(1, 1);
            let add_ret = batch.add(LlamaToken(next_token_id), pos,&[0],true).map_err(|e| LlmError(LlmErrorKind::Error));
            match add_ret {
                Ok(_) => {},
                Err(e) => {
                    response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                    return Err(LlmError(LlmErrorKind::Error));
                }
            }

            let decode_ret = self.context.decode(&mut batch).map_err(|e| LlmError(LlmErrorKind::Error));
            match decode_ret {
                Ok(_) => {},
                Err(e) => {
                    response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                    return Err(LlmError(LlmErrorKind::Error));
                }
            }

            let next_token = self.sampler.sample(&self.context,0);

            self.sampler.accept(next_token);
            next_token_id = next_token.0;
            generate_tokens.push(next_token_id.clone());

            if next_token_id == eos.0 {
                pos += 1;
                // info!("pos:{}",pos.clone());
                generate_text.push_str("\n");
                response_sender.send(ChatMsgPiece::new(generate_tokens.clone(), generate_text.clone(), true, None));
                generate_tokens.clear();
                generate_text.clear();
                
                break;
            }

           
            let next_token = self.context.model.detokenize(&[LlamaToken(next_token_id)], false, false);
            let next_token = match next_token {
                Ok(token) => token,
                Err(e) => {
                    response_sender.send(ChatMsgPiece::new(vec![], "".to_string(), true, Some(LlmError(LlmErrorKind::Error))));
                    return Err(LlmError(LlmErrorKind::Error));
                }
            };
            generate_text.push_str(next_token.as_str());
            // info!("next_token_id:{}",next_token_id);
            // info!("next_token:{:}",next_token.clone());
            // info!("output:{}",generate_text.clone());
            // info!("pos:{}",pos.clone());

            pos += 1;
            if next_token.ends_with("\n") || next_token.ends_with("\r"){
                // info!("send");
                let ret = response_sender.send(ChatMsgPiece::new(generate_tokens.clone(), generate_text.clone(), false, None));
                match ret {
                    Ok(_) => {},
                    Err(e) => { info!("{}",e.to_string()); }
                }
                generate_tokens.clear();
                generate_text.clear();
            }
            // info!("pos:{}",pos.clone());
        }

        self.kv_cache_pos = pos;



        return Ok(true);
      
    }

}

#[derive(Debug,Clone)]
pub struct ChatMsgPiece {
    pub token_id: Vec<i32>,
    pub token: String,
    pub done: bool,
    pub err: Option<LlmError>,
}

impl ChatMsgPiece {
    pub fn new(token_id: Vec<i32>, token: String, done: bool, err: Option<LlmError>) -> Self {
        Self {
            token_id,
            token,
            done,
            err,
        }
    }
}


#[cfg(test)] 
mod tests{
    use super::*;
    use std::{thread, time::Duration};
    use tokio::sync::mpsc::{Sender,Receiver};
    use std::sync::Mutex;
    use async_stream::stream;

    // 💡 重要的辅助函数：在测试开始前初始化 logger
    fn setup_logging() {
        // 尝试初始化 logger。如果已初始化，则忽略。
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info) // 设置最低显示级别为 INFO
            .try_init();
    }

    pub fn new(model:Option<String>, keep_kv_cache: Option<bool>) -> Arc<Mutex<LLMLlamaWrapper>> {
         // 這裡的 LLMLlamaWrapper 是一個結構體，它需要被保護起來。
        let wrapper = LLMLlamaWrapper::new(model, keep_kv_cache);
        Arc::new(Mutex::new(wrapper))
    }

    
    #[tokio::test]
    pub async fn generate_test() {
        setup_logging(); 
        let mut datas = create_generate_test().await;
        pin_mut!(datas);
        while let Some(data) = datas.next().await {
            match data {
                Ok(data) => {
                    info!("{}",data.token);
                },
                Err(_) =>{}
            }
        }
    }
    
    async fn create_generate_test() ->  impl Stream<Item = std::result::Result<ChatMsgPiece, LlmError>> {
        let model_path = "J:/gemma4/models--unsloth--gemma-4-E4B-it-GGUF/snapshots/ce152932ac27bc40bc9c727386760424d50bb456/gemma-4-E4B-it-Q4_0.gguf";
        let mut wrapper = new (Some(model_path.to_string()),Some(true));
       
        let stdin = std::io::stdin();

        let mut wrapper = wrapper.clone();
        let mut input = String::new();
        stdin.read_line(&mut input);
        
        let (tx,mut rx) = std::sync::mpsc::channel::<ChatMsgPiece>();
        //start the remove channel async work
        

        thread::spawn(move ||{
            let mut wrapper = wrapper.clone();
            thread::spawn(move || {
                let user = 1;
                let project = 0;
                let flow = 0;
                let workspace = 0;
                let msg = ChatMsg::new(user,project,flow,workspace,"user".to_string(),input.clone().trim().to_string());
                let mut wrapper = wrapper.lock().unwrap();
                let mut llm_wrapper = &mut *wrapper;
                let data = llm_wrapper.chat_message_stream(vec![msg],None, tx);
                let ret = match data {
                    Ok(data)=> {
                        "ok".to_string()
                    },
                    Err(e)=>{
                        "error found".to_string()
                    }
                };
                info!("{}",ret.clone());
            });
        });

        let datas = async_stream::stream! {
            let mut response_data = String::new();
            while let data = rx.recv() {
                match data {
                    Ok(response) => {
                        info!("line:{}",response.token.trim());
                        response_data.push_str(response.token.as_str());
                        yield Ok(response.clone());
                        if response.done {
                            break;
                        }
                    }
                    Err(_) => {
                        yield Err(LlmError(LlmErrorKind::Error))
                    }
                }
            }
        };
        datas
        
    }
}
