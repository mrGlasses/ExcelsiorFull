use crate::engine::cache_engine::CachePool;
use crate::engine::db_engine::DbPool;
use std::sync::Arc;

// The app's shared state: what every route handler can reach.
// Default exists only in tests: each field defaults to an empty mock, so a unit test fills
// just the field it needs: `AppState { db_pool: ..., ..Default::default() }`. An empty mock
// fails the test if it's ever called, so a test can't touch a resource by accident.
#[derive(Clone)]
#[cfg_attr(test, derive(Default))]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub cache_pool: Arc<CachePool>,
}

// Called once at startup: connects to every service the app uses.
pub async fn build_app_state() -> AppState {
    AppState {
        db_pool: crate::database::connection::start().await,
        cache_pool: crate::database::redis_connection::start().await,
    }
}
