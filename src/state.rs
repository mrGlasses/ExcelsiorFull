use crate::engine::cache_engine::CachePool;
use crate::engine::db_engine::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub cache_pool: Arc<CachePool>,
}
