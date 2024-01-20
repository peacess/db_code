use std::sync::Arc;

use sqlx::SqlitePool;

pub trait Dao<M> {
    fn pool(&self) -> &SqlitePool;
    fn new(pool: Arc<SqlitePool>) -> Self;

    fn add(&self, m: &mut M) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send;

    fn remove(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send;
    fn remove_all(&self) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send;

    fn update(
        &self,
        m: &mut M,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send;

    /// update with ol(Optimistic Locking)
    fn update_ol(
        &self,
        m: &mut M,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send;

    fn get(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<M>, sqlx::Error>> + Send;
    fn list(&self) -> impl std::future::Future<Output = Result<Vec<M>, sqlx::Error>> + Send;
}

///
pub struct MBase {
    pub id: String,
    /// timestamp, ms
    pub update_ts: i64,
    pub version: i64,
}
