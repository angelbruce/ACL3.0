//! 知识图谱服务模块
//!
//! 提供知识图谱服务，用于提取文档中的实体和关系。
//! 基于领域本体定义，结合规则匹配和语义嵌入进行实体抽取。

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, TextExpressionMethods};

use crate::model::{KnowledgeEntity, NewKnowledgeEntity, KnowledgeRelation, NewKnowledgeRelation};
use crate::rdb_repository::{DbPool, RepositoryError};
use crate::schema;
use crate::embedding::EmbeddingService;
use crate::semantic_extractor::SemanticExtractor;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityExtractionResult {
    pub entities: Vec<ExtractedEntity>,
    pub relations: Vec<ExtractedRelation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
    pub aliases: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedRelation {
    pub from_entity: String,
    pub to_entity: String,
    pub relation_type: String,
    pub evidence: String,
    pub confidence: f64,
}

#[derive(Clone)]
pub struct KnowledgeGraphService {
    db_pool: Arc<DbPool>,
    semantic_extractor: Option<Arc<SemanticExtractor>>,
}

impl KnowledgeGraphService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool, semantic_extractor: None }
    }

    pub fn with_extractor(db_pool: Arc<DbPool>, embedding_service: Arc<EmbeddingService>) -> Self {
        let extractor = Arc::new(SemanticExtractor::new(embedding_service));
        Self { db_pool, semantic_extractor: Some(extractor) }
    }

    pub async fn extract_from_document(&self, document_id: i64, content: &str, project_id: Option<i64>) -> Result<EntityExtractionResult, String> {
        // 优先使用语义抽取器，否则回退到规则抽取
        let entities = if let Some(ref extractor) = self.semantic_extractor {
            extractor.extract_entities(content, project_id).await
        } else {
            self.rule_based_extract(content, project_id)
        };
        let relations = self.extract_relations(content, &entities);

        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        for entity in &entities {
            let existing = if let Some(pid) = project_id {
                crate::schema::knowledge_entities::table
                    .filter(crate::schema::knowledge_entities::name.eq(&entity.name))
                    .filter(crate::schema::knowledge_entities::project_id.eq(pid))
                    .first::<KnowledgeEntity>(&mut conn)
                    .optional()
                    .map_err(|e| format!("Failed to check entity: {}", e))?
            } else {
                crate::schema::knowledge_entities::table
                    .filter(crate::schema::knowledge_entities::name.eq(&entity.name))
                    .filter(crate::schema::knowledge_entities::project_id.is_null())
                    .first::<KnowledgeEntity>(&mut conn)
                    .optional()
                    .map_err(|e| format!("Failed to check entity: {}", e))?
            };

            if existing.is_none() {
                let new_entity = NewKnowledgeEntity {
                    project_id,
                    name: Some(entity.name.clone()),
                    entity_type: Some(entity.entity_type.clone()),
                    description: None,
                    aliases: Some(serde_json::to_value(entity.aliases.clone()).map_err(|e| format!("Serialize error: {}", e))?),
                    confidence: Some(entity.confidence),
                    source_document_id: Some(document_id),
                };

                diesel::insert_into(crate::schema::knowledge_entities::table)
                    .values(&new_entity)
                    .execute(&mut conn)
                    .map_err(|e| format!("Failed to insert entity: {}", e))?;
            }
        }

        for relation in &relations {
            let from_entity = crate::schema::knowledge_entities::table
                .filter(crate::schema::knowledge_entities::name.eq(&relation.from_entity))
                .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
                .first::<KnowledgeEntity>(&mut conn)
                .optional()
                .map_err(|e| format!("Failed to get from entity: {}", e))?;

            let to_entity = crate::schema::knowledge_entities::table
                .filter(crate::schema::knowledge_entities::name.eq(&relation.to_entity))
                .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
                .first::<KnowledgeEntity>(&mut conn)
                .optional()
                .map_err(|e| format!("Failed to get to entity: {}", e))?;

            if let (Some(from), Some(to)) = (from_entity, to_entity) {
                let existing = crate::schema::knowledge_relations::table
                    .filter(crate::schema::knowledge_relations::source_entity_id.eq(from.id))
                    .filter(crate::schema::knowledge_relations::target_entity_id.eq(to.id))
                    .filter(crate::schema::knowledge_relations::relation_type.eq(&relation.relation_type))
                    .first::<KnowledgeRelation>(&mut conn)
                    .optional()
                    .map_err(|e| format!("Failed to check relation: {}", e))?;

                if existing.is_none() {
                    let new_relation = NewKnowledgeRelation {
                        project_id,
                        source_entity_id: from.id,
                        target_entity_id: to.id,
                        relation_type: Some(relation.relation_type.clone()),
                        relation_strength: Some(relation.confidence),
                        evidence_text: Some(relation.evidence.clone()),
                        source_document_id: Some(document_id),
                        confidence: Some(relation.confidence),
                    };

                    diesel::insert_into(crate::schema::knowledge_relations::table)
                        .values(&new_relation)
                        .execute(&mut conn)
                        .map_err(|e| format!("Failed to insert relation: {}", e))?;
                }
            }
        }

        Ok(EntityExtractionResult { entities, relations })
    }

    /// 规则抽取（回退方案）
    fn rule_based_extract(&self, content: &str, _project_id: Option<i64>) -> Vec<ExtractedEntity> {
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

    fn extract_relations(&self, content: &str, entities: &[ExtractedEntity]) -> Vec<ExtractedRelation> {
        let mut relations = Vec::new();
        let mut seen = HashSet::new();

        let relation_patterns = [
            (r"(\S+)\s+(提供|提供了|提供了)\s+(\S+)", "provides"),
            (r"(\S+)\s+(支持|支持)\s+(\S+)", "supports"),
            (r"(\S+)\s+(使用|采用|基于)\s+(\S+)", "uses"),
            (r"(\S+)\s+(是|称为|定义)\s+(\S+)", "is"),
            (r"(\S+)\s+(包含|包括)\s+(\S+)", "contains"),
            (r"(\S+)\s+(依赖|依赖)\s+(\S+)", "depends_on"), 
            (r"(\S+)\s+(由|由于)\s+(\S+)\s+(开发|创建|设计)", "developed_by"),
            (r"(\S+)\s+(发布|推出)\s+于\s+(\S+)", "released_on"),
        ];

        let entity_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();

        for (pattern, relation_type) in relation_patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(content) {
                let text = cap.as_str();
                for from_entity in &entity_names {
                    for to_entity in &entity_names {
                        if from_entity != to_entity && text.contains(from_entity) && text.contains(to_entity) {
                            let key = format!("{}->{}->{}", from_entity, relation_type, to_entity);
                            if !seen.contains(&key) {
                                seen.insert(key);
                                relations.push(ExtractedRelation {
                                    from_entity: from_entity.to_string(),
                                    to_entity: to_entity.to_string(),
                                    relation_type: relation_type.to_string(),
                                    evidence: text.to_string(),
                                    confidence: 0.65,
                                });
                            }
                        }
                    }
                }
            }
        }

        relations
    }

    pub async fn get_entity_by_id(&self, entity_id: i64) -> Result<KnowledgeEntity, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        crate::schema::knowledge_entities::table
            .find(entity_id)
            .first::<KnowledgeEntity>(&mut conn)
            .map_err(|e| format!("Failed to get entity: {}", e))
    }

    pub async fn get_entities_by_project(&self, project_id: i64) -> Result<Vec<KnowledgeEntity>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .order(crate::schema::knowledge_entities::created_at.desc())
            .load::<KnowledgeEntity>(&mut conn)
            .map_err(|e| format!("Failed to get entities: {}", e))
    }

    pub async fn get_relations_by_entity(&self, entity_id: i64) -> Result<Vec<KnowledgeRelation>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        crate::schema::knowledge_relations::table
            .filter(crate::schema::knowledge_relations::source_entity_id.eq(entity_id))
            .or_filter(crate::schema::knowledge_relations::target_entity_id.eq(entity_id))
            .order(crate::schema::knowledge_relations::created_at.desc())
            .load::<KnowledgeRelation>(&mut conn)
            .map_err(|e| format!("Failed to get relations: {}", e))
    }

    pub async fn search_entities(&self, query: &str, project_id: Option<i64>) -> Result<Vec<KnowledgeEntity>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        let mut query_builder = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.like(format!("%{}%", query)))
            .into_boxed();

        if let Some(pid) = project_id {
            query_builder = query_builder.filter(crate::schema::knowledge_entities::project_id.eq(pid));
        }

        query_builder
            .order(crate::schema::knowledge_entities::confidence.desc())
            .limit(20)
            .load::<KnowledgeEntity>(&mut conn)
            .map_err(|e| format!("Failed to search entities: {}", e))
    }

    pub async fn delete_entity(&self, entity_id: i64) -> Result<(), String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;
        
        diesel::delete(crate::schema::knowledge_relations::table)
            .filter(crate::schema::knowledge_relations::source_entity_id.eq(entity_id))
            .or_filter(crate::schema::knowledge_relations::target_entity_id.eq(entity_id))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to delete relations: {}", e))?;

        diesel::delete(crate::schema::knowledge_entities::table.find(entity_id))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to delete entity: {}", e))?;

        Ok(())
    }
}
