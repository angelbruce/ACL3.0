//! GGUF Embedding Service
//! 
//! Extract embedding vectors from GGUF model files
//! Tokenize text and lookup vectors by token IDs

use crate::loader;
use crate::tokenizer;
use loader::Value;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("GGUF parse error: {0}")]
    Parse(String),
    
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
    
    #[error("Model not loaded")]
    ModelNotLoaded,
    
    #[error("Unsupported tensor type: {0}")]
    UnsupportedTensor(String),
}

impl From<EmbeddingError> for String {
    fn from(err: EmbeddingError) -> Self {
        err.to_string()
    }
}

/// Embedding configuration
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Model file path
    pub model_path: String,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Max sequence length
    pub max_sequence_length: usize,
}

/// Embedding 服务
#[derive(Clone)]
pub struct EmbeddingService {
    config: EmbeddingConfig,
    tokenizer: tokenizer::Tokenizer,
    /// token embeddings 权重，shape: [vocab_size, embedding_dim]
    embedding_weights: Vec<f32>,
    /// vocab size
    vocab_size: usize,
}

impl EmbeddingService {
    /// GGUF 文件加载模型
    pub fn load<P: AsRef<Path>>(model_path: P, embedding_dim: usize) -> Result<Self, EmbeddingError> {
        let path = model_path.as_ref();
        
        // 1. 加载 tokenizer
        let tokenizer = tokenizer::Tokenizer::from_file(path)?;
        
        // 2. 加载 GGUF 文件，提取 embedding 权重
        let (embedding_weights, vocab_size) = Self::load_embedding_weights(path, embedding_dim)?;
        
        let config = EmbeddingConfig {
            model_path: path.to_string_lossy().to_string(),
            embedding_dim,
            max_sequence_length: 8192,
        };
        
        Ok(Self {
            config,
            tokenizer,
            embedding_weights,
            vocab_size,
        })
    }
    
    /// 
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // 1. 分词
        let token_ids = self.tokenizer.encode(text)?;
        
        // 2. 计算平均向量（CLS 策略，Mean Pooling）        
        let mut embedding = vec![0.0f32; self.config.embedding_dim];
        
        for token_id in &token_ids {
            let start = (*token_id as usize) * self.config.embedding_dim;
            let end = start + self.config.embedding_dim;
            
            if end <= self.embedding_weights.len() {
                for i in 0..self.config.embedding_dim {
                    embedding[i] += self.embedding_weights[start + i];
                }
            }
        }
        
        // 
        if !token_ids.is_empty() {
            let scale = 1.0 / (token_ids.len() as f32);
            for v in &mut embedding {
                *v *= scale;
            }
        }
        
