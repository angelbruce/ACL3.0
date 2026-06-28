//! Verification service
//!
//! Fact verification and conflict detection
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, TextExpressionMethods, RunQueryDsl};

use crate::model::{KnowledgeEntity, KnowledgeRelation, VerificationConflict};
use crate::rdb_repository::{DbPool};
use crate::schema;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactVerificationResult {
    pub facts: Vec<FactCheck>,
    pub overall_confidence: f64,
    pub has_conflicts: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactCheck {
    pub fact: String,
    pub supporting_evidence: Vec<String>,
    pub confidence: f64,
    pub status: FactStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactStatus {
    Supported,
    PartiallySupported,
    Unsupported,
    Conflicting,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphConsistencyResult {
    pub entities: Vec<EntityCheck>,
    pub relations: Vec<RelationCheck>,
    pub consistency_score: f64,
    pub conflicts: Vec<ConflictDetail>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityCheck {
    pub entity_name: String,
    pub exists_in_graph: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelationCheck {
    pub from_entity: String,
    pub to_entity: String,
    pub relation_type: String,
    pub exists_in_graph: bool,
    pub confidence: f64,
    pub conflicting_relations: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConflictDetail {
    pub conflict_type: String,
    pub description: String,
    pub confidence_score: f64,
}

pub struct VerificationService {
    db_pool: Arc<DbPool>,
}

impl VerificationService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    pub async fn verify_facts(&self, query_text: &str, llm_summary: &str, project_id: Option<i64>) -> Result<FactVerificationResult, String> {
        let facts = self.extract_facts(llm_summary);
        let mut fact_checks = Vec::new();
        let mut overall_confidence = 0.0;
        let mut has_conflicts = false;

        for fact in facts {
            let supporting_evidence = self.find_supporting_evidence(&fact, project_id).await?;
            let (confidence, status) = self.assess_fact_support(&fact, &supporting_evidence);
            
            fact_checks.push(FactCheck {
                fact,
                supporting_evidence,
                confidence,
                status,
            });

            overall_confidence += confidence;

            if status == FactStatus::Conflicting {
                has_conflicts = true;
                self.save_conflict(project_id, query_text, llm_summary, "fact_conflict", &format!("Fact conflicts with knowledge base"))
                    .await?;
            }
        }

        let avg_confidence = if fact_checks.is_empty() { 0.0 } else { overall_confidence / fact_checks.len() as f64 };

        Ok(FactVerificationResult {
            facts: fact_checks,
            overall_confidence: avg_confidence,
            has_conflicts,
        })
    }

    pub async fn verify_graph_consistency(&self, query_text: &str, llm_summary: &str, project_id: Option<i64>) -> Result<GraphConsistencyResult, String> {
        let entities = self.extract_entities_from_text(llm_summary);
        let relations = self.extract_relations_from_text(llm_summary);
        
        let mut entity_checks = Vec::new();
        let mut relation_checks = Vec::new();
        let mut conflicts = Vec::new();

        for entity in entities {
            let exists = self.entity_exists(&entity, project_id).await?;
            entity_checks.push(EntityCheck {
                entity_name: entity,
                exists_in_graph: exists,
                confidence: if exists { 0.8 } else { 0.3 },
            });
        }

        for (from_entity, to_entity, relation_type) in relations {
            let exists = self.relation_exists(&from_entity, &to_entity, &relation_type, project_id).await?;
            let conflicting_relations = self.find_conflicting_relations(&from_entity, &to_entity, project_id).await?;

            if !conflicting_relations.is_empty() {
                conflicts.push(ConflictDetail {
                    conflict_type: "relation_conflict".to_string(),
                    description: format!("Relation {}->{}->{} conflicts with existing relations", from_entity, relation_type, to_entity),
                    confidence_score: 0.75,
                });

                self.save_conflict(project_id, query_text, llm_summary, "relation_conflict", &format!("Relation conflict: {}->{}", from_entity, to_entity))
                    .await?;
            }

            relation_checks.push(RelationCheck {
                from_entity,
                to_entity,
                relation_type,
                exists_in_graph: exists,
                confidence: if exists { 0.75 } else { 0.4 },
                conflicting_relations,
            });
        }

        let entity_score = entity_checks.iter().map(|e| e.confidence).sum::<f64>() / entity_checks.len().max(1) as f64;
        let relation_score = relation_checks.iter().map(|r| r.confidence).sum::<f64>() / relation_checks.len().max(1) as f64;
        let consistency_score = (entity_score * 0.6 + relation_score * 0.4) * (1.0 - (conflicts.len() as f64 * 0.1).min(0.5));

        Ok(GraphConsistencyResult {
            entities: entity_checks,
            relations: relation_checks,
            consistency_score,
            conflicts,
        })
    }

    fn extract_facts(&self, text: &str) -> Vec<String> {
        let mut facts = Vec::new();
        
        let patterns = [
            r"([^。！。！？？]+[是|为|等于|等于|称为|定义为][^。！。！？？]+[。！。！？？])",
            r"([^。！。！？？]+[包含|包括|由|组成][^。！。！？？]+[。！。！？？])",
            r"([^。！。！？？]+[提供|支持|使用|采用][^。！。！？？]+[。！。！？？])",
            r"([^。！。！？？]+[依赖|基于][^。！。！？？]+[。！。！？？])",
            r"([^\n]+[\d]+[^。！。！？？]*[。！。！？？])",
        ];

        for pattern in patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(text) {
                let fact = cap.as_str().trim().to_string();
                if fact.len() >= 10 && fact.len() <= 200 {
                    facts.push(fact);
                }
            }
        }

        if facts.is_empty() {
            for sentence in text.split(|c| c == '。' || c == '！' || c == '？' || c == '.' || c == '!') {
                let trimmed = sentence.trim();
                if trimmed.len() >= 10 && trimmed.len() <= 200 {
                    facts.push(trimmed.to_string());
                }
            }
        }

        facts.into_iter().take(10).collect()
    }

    async fn find_supporting_evidence(&self, fact: &str, project_id: Option<i64>) -> Result<Vec<String>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let keywords: Vec<String> = fact
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= 2)
            .map(|s| s.to_lowercase())
            .take(5)
            .collect();

        if keywords.is_empty() {
            return Ok(Vec::new());
        }

        let mut evidence = Vec::new();

        for keyword in keywords {
            let results = crate::schema::knowledge_points::table
                .filter(crate::schema::knowledge_points::point_content.like(format!("%{}%", keyword)))
                .limit(3)
                .load::<crate::model::KnowledgePoint>(&mut conn)
                .map_err(|e| format!("Failed to search knowledge points: {}", e))?;

            for result in results {
                if let Some(content) = result.point_content {
                    if !evidence.contains(&content) {
                        evidence.push(content);
                    }
                }
            }
        }

        Ok(evidence.into_iter().take(5).collect())
    }

    fn assess_fact_support(&self, fact: &str, evidence: &[String]) -> (f64, FactStatus) {
        if evidence.is_empty() {
            return (0.3, FactStatus::Unsupported);
        }

        let mut score = 0.0;
        for ev in evidence {
            let overlap = self.calculate_text_overlap(fact, ev);
            score += overlap;
        }

        let avg_score = score / evidence.len() as f64;

        if avg_score > 0.7 {
            (0.9, FactStatus::Supported)
        } else if avg_score > 0.4 {
            (0.65, FactStatus::PartiallySupported)
        } else {
            (0.35, FactStatus::Unsupported)
        }
    }

    fn calculate_text_overlap(&self, text1: &str, text2: &str) -> f64 {
        let words1: HashSet<String> = text1
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= 2)
            .map(|s| s.to_lowercase())
            .collect();

        let words2: HashSet<String> = text2
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= 2)
            .map(|s| s.to_lowercase())
            .collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection: HashSet<_> = words1.intersection(&words2).collect();
        intersection.len() as f64 / words1.len().max(words2.len()) as f64
    }

    fn extract_entities_from_text(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let patterns = [
            r"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)",
            r"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?\s+(?:Inc|Ltd|Corp|Company|Group|LLC))",
            r"(?:https?://)?([a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+(?:/[^\s]*)?)",
        ];

        for pattern in patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(text) {
                entities.push(cap.as_str().trim().to_string());
            }
        }

        entities.into_iter().take(20).collect()
    }

    fn extract_relations_from_text(&self, text: &str) -> Vec<(String, String, String)> {
        let mut relations = Vec::new();
        let patterns = [
            (r"(\S+)\s+(提供|提供�?\s+(\S+)", "provides"),
            (r"(\S+)\s+(支持|支持�?\s+(\S+)", "supports"),
            (r"(\S+)\s+(使用|采用|基于)\s+(\S+)", "uses"),
            (r"(\S+)\s+(是|称为)\s+(\S+)", "is"),
            (r"(\S+)\s+(包含|包括)\s+(\S+)", "contains"),
            (r"(\S+)\s+(依赖|依赖�?\s+(\S+)", "depends_on"),
        ];

        for (pattern, relation_type) in patterns {
            for cap in regex::Regex::new(pattern).unwrap().find_iter(text) {
                let text = cap.as_str();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 3 {
                    relations.push((parts[0].to_string(), parts[parts.len()-1].to_string(), relation_type.to_string()));
                }
            }
        }

        relations.into_iter().take(10).collect()
    }

    async fn entity_exists(&self, entity_name: &str, project_id: Option<i64>) -> Result<bool, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let count = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.eq(entity_name))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| format!("Failed to check entity: {}", e))?;

        Ok(count > 0)
    }

    async fn relation_exists(&self, from_entity: &str, to_entity: &str, relation_type: &str, project_id: Option<i64>) -> Result<bool, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let from = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.eq(from_entity))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .first::<KnowledgeEntity>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get from entity: {}", e))?;

        let to = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.eq(to_entity))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .first::<KnowledgeEntity>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get to entity: {}", e))?;

        if let (Some(from), Some(to)) = (from, to) {
            let count = crate::schema::knowledge_relations::table
                .filter(crate::schema::knowledge_relations::source_entity_id.eq(from.id))
                .filter(crate::schema::knowledge_relations::target_entity_id.eq(to.id))
                .filter(crate::schema::knowledge_relations::relation_type.eq(relation_type))
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| format!("Failed to check relation: {}", e))?;

            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    async fn find_conflicting_relations(&self, from_entity: &str, to_entity: &str, project_id: Option<i64>) -> Result<Vec<String>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let from = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.eq(from_entity))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .first::<KnowledgeEntity>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get from entity: {}", e))?;

        let to = crate::schema::knowledge_entities::table
            .filter(crate::schema::knowledge_entities::name.eq(to_entity))
            .filter(crate::schema::knowledge_entities::project_id.eq(project_id))
            .first::<KnowledgeEntity>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get to entity: {}", e))?;

        if let (Some(from), Some(to)) = (from, to) {
            let relations = crate::schema::knowledge_relations::table
                .filter(crate::schema::knowledge_relations::source_entity_id.eq(from.id))
                .filter(crate::schema::knowledge_relations::target_entity_id.eq(to.id))
                .load::<KnowledgeRelation>(&mut conn)
                .map_err(|e| format!("Failed to get relations: {}", e))?;

            Ok(relations.iter()
                .filter_map(|r| r.relation_type.clone())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_conflict(&self, project_id: Option<i64>, query_text: &str, llm_summary: &str, conflict_type: &str, description: &str) -> Result<(), String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        diesel::insert_into(crate::schema::verification_conflicts::table)
            .values((
                crate::schema::verification_conflicts::project_id.eq(project_id),
                crate::schema::verification_conflicts::query_text.eq(query_text),
                crate::schema::verification_conflicts::llm_summary.eq(llm_summary),
                crate::schema::verification_conflicts::conflict_type.eq(conflict_type),
                crate::schema::verification_conflicts::conflict_description.eq(description),
                crate::schema::verification_conflicts::confidence_score.eq(0.7),
                crate::schema::verification_conflicts::resolved.eq(false),
                crate::schema::verification_conflicts::created_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to save conflict: {}", e))?;

        Ok(())
    }

    pub async fn list_conflicts(&self, project_id: Option<i64>, resolved: Option<bool>, limit: usize) -> Result<Vec<VerificationConflict>, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let mut query = crate::schema::verification_conflicts::table
            .order(crate::schema::verification_conflicts::created_at.desc())
            .limit(limit as i64)
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(crate::schema::verification_conflicts::project_id.eq(pid));
        }

        if let Some(r) = resolved {
            query = query.filter(crate::schema::verification_conflicts::resolved.eq(r));
        }

        query.load::<VerificationConflict>(&mut conn)
            .map_err(|e| format!("Failed to list conflicts: {}", e))
    }

    pub async fn resolve_conflict(&self, conflict_id: i64, resolution: &str) -> Result<(), String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("Failed to get DB connection: {}", e))?;

        diesel::update(crate::schema::verification_conflicts::table.find(conflict_id))
            .set((
                crate::schema::verification_conflicts::resolved.eq(true),
                crate::schema::verification_conflicts::resolution.eq(resolution),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to resolve conflict: {}", e))?;

        Ok(())
    }
}
