use std::sync::Arc;
use crate::engine::db_engine::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>
}