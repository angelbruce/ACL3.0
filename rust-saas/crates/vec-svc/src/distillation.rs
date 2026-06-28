
use crate::model::{NewKnowledgePoint, KnowledgePoint};
use crate::tokenizer::Tokenizer;

/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointType {
    Summary,
    KeyPhrase,
    QnA,
    Fact,
    BestPractice,
}

impl PointType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PointType::Summary => "summary",
            PointType::KeyPhrase => "key_phrase",
            PointType::QnA => "qna",
            PointType::Fact => "fact",
            PointType::BestPractice => "best_practice",
        }
    }
}

/// 
#[derive(Debug, Clone)]
pub struct DistillationResult {
    pub summary: Option<String>,
    pub key_phrases: Vec<String>,
    pub qna_pairs: Vec<QnAPair>,
    pub facts: Vec<String>,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QnAPair {
    pub question: String,
    pub answer: String,
}

use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct DistillationService {
    tokenizer: Tokenizer,
    max_summary_tokens: usize,
    max_key_phrases: usize,
    max_qna_pairs: usize,
}

impl DistillationService {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            max_summary_tokens: 200,
            max_key_phrases: 10,
            max_qna_pairs: 5,
        }
    }

       pub fn distill(&self, content: &str, document_id: i64) -> Result<Vec<NewKnowledgePoint>, String> {
        let result = self.extract_all(content)?;
        let mut points = Vec::new();

        if let Some(summary) = result.summary {
            points.push(NewKnowledgePoint {
                document_id,
                point_type: Some(PointType::Summary.as_str().to_string()),
                point_content: Some(summary),
                confidence: Some(0.8),
                keywords: None,
            });
        }

                for phrase in result.key_phrases {
            points.push(NewKnowledgePoint {
                document_id,
                point_type: Some(PointType::KeyPhrase.as_str().to_string()),
                point_content: Some(phrase),
                confidence: Some(0.7),
                keywords: None,
            });
        }

                for qna in result.qna_pairs {
            let content_json = serde_json::to_string(&qna)
                .map_err(|e| format!("Serialize QnA error: {}", e))?;
            points.push(NewKnowledgePoint {
                document_id,
                point_type: Some(PointType::QnA.as_str().to_string()),
                point_content: Some(content_json),
                confidence: Some(0.65),
                keywords: None,
            });
        }

       
        for fact in result.facts {
            points.push(NewKnowledgePoint {
                document_id,
                point_type: Some(PointType::Fact.as_str().to_string()),
                point_content: Some(fact),
                confidence: Some(0.75),
                keywords: None,
            });
        }

                for bp in result.best_practices {
            points.push(NewKnowledgePoint {
                document_id,
                point_type: Some(PointType::BestPractice.as_str().to_string()),
                point_content: Some(bp),
                confidence: Some(0.7),
                keywords: None,
            });
        }

        Ok(points)
    }

    pub fn extract_all(&self, content: &str) -> Result<DistillationResult, String> {
        let summary = self.extract_summary(content);
        let key_phrases = self.extract_key_phrases(content);
        let qna_pairs = self.extract_qna_pairs(content);
        let facts = self.extract_facts(content);
        let best_practices = self.extract_best_practices(content);

        Ok(DistillationResult {
            summary,
            key_phrases,
            qna_pairs,
            facts,
            best_practices,
        })
    }

    /// TextRank 
    fn extract_summary(&self, content: &str) -> Option<String> {
        let sentences: Vec<&str> = self.split_sentences(content);
        if sentences.is_empty() {
            return None;
        }

        let mut summary_sentences = Vec::new();
        let mut total_tokens = 0;

        for (i, sentence) in sentences.iter().enumerate() {
            let trimmed = sentence.trim();
            if trimmed.is_empty() {
                continue;
            }

            let tokens = self.tokenizer.encode(trimmed).unwrap_or_default();
            if total_tokens + tokens.len() > self.max_summary_tokens {
                break;
            }

            if i == 0 {
                summary_sentences.push(trimmed.to_string());
                total_tokens += tokens.len();
                continue;
            }

            // 
            if self.is_important_sentence(trimmed) && summary_sentences.len() < 5 {
                summary_sentences.push(trimmed.to_string());
                total_tokens += tokens.len();
            }
        }

        if summary_sentences.is_empty() {
            let chars: Vec<char> = content.chars().take(200).collect();
            return Some(chars.into_iter().collect());
        }

        Some(summary_sentences.join(" "))
    }

    /// 
    fn extract_key_phrases(&self, content: &str) -> Vec<String> {
        use std::collections::HashMap;

        let mut word_freq: HashMap<String, (usize, f32)> = HashMap::new(); // (count, score)
        let total_len = content.len();

        // 
        let words: Vec<String> = content
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_lowercase())
            .collect();

        let stop_words = self.get_stop_words();

        for (i, word) in words.iter().enumerate() {
            if stop_words.contains(&word.as_str()) {
                continue;
            }

            // 
            let pos_weight = if i < words.len() / 10 {
                1.5
            } else if i > words.len() * 9 / 10 {
                1.3
            } else {
                1.0
            };

            let entry = word_freq.entry(word.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += pos_weight;
        }

        // * 位置权重
        let mut scored_words: Vec<(String, f32)> = word_freq
            .into_iter()
            .map(|(word, (count, score))| (word, count as f32 * score))
            .collect();
        scored_words.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_words
            .into_iter()
            .take(self.max_key_phrases)
            .map(|(word, _)| word)
            .collect()
    }

    fn extract_qna_pairs(&self, content: &str) -> Vec<QnAPair> {
        let sentences = self.split_sentences(content);
        let mut qna_pairs = Vec::new();

        for sentence in sentences.iter() {
            if qna_pairs.len() >= self.max_qna_pairs {
                break;
            }

            let trimmed = sentence.trim();
            if trimmed.len() < 20 || trimmed.len() > 200 {
                continue;
            }

            // 
            if trimmed.ends_with('.') || trimmed.ends_with('?') || trimmed.ends_with('!') || trimmed.ends_with('？') {
                if let Some(qna) = self.sentence_to_qna(trimmed) {
                    qna_pairs.push(qna);
                }
            }
        }

        qna_pairs
    }

    fn sentence_to_qna(&self, sentence: &str) -> Option<QnAPair> {
        let patterns = [
            ("什么", "是什么"),
            ("哪儿", "在哪儿"),
            ("什么", "有什么"),
            ("包含", "包含什么"),
            ("提供", "提供什么"),
            ("支持", "支持什么"),
        ];

        for (keyword, question_suffix) in patterns.iter() {
            if let Some(pos) = sentence.find(keyword) {
                let subject: String = sentence.chars().take(pos).collect();
                if !subject.is_empty() && subject.chars().count() > 2 {
                    let question = format!("{}{}", subject, question_suffix);
                    return Some(QnAPair {
                        question,
                        answer: sentence.to_string(),
                    });
                }
            }
        }

        if sentence.chars().count() > 10 {
            let question = "这段内容主要讲了什么？".to_string();
            return Some(QnAPair {
                question,
                answer: sentence.to_string(),
            });
        }

        None
    }

 fn extract_facts(&self, content: &str) -> Vec<String> {
        let sentences = self.split_sentences(content);
        let mut facts = Vec::new();

        for sentence in sentences.iter() {
            let trimmed = sentence.trim();
            if trimmed.len() < 15 {
                continue;
            }

            if self.is_fact_sentence(trimmed) && facts.len() < 8 {
                facts.push(trimmed.to_string());
            }
        }

        facts
    }

    fn extract_best_practices(&self, content: &str) -> Vec<String> {
        let sentences = self.split_sentences(content);
        let mut practices = Vec::new();

        let keywords = [
            "应该", "应当", "建议", "最佳实践", "推荐", "需要", "必须",
            "should", "recommend", "must", "need to", "best practice",
        ];

        for sentence in sentences.iter() {
            let trimmed = sentence.trim();
            let lower = trimmed.to_lowercase();

            for keyword in keywords.iter() {
                if lower.contains(keyword) && practices.len() < 5 {
                    practices.push(trimmed.to_string());
                    break;
                }
            }
        }

        practices
    }

    /// 
    fn is_important_sentence(&self, sentence: &str) -> bool {
        let important_keywords = [
            "重要", "关键", "核心", "主要", "首先", "其次", "最后",
            "总之", "因此", "所以", "结论", "结果",
            "important", "key", "main", "first", "second",
            "finally", "therefore", "conclusion", "result",
        ];

        let lower = sentence.to_lowercase();
        important_keywords.iter().any(|k| lower.contains(k))
    }

 fn is_fact_sentence(&self, sentence: &str) -> bool {
        let has_number = sentence.chars().any(|c| c.is_ascii_digit());
        let has_date = sentence.contains("年") || sentence.contains("月") || sentence.contains("日");
        let has_noun_marker = sentence.contains("是") || sentence.contains("称为") || sentence.contains("定义");

        has_number || has_date || has_noun_marker
    }

  fn split_sentences<'a>(&self, content: &'a str) -> Vec<&'a str> {
        let mut sentences = Vec::new();
        let mut start = 0;
        let chars: Vec<char> = content.chars().collect();

        for i in 0..chars.len() {
            let c = chars[i];
            if c == '.' || c == '!' || c == '。' || c == '\r' || c == '\n' || c == '?' {
                if i > start {
                    if let Some(slice) = content.get(start..i + 1) {
                        sentences.push(slice);
                    }
                }
                start = i + 1;
            }
        }

        if start < chars.len() {
            if let Some(slice) = content.get(start..) {
                let trimmed = slice.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
            }
        }

        sentences
    }

    fn get_stop_words(&self) -> Vec<&'static str> {
        vec![
            "一", "的", "着", "没有", "自己",
            "这个", "那个", "什么", "怎么", "为什么",
            "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "of", "in", "to", "for", "on", "with", "at", "by", "from",
            "and", "or", "but", "not", "no", "so", "if", "then", "than",
            "this", "that", "these", "those", "it", "its", "they", "them",
            "we", "us", "our", "you", "your", "he", "she", "his", "her",
            "will", "would", "can", "could", "should", "may", "might",
            "has", "have", "had", "do", "does", "did", "as", "into",
        ]
    }
}