        Ok(embedding)
    }
    
    /// Embed multiple texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    
    /// Get tokenizer
    pub fn tokenizer(&self) -> &tokenizer::Tokenizer {
        &self.tokenizer
    }
    
    /// Get config
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }
    
    // ============ Internal methods ============
    
    /// Load embedding weights from GGUF file
    fn load_embedding_weights(path: &Path, embedding_dim: usize) -> Result<(Vec<f32>, usize), EmbeddingError> {
        let file = File::open(path)?;
        let mut file = std::io::BufReader::with_capacity(8 * 1024 * 1024, file);
        
        // Read GGUF header
        let magic = read_u32(&mut file)?;
        if magic != 0x46554747 { // "GGUF"
            return Err(EmbeddingError::Parse("Invalid GGUF magic number".to_string()));
        }
        
        let version = read_u32(&mut file)?;
        let n_tensors = read_u64(&mut file)?;
        let n_kv = read_u64(&mut file)?;
        
        tracing::info!("GGUF: version={}, n_tensors={}, n_kv={}", version, n_tensors, n_kv);
        
        // Read metadata
        let metadata = Self::read_metadata(&mut file, n_kv)?;
        
        // 
        let architecture = metadata.get("general.architecture")
            .and_then(|v| v.as_string())
            .ok_or_else(|| EmbeddingError::Parse("Missing architecture".to_string()))?;
        
        let arch_prefix = architecture;
        
        // Get embedding dimension from metadata
        let actual_embedding_dim = metadata
            .get(&format!("{}.embedding_length", arch_prefix))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(embedding_dim);
        
        tracing::info!("Model embedding_dim: {}", actual_embedding_dim);
        
        // Get vocab size
        let vocab_size = metadata
            .get(&format!("{}.vocab_size", arch_prefix))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);
        
        tracing::info!("Vocab size: {}", vocab_size);
        
        // Read tensor info
        let tensor_infos = Self::read_tensor_infos(&mut file, n_tensors)?;
        
        // Get tensor data start position
        let tensor_data_start = file.stream_position()?;
        
        // Find token_embeddings weights
        let mut embedding_weights: Option<Vec<f32>> = None;
        
        for info in &tensor_infos {
            // Find token embedding tensor
            let name_lower = info.name.to_lowercase();
            let is_candidate = name_lower.contains("token_embd") || name_lower.contains("embed_tokens");
            if is_candidate {
                tracing::info!("Embedding candidate tensor: {} (dims: {:?}, type: {:?})",
                    info.name, info.dims, info.tensor_type);
            }
            if (name_lower == "token_embd.weight" || name_lower == "embed_tokens.weight") && !name_lower.contains("per_layer") {
                tracing::info!("Found embedding tensor: {} (dims: {:?}, type: {:?})",
                    info.name, info.dims, info.tensor_type);
                let weights = Self::load_tensor_data(&mut file, info, tensor_data_start)?;
                embedding_weights = Some(weights);
                break;
            }
        }
        
        let weights = embedding_weights.ok_or_else(|| {
            EmbeddingError::Parse("Token embedding tensor not found".to_string())
        })?;
        
        Ok((weights, vocab_size))
    }
    
    /// Read metadata
    fn read_metadata<R: Read>(reader: &mut R, n_kv: u64) -> Result<HashMap<String, loader::Value>, EmbeddingError> {
        let mut metadata = HashMap::new();
        
        for _ in 0..n_kv {
            //  key
            let key_len = read_u64(reader)? as usize;
            let mut key_bytes = vec![0u8; key_len];
            reader.read_exact(&mut key_bytes)?;
            let key = String::from_utf8(key_bytes)
                .map_err(|e| EmbeddingError::Parse(e.to_string()))?;
            
            //  value type
            let value_type = read_u32(reader)?;
            
            //  value
            let value = Self::read_value(reader, value_type)?;
            metadata.insert(key, value);
        }
        
        Ok(metadata)
    }
    
    ///  value
    fn read_value<R: Read>(reader: &mut R, value_type: u32) -> Result<loader::Value, EmbeddingError> {
        match value_type {
            0 => Ok(loader::Value::Uint8(read_u8(reader)?)),
            1 => Ok(loader::Value::Int8(read_u8(reader)? as i8)),
            2 => Ok(loader::Value::Uint16(read_u16(reader)?)),
            3 => Ok(loader::Value::Int16(read_u16(reader)? as i16)),
            4 => Ok(loader::Value::Uint32(read_u32(reader)?)),
            5 => Ok(loader::Value::Int32(read_u32(reader)? as i32)),
            6 => Ok(loader::Value::Float32(read_f32(reader)?)),
            7 => Ok(loader::Value::Bool(read_u8(reader)? != 0)),
            8 => {
                let len = read_u64(reader)? as usize;
                let mut bytes = vec![0u8; len];
                reader.read_exact(&mut bytes)?;
                let s = String::from_utf8(bytes)
                    .map_err(|e| EmbeddingError::Parse(e.to_string()))?;
                Ok(loader::Value::String(s))
            }
            9 => {
                // Array
                let element_type = read_u32(reader)?;
                let count = read_u64(reader)? as usize;
                let mut elements = Vec::with_capacity(count);
                for _ in 0..count {
                    elements.push(Self::read_value(reader, element_type)?);
                }
                Ok(loader::Value::Array(element_type, elements))
            }
            10 => Ok(loader::Value::Uint64(read_u64(reader)?)),
            11 => Ok(loader::Value::Int64(read_u64(reader)? as i64)),
            12 => Ok(loader::Value::Float64(read_f64(reader)?)),
            _ => Err(EmbeddingError::Parse(format!("Unknown value type: {}", value_type))),
        }
    }
    
    /// Read tensor info list
    fn read_tensor_infos<R: Read + Seek>(reader: &mut R, n_tensors: u64) -> Result<Vec<loader::TensorInfo>, EmbeddingError> {
        let mut infos = Vec::with_capacity(n_tensors as usize);
        
        for _ in 0..n_tensors {
            // name
            let name_len = read_u64(reader)? as usize;
            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes)
                .map_err(|e| EmbeddingError::Parse(e.to_string()))?;
            
            // n_dims
            let n_dims = read_u32(reader)?;
            
            // dims
            let mut dims = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                dims.push(read_u64(reader)?);
            }
            
            // tensor_type
            let tensor_type = read_u32(reader)?;
            
            // offset
            let offset = read_u64(reader)?;
            
            infos.push(loader::TensorInfo {
                name,
                n_dims,
                dims,
                tensor_type,
                offset,
            });
        }
        
        Ok(infos)
    }
    
    /// Load tensor data
    fn load_tensor_data<R: Read + Seek>(
        reader: &mut R,
        info: &loader::TensorInfo,
        tensor_data_start: u64,
    ) -> Result<Vec<f32>, EmbeddingError> {
        let offset = tensor_data_start + info.offset;
        reader.seek(SeekFrom::Start(offset))?;
        
        let element_count: u64 = info.dims.iter().product();

        // Convert to f32
        match info.tensor_type {
            0 => {
                // F32
                let byte_size = (element_count * 4) as usize;
                let mut data = vec![0u8; byte_size];
                reader.read_exact(&mut data)?;
                let mut result = Vec::with_capacity(element_count as usize);
                for chunk in data.chunks_exact(4) {
                    let bytes: [u8; 4] = chunk.try_into().unwrap();
                    result.push(f32::from_le_bytes(bytes));
                }
                Ok(result)
            }
            1 => {
                // F16 -> F32
                let byte_size = (element_count * 2) as usize;
                let mut data = vec![0u8; byte_size];
                reader.read_exact(&mut data)?;
                let mut result = Vec::with_capacity(element_count as usize);
                for chunk in data.chunks_exact(2) {
                    let bytes: [u8; 2] = chunk.try_into().unwrap();
                    let f16_bits = u16::from_le_bytes(bytes);
                    result.push(f16_to_f32(f16_bits));
                }
                Ok(result)
            }
            2 => {
                // Q4_0 quantized: blocks of 32 weights, each block = 2 bytes scale (f16) + 16 bytes nibbles
                const BLOCK_SIZE: u64 = 32;
                let blocks = element_count / BLOCK_SIZE;
                let bytes_per_block = 2 + (BLOCK_SIZE / 2);
                let byte_size = (blocks * bytes_per_block) as usize;
                let mut data = vec![0u8; byte_size];
                reader.read_exact(&mut data)?;
                let mut result = Vec::with_capacity(element_count as usize);
                for block in 0..blocks {
                    let offset = (block * bytes_per_block) as usize;
                    let scale_bytes: [u8; 2] = data[offset..offset + 2].try_into().unwrap();
                    let scale = f16_to_f32(u16::from_le_bytes(scale_bytes));
                    let nibbles = &data[offset + 2..offset + bytes_per_block as usize];
                    for byte in nibbles {
                        let low = byte & 0x0F;
                        let high = byte >> 4;
                        result.push(scale * ((low as f32) - 8.0));
                        result.push(scale * ((high as f32) - 8.0));
                    }
                }
                Ok(result)
            }
            12 => {
                // Q4_K quantized: super-blocks of 256 weights
                // block layout: 2 bytes d (f16) + 2 bytes dmin (f16) + 12 bytes scales + 128 bytes qs
                const BLOCK_SIZE: u64 = 256;
                let blocks = element_count / BLOCK_SIZE;
                const BYTES_PER_BLOCK: usize = 2 + 2 + 12 + 128;
                let byte_size = (blocks as usize) * BYTES_PER_BLOCK;
                let mut data = vec![0u8; byte_size];
                reader.read_exact(&mut data)?;
                let mut result = Vec::with_capacity(element_count as usize);
                for block in 0..blocks as usize {
                    let offset = block * BYTES_PER_BLOCK;
                    let d_bits = u16::from_le_bytes([data[offset], data[offset + 1]]);
                    let dmin_bits = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
                    let dall = f16_to_f32(d_bits);
                    let dmin = f16_to_f32(dmin_bits);
                    let scales = &data[offset + 4..offset + 16];
                    let qs = &data[offset + 16..offset + 144];
                    for j in 0..8 {
                        let (sc, m) = get_scale_min_k4(j, scales);
                        let q = &qs[j * 16..(j + 1) * 16];
                        for l in 0..16 {
                            let low = q[l] & 0x0F;
                            let high = q[l] >> 4;
                            result.push(dall * (sc as f32 * low as f32 - m as f32));
                            result.push(dall * (sc as f32 * high as f32 - m as f32));
                        }
                    }
                }
                Ok(result)
            }
            _ => Err(EmbeddingError::UnsupportedTensor(format!("{}", info.tensor_type))),
        }
    }
}

