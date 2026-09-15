use crate::state::AppState;
use anyhow::Result;
use axum::extract::State;
#[cfg(test)]
use mockall::automock;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, ExistenceCheck, SetOptions};

pub const USERS_CACHE_KEY: &str = "cache:users:all";
pub const USERS_CACHE_TTL_SECONDS: u64 = 60;

// Namespaced separately from USERS_CACHE_KEY so the generic key/value demo endpoints
// (create/read/update/delete) can't collide with or clobber the get_users cache-aside key.
const DEMO_ENTRY_PREFIX: &str = "demo:";

fn namespaced_key(key: &str) -> String {
    format!("{DEMO_ENTRY_PREFIX}{key}")
}

// Wrapper type that can be either a real Redis connection manager or a mock (in tests)
pub enum CachePool {
    Real(ConnectionManager),
    #[cfg(test)]
    Mock(MockCacheExecutor),
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait CacheExecutor: Send + Sync {
    async fn execute_get_cached_users(&self) -> Result<Option<String>>;
    async fn execute_set_cached_users(&self, users_json: String) -> Result<()>;
    async fn execute_invalidate_users_cache(&self) -> Result<()>;

    /// Sets `key` only if it doesn't already exist. Returns `false` (no-op) if it does.
    async fn execute_create_entry(&self, key: String, value: String) -> Result<bool>;
    async fn execute_get_entry(&self, key: String) -> Result<Option<String>>;
    /// Sets `key` only if it already exists. Returns `false` (no-op) if it doesn't.
    async fn execute_update_entry(&self, key: String, value: String) -> Result<bool>;
    /// Returns `false` if `key` didn't exist.
    async fn execute_delete_entry(&self, key: String) -> Result<bool>;
}

#[async_trait::async_trait]
impl CacheExecutor for ConnectionManager {
    async fn execute_get_cached_users(&self) -> Result<Option<String>> {
        let mut conn = self.clone();
        let cached: Option<String> = conn.get(USERS_CACHE_KEY).await?;
        Ok(cached)
    }

    async fn execute_set_cached_users(&self, users_json: String) -> Result<()> {
        let mut conn = self.clone();
        conn.set_ex::<_, _, ()>(USERS_CACHE_KEY, users_json, USERS_CACHE_TTL_SECONDS)
            .await?;
        Ok(())
    }

    async fn execute_invalidate_users_cache(&self) -> Result<()> {
        let mut conn = self.clone();
        conn.del::<_, ()>(USERS_CACHE_KEY).await?;
        Ok(())
    }

    async fn execute_create_entry(&self, key: String, value: String) -> Result<bool> {
        let mut conn = self.clone();
        let options = SetOptions::default().conditional_set(ExistenceCheck::NX);
        let result: Option<String> = conn
            .set_options(namespaced_key(&key), value, options)
            .await?;
        Ok(result.is_some())
    }

    async fn execute_get_entry(&self, key: String) -> Result<Option<String>> {
        let mut conn = self.clone();
        let value: Option<String> = conn.get(namespaced_key(&key)).await?;
        Ok(value)
    }

    async fn execute_update_entry(&self, key: String, value: String) -> Result<bool> {
        let mut conn = self.clone();
        let options = SetOptions::default().conditional_set(ExistenceCheck::XX);
        let result: Option<String> = conn
            .set_options(namespaced_key(&key), value, options)
            .await?;
        Ok(result.is_some())
    }

    async fn execute_delete_entry(&self, key: String) -> Result<bool> {
        let mut conn = self.clone();
        let deleted: usize = conn.del(namespaced_key(&key)).await?;
        Ok(deleted > 0)
    }
}

#[async_trait::async_trait]
impl CacheExecutor for CachePool {
    async fn execute_get_cached_users(&self) -> Result<Option<String>> {
        match self {
            CachePool::Real(conn) => conn.execute_get_cached_users().await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_get_cached_users().await,
        }
    }

    async fn execute_set_cached_users(&self, users_json: String) -> Result<()> {
        match self {
            CachePool::Real(conn) => conn.execute_set_cached_users(users_json).await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_set_cached_users(users_json).await,
        }
    }

    async fn execute_invalidate_users_cache(&self) -> Result<()> {
        match self {
            CachePool::Real(conn) => conn.execute_invalidate_users_cache().await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_invalidate_users_cache().await,
        }
    }

    async fn execute_create_entry(&self, key: String, value: String) -> Result<bool> {
        match self {
            CachePool::Real(conn) => conn.execute_create_entry(key, value).await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_create_entry(key, value).await,
        }
    }

    async fn execute_get_entry(&self, key: String) -> Result<Option<String>> {
        match self {
            CachePool::Real(conn) => conn.execute_get_entry(key).await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_get_entry(key).await,
        }
    }

    async fn execute_update_entry(&self, key: String, value: String) -> Result<bool> {
        match self {
            CachePool::Real(conn) => conn.execute_update_entry(key, value).await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_update_entry(key, value).await,
        }
    }

    async fn execute_delete_entry(&self, key: String) -> Result<bool> {
        match self {
            CachePool::Real(conn) => conn.execute_delete_entry(key).await,
            #[cfg(test)]
            CachePool::Mock(mock) => mock.execute_delete_entry(key).await,
        }
    }
}

pub async fn get_cached_users_call(State(state): State<AppState>) -> Result<Option<String>> {
    state.cache_pool.execute_get_cached_users().await
}

pub async fn set_cached_users_call(
    State(state): State<AppState>,
    users_json: String,
) -> Result<()> {
    state.cache_pool.execute_set_cached_users(users_json).await
}

pub async fn invalidate_users_cache_call(State(state): State<AppState>) -> Result<()> {
    state.cache_pool.execute_invalidate_users_cache().await
}

pub async fn create_entry_call(
    State(state): State<AppState>,
    key: String,
    value: String,
) -> Result<bool> {
    state.cache_pool.execute_create_entry(key, value).await
}

pub async fn get_entry_call(State(state): State<AppState>, key: String) -> Result<Option<String>> {
    state.cache_pool.execute_get_entry(key).await
}

pub async fn update_entry_call(
    State(state): State<AppState>,
    key: String,
    value: String,
) -> Result<bool> {
    state.cache_pool.execute_update_entry(key, value).await
}

pub async fn delete_entry_call(State(state): State<AppState>, key: String) -> Result<bool> {
    state.cache_pool.execute_delete_entry(key).await
}
