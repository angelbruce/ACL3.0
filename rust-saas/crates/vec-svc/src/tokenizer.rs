//! Tokenizer wrapper
//!
//! Use shimmytok for professional GGUF tokenizer
//! Supports SentencePiece, BPE, WordPiece and other tokenizer algorithms

use crate::embedding::EmbeddingError;
use std::path::Path;
use std::sync::Arc;

/// shimmytok Tokenizer wrapper
#[derive(Clone)]
pub struct Tokenizer {
    inner: Arc<shimmytok::Tokenizer>,
    vocab_size: usize,
}

impl Tokenizer {
    /// Load tokenizer from GGUF file
    /// 
    /// shimmytok automatically detects tokenizer info embedded in GGUF files
    /// Supports: SentencePiece (LLaMA/Gemma/Mistral), BPE (GPT/Qwen), WordPiece (BERT)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, EmbeddingError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        tracing::info!("Loading tokenizer from GGUF file: {}", path_str);
        
        let inner = shimmytok::Tokenizer::from_gguf_file(&path_str)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Failed to load tokenizer: {}", e)))?;
        
        let vocab_size = inner.vocab_size();
        
        tracing::info!("Tokenizer loaded successfully. vocab_size={}", vocab_size);

        // Print metadata info
        let bos = inner.bos_token();
        tracing::info!("BOS token ID: {}", bos);

        let eos = inner.eos_token();
        tracing::info!("EOS token ID: {}", eos);
        
        Ok(Self {
            inner: Arc::new(inner),
            vocab_size,
        })
    }

    /// Encode text to token IDs
    /// 
    /// - text: Text to encode
    /// 
    /// Returns token IDs array
    /// 
    /// encode by default does not add BOS token (for embedding lookup scenarios)
    /// If you need BOS/EOS, pass special parameters
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, EmbeddingError> {
        // add_bos = false, because embedding lookup doesn't need BOS token
        // LLM inference scenarios need BOS token
        let tokens = self.inner.encode(text, false)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Encode error: {}", e)))?;
        
        Ok(tokens)
    }

    /// Encode with BOS/EOS tokens
    /// 
    /// For LLM inference scenarios, adds model's required BOS token
    pub fn encode_with_special(&self, text: &str) -> Result<Vec<u32>, EmbeddingError> {
        let tokens = self.inner.encode(text, true)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Encode error: {}", e)))?;
        
        Ok(tokens)
    }

    /// Decode token IDs to text
    /// 
    /// - token_ids: Token IDs array
    /// 
    /// Returns decoded text
    pub fn decode(&self, token_ids: &[u32]) -> Result<String, EmbeddingError> {
        // skip_special_tokens = true, remove BOS/EOS and other special tokens
        let text = self.inner.decode(token_ids, true)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Decode error: {}", e)))?;
        
        Ok(text)
    }

    /// Decode single token (for streaming output)
    /// 
    /// - token_id: Single token ID
    /// 
    /// Returns text piece for this token
    pub fn decode_single(&self, token_id: u32) -> Result<String, EmbeddingError> {
        let piece = self.inner.decode_single(token_id, false)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Decode single error: {}", e)))?;
        
        Ok(piece)
    }

    /// Get vocab size
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    /// Get BOS token ID
    pub fn bos_token(&self) -> u32 {
        self.inner.bos_token()
    }

    /// Get EOS token ID
    pub fn eos_token(&self) -> u32 {
        self.inner.eos_token()
    }

    /// Convert token ID to text piece
    pub fn token_to_piece(&self, token_id: u32) -> Result<String, EmbeddingError> {
        let piece = self.inner.token_to_piece(token_id)
            .map_err(|e| EmbeddingError::Tokenizer(format!("Token to piece error: {}", e)))?;
        
        Ok(piece)
    }

    /// Encode to token text (for debugging)
    pub fn encode_tokens(&self, text: &str) -> Result<Vec<String>, EmbeddingError> {
        let token_ids = self.encode(text)?;
        
        let tokens: Vec<String> = token_ids
            .iter()
            .map(|&id| self.token_to_piece(id).unwrap_or_default())
            .collect();
        
        Ok(tokens)
    }

    /// Get inner shimmytok tokenizer (for advanced operations)
    pub fn inner(&self) -> &shimmytok::Tokenizer {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_api() {
        // Requires GGUF file to run
        // 
        // 
        // let tokenizer = Tokenizer::from_file("models/gemma-4-E4B-it-Q4_0.gguf").unwrap();
        // 
        // // Chinese
        // let tokens = tokenizer.encode("Hello, world!").unwrap();
        // println!("Tokens for 'Hello, world!': {:?}", tokens);
        // 
        // // English
        // let tokens = tokenizer.encode("Hello, world!").unwrap();
        // println!("Tokens for 'Hello, world!': {:?}", tokens);
        // 
        // // Decode
        // let text = tokenizer.decode(&tokens).unwrap();
        // println!("Decoded text: {}", text);
        // 
        // // encode + decode round-trip
        // assert_eq!(text, "Hello, world!");
    }

    #[test]
    fn test_vocab_size() {
        // Requires GGUF file
        // gemma-4-E4B vocab_size should be around 256000
        // 
        // let tokenizer = Tokenizer::from_file("models/gemma-4-E4B-it-Q4_0.gguf").unwrap();
        // let vocab_size = tokenizer.vocab_size();
        // println!("vocab_size: {}", vocab_size);
        // assert!(vocab_size > 250000);
    }
}
