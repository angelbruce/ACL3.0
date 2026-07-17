//! GGUF 文件解析相关类型定义
//! 
//! GGUF 文件中读取模型权�?
use std::collections::HashMap;

/// GGUF Metadata Value
#[derive(Debug, Clone)]
pub enum Value {
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
    Bool(bool),
    String(String),
    Array(u32, Vec<Value>),
    Uint64(u64),
    Int64(i64),
    Float64(f64),
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Uint64(n) => Some(*n),
            Value::Uint32(n) => Some(*n as u64),
            Value::Uint16(n) => Some(*n as u64),
            Value::Uint8(n) => Some(*n as u64),
            _ => None,
        }
    }
}

/// GGUF Tensor Info
#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub n_dims: u32,
    pub dims: Vec<u64>,
    pub tensor_type: u32,
    pub offset: u64,
}

/// GGUF Tensor Type
#[derive(Debug, Clone, Copy)]
pub enum TensorType {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    Q5_0 = 6,
    Q5_1 = 7,
    Q8_0 = 8,
    Q8_1 = 9,
}

/// GGUF Value Type
#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Uint32 = 4,
    Int32 = 5,
    Float32 = 6,
    Bool = 7,
    String = 8,
    Array = 9,
    Uint64 = 10,
    Int64 = 11,
    Float64 = 12,
}

/// GGUF 文件格式常量
pub const GGUF_MAGIC: u32 = 0x46554747; // "GGUF"
