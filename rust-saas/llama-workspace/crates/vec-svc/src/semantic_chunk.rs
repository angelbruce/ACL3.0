use std::sync::Arc;
use crate::tokenizer::Tokenizer;

pub struct SemanticChunker {
    tokenizer: Arc<Tokenizer>,
    max_chunk_tokens: usize,
    min_chunk_tokens: usize,
    overlap_tokens: usize,
}

impl SemanticChunker {
    pub fn new(tokenizer: Arc<Tokenizer>) -> Self {
        Self {
            tokenizer,
            max_chunk_tokens: 512,
            min_chunk_tokens: 100,
            overlap_tokens: 50,
        }
    }

    pub fn with_config(
        tokenizer: Arc<Tokenizer>,
        max_chunk_tokens: usize,
        min_chunk_tokens: usize,
        overlap_tokens: usize,
    ) -> Self {
        Self {
            tokenizer,
            max_chunk_tokens,
            min_chunk_tokens,
            overlap_tokens,
        }
    }

    pub fn chunk_text(&self, text: &str) -> Vec<ChunkResult> {
        let paragraphs = self.split_paragraphs(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_tokens = 0;

        for paragraph in paragraphs {
            let para_tokens = self.count_tokens(&paragraph);

            if para_tokens > self.max_chunk_tokens {
                if !current_chunk.is_empty() {
                    chunks.push(ChunkResult {
                        text: current_chunk.trim().to_string(),
                        token_count: current_tokens,
                        chunk_type: ChunkType::Mixed,
                    });
                    current_chunk = String::new();
                    current_tokens = 0;
                }

                let sub_chunks = self.chunk_large_paragraph(&paragraph);
                chunks.extend(sub_chunks);
                continue;
            }

            if current_tokens + para_tokens > self.max_chunk_tokens && current_tokens >= self.min_chunk_tokens {
                chunks.push(ChunkResult {
                    text: current_chunk.trim().to_string(),
                    token_count: current_tokens,
                    chunk_type: ChunkType::Mixed,
                });

                if self.overlap_tokens > 0 {
                    let overlap_text = self.get_overlap(&current_chunk, self.overlap_tokens);
                    current_chunk = overlap_text;
                    current_tokens = self.count_tokens(&current_chunk);
                } else {
                    current_chunk = String::new();
                    current_tokens = 0;
                }
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(&paragraph);
            current_tokens += para_tokens;
        }

        if !current_chunk.is_empty() {
            chunks.push(ChunkResult {
                text: current_chunk.trim().to_string(),
                token_count: current_tokens,
                chunk_type: ChunkType::Mixed,
            });
        }

        self.classify_chunks(chunks)
    }

    fn split_paragraphs(&self, text: &str) -> Vec<String> {
        let mut paragraphs = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !current.is_empty() {
                    paragraphs.push(current.trim().to_string());
                    current = String::new();
                }
            } else if self.is_heading(trimmed) {
                if !current.is_empty() {
                    paragraphs.push(current.trim().to_string());
                    current = String::new();
                }
                paragraphs.push(trimmed.to_string());
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(trimmed);
            }
        }

        if !current.is_empty() {
            paragraphs.push(current.trim().to_string());
        }

        paragraphs
    }

    fn is_heading(&self, line: &str) -> bool {
        if line.starts_with('#') {
            return true;
        }

        if line.ends_with(':') && line.len() < 50 {
            return true;
        }

        if line.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace() || c.is_ascii_digit())
            && line.len() < 60
            && line.len() > 2
        {
            return true;
        }

