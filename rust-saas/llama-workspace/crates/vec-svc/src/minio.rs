//! MinIO 对象存储服务
//!
//! 提供 MinIO 对象存储服务，用于存储和检索文件。

use futures::StreamExt;
use axum::http::Method;

use minio::s3::MinioClient;
use minio::s3::MinioClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::types::{BucketName, ObjectKey, S3Api, ToStream};
use minio::s3::builders::ObjectContent;
use minio::s3::response::{BucketExistsResponse, GetPresignedObjectUrlResponse};

#[derive(Debug, Clone)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

impl Default for MinioConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
            bucket: "vec-svc".to_string(),
        }
    }
}

impl MinioConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket: std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "vec-svc".to_string()),
        }
    }
}

pub struct MinioService {
    client: MinioClient,
    bucket: String,
}

impl MinioService {
    pub fn new(config: MinioConfig) -> Result<Self, String> {
        let static_provider = StaticProvider::new(&config.access_key, &config.secret_key, None);
        
        let client = MinioClientBuilder::new(config.endpoint.parse().map_err(|e| format!("Invalid MinIO endpoint: {}", e))?)
            .provider(Some(static_provider))
            .build()
            .map_err(|e| format!("Failed to create MinIO client: {}", e))?;

        Ok(Self {
            client,
            bucket: config.bucket,
        })
    }

    pub async fn upload_file(
        &self,
        project_id: Option<i64>,
        file_name: &str,
        content: &[u8],
        _content_type: &str,
    ) -> Result<String, String> {
        let object_name = Self::generate_object_key(project_id, file_name);
        
        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;
        let object_key = ObjectKey::new(&object_name).map_err(|e| format!("Invalid object key: {}", e))?;
        
        let object_content = ObjectContent::from(content.to_vec());
        
        self.client
            .put_object_content(bucket_name, object_key, object_content)
            .map_err(|e| format!("Failed to build put object request: {}", e))?
            .build()
            .send()
            .await
            .map_err(|e| format!("Failed to upload file: {}", e))?;

        Ok(object_name)
    }

    pub async fn download_file(&self, object_name: &str) -> Result<Vec<u8>, String> {
        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;
        let object_key = ObjectKey::new(object_name).map_err(|e| format!("Invalid object key: {}", e))?;

        let response = self.client
            .get_object(bucket_name, object_key)
            .map_err(|e| format!("Failed to build get object request: {}", e))?
            .build()
            .send()
            .await
            .map_err(|e| format!("Failed to download file: {}", e))?;

        let content = response
            .content()
            .map_err(|e| format!("Failed to get object content: {}", e))?;

        let bytes = content
            .to_segmented_bytes()
            .await
            .map_err(|e| format!("Failed to read object bytes: {}", e))?
            .to_bytes();

        Ok(bytes.to_vec())
    }

    pub async fn delete_file(&self, object_name: &str) -> Result<(), String> {
        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;

        self.client
            .delete_object(bucket_name, object_name)
            .map_err(|e| format!("Failed to build delete object request: {}", e))?
            .build()
            .send()
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        Ok(())
    }

    pub async fn get_presigned_url(&self, object_name: &str, expires_in_seconds: u32) -> Result<String, String> {
        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;
        let object_key = ObjectKey::new(object_name).map_err(|e| format!("Invalid object key: {}", e))?;

        let response: GetPresignedObjectUrlResponse = self.client
            .get_presigned_object_url(bucket_name, object_key, Method::GET)
            .map_err(|e| format!("Failed to build presigned URL request: {}", e))?
            .build()
            .send()
            .await
            .map_err(|e| format!("Failed to generate presigned URL: {}", e))?;

        Ok(response.url.to_string())
    }

    pub async fn list_objects(&self, project_id: Option<i64>) -> Result<Vec<String>, String> {
        let prefix = match project_id {
            Some(pid) => format!("project_{}/", pid),
            None => "".to_string(),
        };

        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;

        let mut stream = self.client
            .list_objects(bucket_name)
            .map_err(|e| format!("Failed to build list objects request: {}", e))?
            .prefix(prefix)
            .build()
            .to_stream()
            .await;

        let mut objects = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for obj in response.contents {
                        objects.push(obj.name);
                    }
                }
                Err(e) => return Err(format!("Failed to list objects: {}", e)),
            }
        }

        Ok(objects)
    }

    pub async fn bucket_exists(&self) -> Result<bool, String> {
        let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;

        let response: BucketExistsResponse = self.client
            .bucket_exists(bucket_name)
            .map_err(|e| format!("Failed to build bucket exists request: {}", e))?
            .build()
            .send()
            .await
            .map_err(|e| format!("Failed to check bucket: {}", e))?;

        Ok(response.exists())
    }

    pub async fn create_bucket_if_not_exists(&self) -> Result<(), String> {
        if !self.bucket_exists().await? {
            let bucket_name = BucketName::new(&self.bucket).map_err(|e| format!("Invalid bucket name: {}", e))?;
            
            self.client
                .create_bucket(bucket_name)
                .map_err(|e| format!("Failed to build create bucket request: {}", e))?
                .build()
                .send()
                .await
                .map_err(|e| format!("Failed to create bucket: {}", e))?;
        }
        Ok(())
    }

    pub async fn list_project_files(&self, project_id: i64) -> Result<Vec<String>, String> {
        self.list_objects(Some(project_id)).await
    }

    pub async fn check_connection(&self) -> bool {
        self.bucket_exists().await.is_ok()
    }

    fn generate_object_key(project_id: Option<i64>, file_name: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let sanitized_name = file_name.replace(|c: char| c.is_ascii_control() || c == '/', "_");
        
        match project_id {
            Some(pid) => format!("project_{}/{}_{}", pid, timestamp, sanitized_name),
            None => format!("global/{}_{}", timestamp, sanitized_name),
        }
    }
}
