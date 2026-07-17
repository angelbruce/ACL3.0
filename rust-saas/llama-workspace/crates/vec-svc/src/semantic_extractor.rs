//! 语义实体抽取模块
//!
//! 基于嵌入向量的语义实体抽取，结合本体定义进行实体识别和分类。

use std::sync::Arc;
use std::collections::HashSet;

use crate::embedding::EmbeddingService;
use crate::ontology::{get_domain_ontology, entity_type_cn_map};
use crate::knowledge_graph::ExtractedEntity;

/// 语义实体抽取器
pub struct SemanticExtractor {
    embedding_service: Arc<EmbeddingService>,
}

impl SemanticExtractor {
    pub fn new(embedding_service: Arc<EmbeddingService>) -> Self {
        Self { embedding_service }
    }

    /// 从文本中抽取实体
    pub async fn extract_entities(&self, content: &str, _project_id: Option<i64>) -> Vec<ExtractedEntity> {
        let ontology = get_domain_ontology();
        let mut entities = Vec::new();
        let mut seen = HashSet::new();

        // 1. 基于规则的初步抽取（保留正则作为基础）
        let rule_entities = self.rule_based_extract(content);
        for entity in rule_entities {
            if !seen.contains(&entity.name) {
                seen.insert(entity.name.clone());
                entities.push(entity);
            }
        }

        // 2. 基于语义的实体分类和补充
        let semantic_entities = self.semantic_classify(content, &ontology).await;
        for entity in semantic_entities {
            if !seen.contains(&entity.name) && entity.name.len() >= 2 {
                seen.insert(entity.name.clone());
                entities.push(entity);
            }
        }

        entities
    }

    /// 基于规则的实体抽取（保留原有逻辑作为基础）
    fn rule_based_extract(&self, content: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        let mut seen = HashSet::new();

        // 英文实体模式
        let en_patterns = [
            (r"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)", "Organization"),
            (r"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?\s+(?:Inc|Ltd|Corp|Company|Group|LLC))", "Organization"),
            (r"([A-Z][a-z]+(?:\s+[A-Z]\.)+)", "Person"),
            (r"(?:https?://)?([a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+(?:/[^\s]*)?)", "URL"),
            (r"(\d{4}-\d{2}-\d{2})", "Date"),
            (r"(\d{1,3}(?:,\d{3})*(?:\.\d+)?\s*(?:元|美元|USD|CNY|欧元|EUR))", "Amount"),
            (r"([A-Za-z0-9_-]+@[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+)", "Email"),
        ];

        for (pattern, entity_type) in en_patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(content) {
                let name = cap.as_str().trim().to_string();
                if !seen.contains(&name) && name.len() >= 2 && name.len() <= 100 {
                    seen.insert(name.clone());
                    entities.push(ExtractedEntity {
                        name,
                        entity_type: entity_type.to_string(),
                        aliases: Vec::new(),
                        confidence: 0.7,
                    });
                }
            }
        }

        // 中文实体模式
        let cn_patterns = [
            (r"([\x{4e00}-\x{9fa5}]{2,6}(?:公司|集团|中心|部门|机构|组织|协会|学会))", "Organization"),
            (r"([\x{4e00}-\x{9fa5}]{2,6}(?:系统|平台|工具|框架|模块|组件|服务|应用))", "Product"),
            (r"([\x{4e00}-\x{9fa5}]{2,6}(?:技术|算法|模型|方法|协议|标准))", "Technology"),
        ];