        false
    }

    fn chunk_large_paragraph(&self, paragraph: &str) -> Vec<ChunkResult> {
        let mut chunks = Vec::new();
        let sentences = self.split_sentences(paragraph);
        let mut current = String::new();
        let mut current_tokens = 0;

        for sentence in sentences {
            let sent_tokens = self.count_tokens(&sentence);

            if sent_tokens > self.max_chunk_tokens {
                if !current.is_empty() {
                    chunks.push(ChunkResult {
                        text: current.trim().to_string(),
                        token_count: current_tokens,
                        chunk_type: ChunkType::Paragraph,
                    });
                    current = String::new();
                    current_tokens = 0;
                }

                let sub_chunks = self.chunk_by_tokens(&sentence);
                chunks.extend(sub_chunks);
                continue;
            }

            if current_tokens + sent_tokens > self.max_chunk_tokens && current_tokens >= self.min_chunk_tokens {
                chunks.push(ChunkResult {
                    text: current.trim().to_string(),
                    token_count: current_tokens,
                    chunk_type: ChunkType::Paragraph,
                });

                if self.overlap_tokens > 0 {
                    let overlap = self.get_overlap(&current, self.overlap_tokens);
                    current = overlap;
                    current_tokens = self.count_tokens(&current);
                } else {
                    current = String::new();
                    current_tokens = 0;
                }
            }

            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&sentence);
            current_tokens += sent_tokens;
        }

        if !current.is_empty() {
            chunks.push(ChunkResult {
                text: current.trim().to_string(),
                token_count: current_tokens,
                chunk_type: ChunkType::Paragraph,
            });
        }

        chunks
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            current.push(c);

            if c == '.' || c == '!' || c == '?' || c == '。' || c == '！' || c == '？' {
                let mut end_idx = i + 1;
                while end_idx < chars.len() && (chars[end_idx] == '"' || chars[end_idx] == '\'' || chars[end_idx] == ')' || chars[end_idx] == ';') {
                    current.push(chars[end_idx]);
                    end_idx += 1;
                }

                if end_idx < chars.len() && chars[end_idx].is_whitespace() {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        sentences.push(trimmed);
                    }
                    current = String::new();
                    i = end_idx;
                    continue;
                }
                i = end_idx.saturating_sub(1);
            }
            i += 1;
        }

        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }

        sentences
    }

    fn chunk_by_tokens(&self, text: &str) -> Vec<ChunkResult> {
        let tokens = self.tokenize(text);
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < tokens.len() {
            let end = std::cmp::min(start + self.max_chunk_tokens, tokens.len());
            let chunk_tokens = &tokens[start..end];
            let chunk_text = self.detokenize(chunk_tokens);

            chunks.push(ChunkResult {
                text: chunk_text.trim().to_string(),
                token_count: chunk_tokens.len(),
                chunk_type: ChunkType::Token,
            });

            if end >= tokens.len() {
                break;
            }

            start = if self.overlap_tokens > 0 {
                end.saturating_sub(self.overlap_tokens)
            } else {
                end
            };
        }

        chunks
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.encode(text).unwrap_or_default().len()
    }

    fn tokenize(&self, text: &str) -> Vec<u32> {
        self.tokenizer.encode(text).unwrap_or_default()
    }

    fn detokenize(&self, tokens: &[u32]) -> String {
        self.tokenizer.decode(tokens).unwrap_or_default()
    }

    fn get_overlap(&self, text: &str, overlap_tokens: usize) -> String {
        let tokens = self.tokenize(text);
        if tokens.len() <= overlap_tokens {
            return text.to_string();
        }
        let start = tokens.len() - overlap_tokens;
        self.detokenize(&tokens[start..])
    }

    fn classify_chunks(&self, chunks: Vec<ChunkResult>) -> Vec<ChunkResult> {
        chunks
            .into_iter()
            .map(|chunk| {
                let chunk_type = if self.is_heading_chunk(&chunk.text) {
                    ChunkType::Heading
                } else if self.is_list_chunk(&chunk.text) {
                    ChunkType::List
                } else if self.is_code_chunk(&chunk.text) {
                    ChunkType::Code
                } else {
                    chunk.chunk_type
                };

                ChunkResult {
                    chunk_type,
                    ..chunk
                }
            })
            .collect()
    }

    fn is_heading_chunk(&self, text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() == 1 && self.is_heading(lines[0]) {
            return true;
        }
        false
    }

    fn is_list_chunk(&self, text: &str) -> bool {
        let lines: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if lines.len() < 2 {
            return false;
        }

        let list_count = lines
            .iter()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.starts_with("- ")
                    || trimmed.starts_with("* ")
                    || trimmed.starts_with("。")
                    || trimmed.starts_with("1. ")
                    || trimmed.starts_with("一")
                    || trimmed.starts_with("二")
            })
            .count();

        list_count as f64 / lines.len() as f64 > 0.5
    }

    fn is_code_chunk(&self, text: &str) -> bool {
        if text.contains("```") {
            return true;
        }

        let code_patterns = ["fn ", "function ", "def ", "class ", "if (", "for (", "while ("];
        let code_count = code_patterns
            .iter()
            .filter(|p| text.contains(**p))
            .count();

        code_count >= 2
    }
}

#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub text: String,
    pub token_count: usize,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Heading,
    Paragraph,
    List,
    Code,
    Token,
    Mixed,
}

impl Default for SemanticChunker {
    fn default() -> Self {
        panic!("SemanticChunker requires a tokenizer");
    }
}