// ============ Helper functions ============

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, EmbeddingError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(buf[0])
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16, EmbeddingError> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, EmbeddingError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64, EmbeddingError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_f32<R: Read>(reader: &mut R) -> Result<f32, EmbeddingError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f64<R: Read>(reader: &mut R) -> Result<f64, EmbeddingError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf).map_err(EmbeddingError::from)?;
    Ok(f64::from_le_bytes(buf))
}

/// Extract 6-bit scale and min for Q4_K inner group `j` from 12-byte scale data
fn get_scale_min_k4(j: usize, q: &[u8]) -> (u8, u8) {
    if j < 4 {
        let d = q[j] & 0x3F;
        let m = q[j + 4] & 0x3F;
        (d, m)
    } else {
        let d = (q[j + 4] & 0x0F) | ((q[j - 4] >> 6) << 4);
        let m = (q[j + 4] >> 4) | ((q[j] >> 6) << 4);
        (d, m)
    }
}

/// F16 -> F32 转换
fn f16_to_f32(f16_bits: u16) -> f32 {
    let sign = (f16_bits >> 15) & 0x1;
    let exponent = (f16_bits >> 10) & 0x1f;
    let mantissa = f16_bits & 0x3ff;
    
    if exponent == 0 {
        if mantissa == 0 {
            return if sign == 1 { -0.0 } else { 0.0 };
        } else {
            let mut value = (mantissa as f32) / 1024.0;
            value *= 2f32.powi(-14);
            return if sign == 1 { -value } else { value };
        }
    } else if exponent == 31 {
        if mantissa == 0 {
            return if sign == 1 { f32::NEG_INFINITY } else { f32::INFINITY };
        } else {
            return f32::NAN;
        }
    }
    
    let f32_exponent = (exponent as i32) - 15 + 127;
    let f32_mantissa = (mantissa as u32) << 13;
    
    let f32_bits = ((sign as u32) << 31) | ((f32_exponent as u32) << 23) | f32_mantissa;
    f32::from_bits(f32_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_f16_to_f32() {
        //  f16 -> f32 转换
        assert_eq!(f16_to_f32(0x3C00), 1.0); // 1.0
        assert_eq!(f16_to_f32(0x4000), 2.0); // 2.0
        assert_eq!(f16_to_f32(0x0000), 0.0); // 0.0
    }
}
