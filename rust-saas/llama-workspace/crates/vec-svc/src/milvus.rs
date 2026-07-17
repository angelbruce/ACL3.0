//! Milvus vector database client wrapper
//! 
//! HTTP-based client for Milvus vector database
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum MilvusError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Insert error: {0}")]
    Insert(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Milvus 配置
#[derive(Debug, Clone)]
pub struct MilvusConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl MilvusConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("MILVUS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("MILVUS_PORT")
                .unwrap_or_else(|_| "19530".to_string())
                .parse()
                .unwrap_or(19530),
            database: std::env::var("MILVUS_DATABASE").unwrap_or_else(|_| "default".to_string()),
        }
    }
    
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Milvus client
#[derive(Clone)]
pub struct MilvusClient {
    config: MilvusConfig,
    http_client: reqwest::Client,
}

impl MilvusClient {
    pub fn new(config: MilvusConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
    
    pub async fn has_collection(&self, collection_name: &str) -> Result<bool, MilvusError> {
        let url = format!("{}/v2/vectordb/collections/has", self.config.url());

        let params = serde_json::json!({
            "collectionName": collection_name,
        });

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["data"]["has"].as_bool().unwrap_or(false))
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Connection(format!("Failed to check collection: {:?}", error)))
        }
    }

    pub async fn load_collection(&self, collection_name: &str) -> Result<(), MilvusError> {
        let url = format!("{}/v2/vectordb/collections/load", self.config.url());

        let params = serde_json::json!({
            "collectionName": collection_name,
        });

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Collection '{}' loaded", collection_name);
            Ok(())
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Connection(format!("Failed to load collection: {:?}", error)))
        }
    }

    pub async fn create_collection(&self, collection_name: &str, dimension: usize) -> Result<(), MilvusError> {
        if self.has_collection(collection_name).await? {
            tracing::info!("Collection '{}' already exists", collection_name);
            self.load_collection(collection_name).await.ok();
            return Ok(());
        }

        let url = format!("{}/v2/vectordb/collections/create", self.config.url());

        let params = serde_json::json!({
            "collectionName": collection_name,
            "dimension": dimension,
            "metricType": "COSINE",
            "schema": {
                "fields": [
                    {"fieldName": "id", "dataType": "Int64", "isPrimary": true, "autoID": false},
                    {"fieldName": "vector", "dataType": "FloatVector", "dimension": dimension},
                    {"fieldName": "document", "dataType": "VarChar", "maxLength": 65535},
                    {"fieldName": "chunk_id", "dataType": "Int64"},
                    {"fieldName": "project_id", "dataType": "Int64"}
                ],
                "enableDynamicField": false
            }
        });

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Collection '{}' created", collection_name);
            self.load_collection(collection_name).await?;
            Ok(())
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Connection(format!("Failed to create collection: {:?}", error)))
        }
    }
    
    pub async fn insert(
        &self,
        collection_name: &str,
        vectors: Vec<Vec<f32>>,
        documents: Vec<String>,
        chunk_ids: Vec<i64>,
        project_ids: Vec<Option<i64>>,
    ) -> Result<Vec<i64>, MilvusError> {
        let url = format!("{}/v2/vectordb/entities/insert", self.config.url());

        let data: Vec<serde_json::Value> = vectors
            .into_iter()
            .zip(documents.into_iter())
            .zip(chunk_ids.into_iter())
            .zip(project_ids.into_iter())
            .map(|(((vector, document), chunk_id), project_id)| {
                serde_json::json!({
                    "id": chunk_id,
                    "vector": vector,
                    "document": document,
                    "chunk_id": chunk_id,
                    "project_id": project_id,
                })
            })
            .collect();

        let params = serde_json::json!({
            "collectionName": collection_name,
            "data": data,
        });

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let inserted_ids = result["data"]["insertIds"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                        .collect()
                })
                .unwrap_or_default();
            Ok(inserted_ids)
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Insert(format!("{:?}", error)))
        }
    }
    
    pub async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        project_id: Option<i64>,
    ) -> Result<Vec<SearchResult>, MilvusError> {
        let url = format!("{}/v2/vectordb/entities/search", self.config.url());

        let mut params = serde_json::json!({
            "collectionName": collection_name,
            "data": [query_vector],
            "annsField": "vector",
            "limit": top_k,
            "outputFields": ["id", "document", "chunk_id", "project_id"],
        });

        if let Some(pid) = project_id {
            params["filter"] = serde_json::json!(format!("project_id == {}", pid));
        }

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;

            let results: Vec<SearchResult> = result["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(SearchResult {
                                id: item["id"].as_i64()?,
                                score: item["distance"].as_f64()? as f32,
                                document: item["document"].as_str()?.to_string(),
                                chunk_id: item["chunk_id"].as_i64(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(results)
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Query(format!("{:?}", error)))
        }
    }
    
    pub async fn delete(&self, collection_name: &str, ids: Vec<i64>) -> Result<(), MilvusError> {
        if ids.is_empty() {
            return Ok(());
        }
        let ids_str = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        let filter = format!("id in [{}]", ids_str);
        self.delete_by_filter(collection_name, &filter).await
    }

    pub async fn delete_by_filter(&self, collection_name: &str, filter: &str) -> Result<(), MilvusError> {
        let url = format!("{}/v2/vectordb/entities/delete", self.config.url());

        let params = serde_json::json!({
            "collectionName": collection_name,
            "filter": filter,
        });

        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Query(format!("{:?}", error)))
        }
    }
    
    pub async fn flush(&self, collection_name: &str) -> Result<(), MilvusError> {
        let url = format!("{}/v2/vectordb/collections/flush", self.config.url());
        
        let params = serde_json::json!({
            "collectionNames": [collection_name],
        });
        
        let response = self.http_client
            .post(&url)
            .json(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error: serde_json::Value = response.json().await.unwrap_or_default();
            Err(MilvusError::Query(format!("{:?}", error)))
        }
    }
}

/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub score: f32,
    pub document: String,
    pub chunk_id: Option<i64>,
}

/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRequest {
    pub collection_name: String,
    pub vectors: Vec<Vec<f32>>,
    pub documents: Vec<String>,
    pub chunk_ids: Option<Vec<i64>>,
    pub project_ids: Option<Vec<i64>>,
}

/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub collection_name: String,
    pub query_vector: Vec<f32>,
    pub top_k: usize,
    pub project_id: Option<i64>,
    pub filter: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config() {
        let config = MilvusConfig::from_env();
        println!("Milvus URL: {}", config.url());
    }
}
