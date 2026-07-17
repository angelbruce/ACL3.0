use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::rdb_repository::DbPool;
use crate::schema;
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, RunQueryDsl};

pub struct VersionService {
    db_pool: Arc<DbPool>,
}

impl VersionService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    pub async fn create_version(
        &self,
        document_id: i64,
        content: &str,
        change_note: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<i64, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let now = chrono::Utc::now().naive_utc();

        let version_number = self.get_next_version_number(document_id).unwrap_or(1);

        let version_id = diesel::insert_into(schema::document_versions::table)
            .values((
                schema::document_versions::document_id.eq(document_id),
                schema::document_versions::version_number.eq(version_number),
                schema::document_versions::content.eq(content),
                schema::document_versions::change_note.eq(change_note),
                schema::document_versions::created_by.eq(created_by),
                schema::document_versions::created_at.eq(now),
            ))
            .returning(schema::document_versions::id)
            .get_result::<i64>(&mut conn)
            .map_err(|e| format!("Failed to create version: {}", e))?;

        Ok(version_id)
    }

    fn get_next_version_number(&self, document_id: i64) -> Result<i32, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let max_version: Option<i32> = schema::document_versions::table
            .filter(schema::document_versions::document_id.eq(document_id))
            .select(diesel::dsl::max(schema::document_versions::version_number))
            .first(&mut conn)
            .map_err(|e| format!("Failed to get max version: {}", e))?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    pub async fn list_versions(&self, document_id: i64, limit: i64, offset: i64) -> Result<Vec<DocumentVersionSummary>, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let versions: Vec<(i64, i64, i32, Option<String>, Option<i64>, chrono::NaiveDateTime)> =
            schema::document_versions::table
                .filter(schema::document_versions::document_id.eq(document_id))
                .order(schema::document_versions::version_number.desc())
                .limit(limit)
                .offset(offset)
                .select((
                    schema::document_versions::id,
                    schema::document_versions::document_id,
                    schema::document_versions::version_number,
                    schema::document_versions::change_note,
                    schema::document_versions::created_by,
                    schema::document_versions::created_at,
                ))
                .load(&mut conn)
                .map_err(|e| format!("Failed to list versions: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| DocumentVersionSummary {
                id: v.0,
                document_id: v.1,
                version_number: v.2,
                change_note: v.3,
                created_by: v.4,
                created_at: v.5.to_string(),
            })
            .collect())
    }

    pub async fn get_version(&self, version_id: i64) -> Result<DocumentVersion, String> {
        let mut conn = self.db_pool.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let version: (i64, i64, i32, String, Option<String>, Option<i64>, chrono::NaiveDateTime) =
            schema::document_versions::table
                .filter(schema::document_versions::id.eq(version_id))
                .select((
                    schema::document_versions::id,
                    schema::document_versions::document_id,
                    schema::document_versions::version_number,
                    schema::document_versions::content,
                    schema::document_versions::change_note,
                    schema::document_versions::created_by,
                    schema::document_versions::created_at,
                ))
                .first(&mut conn)
                .map_err(|e| format!("Failed to get version: {}", e))?;

        Ok(DocumentVersion {
            id: version.0,
            document_id: version.1,
            version_number: version.2,
            content: version.3,
            change_note: version.4,
            created_by: version.5,
            created_at: version.6.to_string(),
        })
    }

    pub async fn compare_versions(
        &self,
        version_a_id: i64,
        version_b_id: i64,
    ) -> Result<DiffResult, String> {
        let version_a = self.get_version(version_a_id).await?;
        let version_b = self.get_version(version_b_id).await?;

        let diff = self.compute_diff(&version_a.content, &version_b.content);

        Ok(DiffResult {
            version_a_id,
            version_b_id,
            version_a_number: version_a.version_number,
            version_b_number: version_b.version_number,
            diffs: diff,
        })
    }

    fn compute_diff(&self, old_text: &str, new_text: &str) -> Vec<DiffSegment> {
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();

        let mut segments = Vec::new();

        let mut i = 0;
        let mut j = 0;

        while i < old_lines.len() && j < new_lines.len() {
            if old_lines[i] == new_lines[j] {
                let mut same = vec![old_lines[i]];
                i += 1;
                j += 1;

                while i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
                    same.push(old_lines[i]);
                    i += 1;
                    j += 1;
                }

                segments.push(DiffSegment {
                    diff_type: DiffType::Unchanged,
                    content: same.join("\n"),
                    old_line_start: (i - same.len() + 1) as i32,
                    new_line_start: (j - same.len() + 1) as i32,
                    line_count: same.len() as i32,
                });
            } else {
                let mut removed = Vec::new();
                let mut added = Vec::new();

                let mut found_match = false;
                let max_lookahead = 50;

                for lookahead in 1..max_lookahead {
                    if i + lookahead < old_lines.len() {
                        for add_idx in 0..lookahead {
                            if j + add_idx < new_lines.len()
                                && old_lines[i + lookahead] == new_lines[j + add_idx]
                            {
                                for k in 0..lookahead {
                                    if i + k < old_lines.len() {
                                        removed.push(old_lines[i + k]);
                                    }
                                }
                                for k in 0..add_idx {
                                    if j + k < new_lines.len() {
                                        added.push(new_lines[j + k]);
                                    }
                                }
                                i += lookahead;
                                j += add_idx;
                                found_match = true;
                                break;
                            }
                        }
                        if found_match {
                            break;
                        }
                    }

                    if j + lookahead < new_lines.len() && !found_match {
                        for rem_idx in 0..lookahead {
                            if i + rem_idx < old_lines.len()
                                && old_lines[i + rem_idx] == new_lines[j + lookahead]
                            {
                                for k in 0..rem_idx {
                                    if i + k < old_lines.len() {
                                        removed.push(old_lines[i + k]);
                                    }
                                }
                                for k in 0..lookahead {
                                    if j + k < new_lines.len() {
                                        added.push(new_lines[j + k]);
                                    }
                                }
                                i += rem_idx;
                                j += lookahead;
                                found_match = true;
                                break;
                            }
                        }
                        if found_match {
                            break;
                        }
                    }
                }

                if !found_match {
                    while i < old_lines.len() {
                        removed.push(old_lines[i]);
                        i += 1;
                    }
                    while j < new_lines.len() {
                        added.push(new_lines[j]);
                        j += 1;
                    }
                }

                if !removed.is_empty() {
                    segments.push(DiffSegment {
                        diff_type: DiffType::Removed,
                        content: removed.join("\n"),
                        old_line_start: (i - removed.len() + 1) as i32,
                        new_line_start: j as i32,
                        line_count: removed.len() as i32,
                    });
                }

                if !added.is_empty() {
                    segments.push(DiffSegment {
                        diff_type: DiffType::Added,
                        content: added.join("\n"),
                        old_line_start: i as i32,
                        new_line_start: (j - added.len() + 1) as i32,
                        line_count: added.len() as i32,
                    });
                }
            }
        }

        segments
    }

    pub async fn rollback_to_version(
        &self,
        document_id: i64,
        version_id: i64,
        _rollback_by: Option<i64>,
    ) -> Result<i64, String> {
        let version = self.get_version(version_id).await?;

        if version.document_id != document_id {
            return Err("Version does not belong to this document".to_string());
        }

        let new_version_id = self
            .create_version(
                document_id,
                &version.content,
                Some(&format!("回滚到版本v{}", version.version_number)),
                None,
            )
            .await?;

        Ok(new_version_id)
    }

    pub async fn get_latest_version(&self, document_id: i64) -> Result<DocumentVersionSummary, String> {
        let versions = self.list_versions(document_id, 1, 0).await?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| "No versions found".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: i64,
    pub document_id: i64,
    pub version_number: i32,
    pub content: String,
    pub change_note: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersionSummary {
    pub id: i64,
    pub document_id: i64,
    pub version_number: i32,
    pub change_note: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub version_a_id: i64,
    pub version_b_id: i64,
    pub version_a_number: i32,
    pub version_b_number: i32,
    pub diffs: Vec<DiffSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSegment {
    pub diff_type: DiffType,
    pub content: String,
    pub old_line_start: i32,
    pub new_line_start: i32,
    pub line_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Unchanged,
}
