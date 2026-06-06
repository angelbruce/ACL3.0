use futures::Stream;
use tokio::io::AsyncReadExt;

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Thread;
use std::time::Duration;
use axum::Error;
use base64::DecodeError;
use serde_json;
use shared::errors::{ServiceError, ServiceResult};
use reqwest::Client;
use base64;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};
use bytes::Bytes;



#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct Article {
    pub user_id : i64,
    pub project_id: i64,
    pub article_id: i64,
    pub content: String,
    pub voice_type : String,
    pub voice_seed: i64,
    pub voice_speed: f64,
}

#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct VoiceDataBuffer {
    buffer : String,
}

#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct VoiceResponse {
    pub message: String,
    pub success: bool,
    pub data: Option<VoiceDataBuffer>,
}





impl Article {

    pub fn create_file_stream(file_path: String, chunk_size: usize) -> impl Stream<Item = Result<Bytes, ServiceError>>  {
        async_stream::stream! {
            // 1. 异步打开文件
            let mut file = match tokio::fs::File::open(file_path).await {
                Ok(f) => f,
                Err(e) =>{ 
                    yield Err(ServiceError::InvalidInput(e.to_string()));
                    return;
                }
            };
            
            // 2. 预分配缓冲区
            let mut buffer = vec![0u8; chunk_size];
            
            // 3. 核心循环：不断读取数据
            loop {
                // 异步地读取数据
                let bytes_read = match file.read_buf(&mut buffer).await {   
                    Ok(0) => 0,
                    Ok(n) => n,
                    Err(e) => {
                        yield Err(ServiceError::InvalidInput(e.to_string()));
                        return;
                    }
                };
                
              
                // 4. 将读取到的数据块转换成 Bytes 类型并 Yield (产出)
                // 注意：我们只将实际读取的部分 (0..n) 转换成 Bytes
                if(bytes_read > 0) {
                    let chunk = Bytes::from(buffer[0..bytes_read].to_vec());
                    yield Ok(chunk);
                } else {
                    break;  
                }
            }
            
            // 循环结束后，流自动结束
        }
    }

    // pub async fn get_voice(user_id:i64,project_id: i64,article_id: i64,voice_type: String, voice_seed: i64) -> Result<Vec<u8>, ServiceError> {
    //     println!("10");
    //     let voice_path = Article::get_voice_path(user_id, project_id, article_id, voice_type, voice_seed).await;
    //     println!("11");
    //     println!("{}", voice_path);
    //     let mut file = File::open(&voice_path).map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    //     println!("12");
    //     let mut buf = Vec::new();
    //     println!("13");
    //     file.read_to_end(&mut buf).map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    //     println!("14");
    //     Ok(buf)
    // }

    pub async fn get_voice_path (user_id: i64, project_id: i64, article_id: i64, voice_type: String, voice_seed: i64) -> ServiceResult<String> {
        let root_path = env::var("WORKSPACE_ROOT").unwrap_or_else(|_| "./workspace_storage".to_string());
        let paths = format!("{}/{}/{}/{}_{}_{}.wav", root_path, user_id, project_id, article_id,voice_type,voice_seed);
        Ok(paths)
    }

    pub async fn get_voice_link_path(user_id: i64, project_id: i64, article_id: i64, voice_type: String, voice_seed: i64) -> ServiceResult<String> {
        let paths = format!("/{}/{}/{}_{}_{}.wav",  user_id, project_id, article_id,voice_type,voice_seed);
        Ok(paths)
    }

    pub async fn create_voice(article : Article) -> Result<bool, ServiceError> {
        println!("0");
        let client = Client::new();
        let url = "http://192.168.0.108:8090/voice/make";
        println!("1");
        let resp = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&article)
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(e.to_string()))
            ?;
        
        println!("2");
        let body = resp.text().await.map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        println!("3");
        let resp: VoiceResponse = serde_json::from_str(&body).map_err(|e| ServiceError::BadRequest(e.to_string()))?;
        println!("4");
        if resp.success {
            let data = resp.data.unwrap();
            let buffer = data.buffer;
            let content =  base64::decode(buffer).map_err(|e| ServiceError::BadRequest(e.to_string()))?;    
            let root_path = env::var("WORKSPACE_ROOT").unwrap_or_else(|_| "./workspace_storage".to_string());
            let file_path = format!("{}/{}/{}/{}_{}_{}.wav", root_path, article.user_id, article.project_id, article.article_id,article.voice_type,article.voice_seed);
            let mut file = fs::File::create(file_path).map_err(|e| ServiceError::BadRequest(e.to_string()))?;
            file.write_all(&content).map_err(|e| ServiceError::BadRequest(e.to_string()))?;
            file.flush().map_err(|e| ServiceError::BadRequest(e.to_string()))?;   
            println!("ok");
            return Ok(true)
        } else {
            return Ok(false)
        }

        println!("6");
    }
}
   
   
