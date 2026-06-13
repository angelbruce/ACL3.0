use async_trait::async_trait;
// use diesel::r2d2::Pool;
// use diesel::pg::PgConnection;
// use std::result::Result;
use crate::errors::ServiceResult;
// use crate::errors::ServiceError;


// 假设这个 Trait 是 DalDataList 要求的契约
#[async_trait]
pub trait DalDataList<T>
where
    T: Send + Sync,
{
    // 增
    async fn create(&self, item: T) -> ServiceResult<T>;
    // 读全部
    async fn find_all(&self) -> ServiceResult<Vec<T>>;
    // 读单个
    async fn find_by_id(&self, id: i64) -> ServiceResult<Option<T>>;
    // 改
    async fn update(&self, id: i64, new_data: T) -> ServiceResult<T>;
    // 删
    async fn delete(&self, id: i64) -> ServiceResult<usize>;
}