        for (pattern, entity_type) in cn_patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(content) {
                let name = cap.as_str().trim().to_string();
                if !seen.contains(&name) && name.len() >= 4 && name.len() <= 20 {
                    seen.insert(name.clone());
                    entities.push(ExtractedEntity {
                        name,
                        entity_type: entity_type.to_string(),
                        aliases: Vec::new(),
                        confidence: 0.7,
                    });
                }
            }
        }

        // 中文关键词组合
        let cn_keywords = ["任务", "文档", "决策", "目标", "系统", "平台", "技术", "功能", "模块", "组件", "框架", "工具", "服务", "应用", "模型", "算法"];
        for keyword in cn_keywords {
            let pattern = format!(r"([\x{{4e00}}-\x{{9fa5}}]{{2,4}}{})", regex::escape(keyword));
            for cap in regex::Regex::new(&pattern).unwrap().find_iter(content) {
                let name = cap.as_str().trim().to_string();
                if !seen.contains(&name) && name.len() >= 4 && name.len() <= 10 {
                    seen.insert(name.clone());
                    entities.push(ExtractedEntity {
                        name,
                        entity_type: "Concept".to_string(),
                        aliases: Vec::new(),
                        confidence: 0.6,
                    });
                }
            }
        }

        entities
    }

    /// 基于语义的实体分类
    async fn semantic_classify(&self, content: &str, ontology: &crate::ontology::Ontology) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        // 获取实体类型的描述向量
        let type_descriptions: Vec<String> = ontology.entity_types.iter().map(|t| {
            format!("{}: {}", t.name, t.description)
        }).collect();

        // 计算类型描述的嵌入（批量）
        let type_refs: Vec<&str> = type_descriptions.iter().map(|s| s.as_str()).collect();
        let type_embeddings = match self.embedding_service.embed_batch(&type_refs) {
            Ok(emb) => {
                tracing::info!("Semantic extractor: computed {} type embeddings", emb.len());
                emb
            }
            Err(e) => {
                tracing::warn!("Semantic extractor: failed to embed type descriptions: {}", e);
                return entities;
            }
        };

        // 将内容分句
        let sentences = self.split_sentences(content);
        tracing::info!("Semantic extractor: split content into {} sentences", sentences.len());

        // 对每个句子计算与实体类型的相似度
        for sentence in &sentences {
            if sentence.len() < 5 || sentence.len() > 200 {
                continue;
            }

            let sentence_embedding = match self.embedding_service.embed(sentence) {
                Ok(emb) => emb,
                Err(e) => {
                    tracing::debug!("Semantic extractor: failed to embed sentence: {}", e);
                    continue;
                }
            };

            // 计算与每个实体类型的相似度
            let mut best_match: Option<(String, f32)> = None;

            for (i, type_emb) in type_embeddings.iter().enumerate() {
                let similarity = cosine_similarity(&sentence_embedding, type_emb);
                if similarity > 0.15 {
                    let type_name = ontology.entity_types[i].name.clone();
                    if let Some((_, best_score)) = &best_match {
                        if similarity > *best_score {
                            best_match = Some((type_name, similarity));
                        }
                    } else {
                        best_match = Some((type_name, similarity));
                    }
                }
            }

            // 如果找到匹配的类型，尝试从句子中提取实体
            if let Some((type_name, score)) = best_match {
                tracing::debug!("Semantic extractor: sentence matched type={} score={:.3} sentence={:.60}", type_name, score, sentence);
                if let Some(entity_name) = self.extract_entity_from_sentence(sentence, &type_name) {
                    entities.push(ExtractedEntity {
                        name: entity_name,
                        entity_type: type_name,
                        aliases: Vec::new(),
                        confidence: score as f64,
                    });
                }
            }
        }

        entities
    }

    /// 从句子中提取实体名称
    fn extract_entity_from_sentence(&self, sentence: &str, entity_type: &str) -> Option<String> {
        // 根据实体类型使用不同的提取策略
        match entity_type {
            "Task" => {
                // 任务实体：包含"任务"的短语
                if let Some(pos) = sentence.find("任务") {
                    let start = sentence[..pos].rfind(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| p + 1).unwrap_or(0);
                    let end = sentence[pos..].find(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| pos + p).unwrap_or(sentence.len());
                    let name = sentence[start..end].trim().to_string();
                    if name.len() >= 4 && name.len() <= 20 {
                        return Some(name);
                    }
                }
                None
            }
            "Document" => {
                // 文档实体：包含"文档"、"文件"的短语
                for keyword in &["文档", "文件", "报告"] {
                    if let Some(pos) = sentence.find(keyword) {
                        let start = sentence[..pos].rfind(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| p + 1).unwrap_or(0);
                        let end = sentence[pos..].find(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| pos + p).unwrap_or(sentence.len());
                        let name = sentence[start..end].trim().to_string();
                        if name.len() >= 4 && name.len() <= 20 {
                            return Some(name);
                        }
                    }
                }
                None
            }
            "Decision" => {
                // 决策实体：包含"决策"、"决定"的短语
                for keyword in &["决策", "决定", "结论"] {
                    if let Some(pos) = sentence.find(keyword) {
                        let start = sentence[..pos].rfind(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| p + 1).unwrap_or(0);
                        let end = sentence[pos..].find(|c: char| c == '，' || c == '。' || c == '、' || c == ' ').map(|p| pos + p).unwrap_or(sentence.len());
                        let name = sentence[start..end].trim().to_string();
                        if name.len() >= 4 && name.len() <= 20 {
                            return Some(name);
                        }
                    }
                }
                None
            }
            _ => {
                // 其他类型：提取句子中的关键名词短语
                self.extract_noun_phrase(sentence)
            }
        }
    }

    /// 提取名词短语
    fn extract_noun_phrase(&self, sentence: &str) -> Option<String> {
        // 简单策略：提取 2-6 个中文字符的连续序列
        let re = regex::Regex::new(r"[\x{4e00}-\x{9fa5}]{2,6}").unwrap();
        for cap in re.find_iter(sentence) {
            let word = cap.as_str();
            // 过滤掉常见停用词
            if !["这是", "一个", "这个", "那个", "我们", "他们", "可以", "需要", "应该"].contains(&word) {
                return Some(word.to_string());
            }
        }
        None
    }

    /// 将文本分割成句子
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);
            if matches!(ch, '。' | '！' | '？' | '；' | '\n') {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() && trimmed.len() >= 5 {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }

        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() && trimmed.len() >= 5 {
            sentences.push(trimmed);
        }

        sentences
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
