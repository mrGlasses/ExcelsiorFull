use crate::domain::database::{NewUser, User};
use crate::engine::cache_engine::{CachePool, MockCacheExecutor};
use crate::engine::db_engine::{DbPool, MockDatabaseExecutor};
use crate::handlers::db_handler::*;
use crate::state::AppState;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::{Json, extract::State};
use mockall::predicate::*;
use std::sync::Arc;

fn cache_miss_mock() -> MockCacheExecutor {
    let mut mock_cache = MockCacheExecutor::new();
    mock_cache
        .expect_execute_get_cached_users()
        .times(1)
        .returning(|| Ok(None));
    mock_cache
}

#[tokio::test]
async fn test_get_users_success() {
    let mut mock_executor = MockDatabaseExecutor::new();
    let mock_users = vec![
        User {
            uid: 1,
            name: "Test User 1".to_string(),
        },
        User {
            uid: 2,
            name: "Test User 2".to_string(),
        },
    ];

    mock_executor
        .expect_execute_get_users()
        .times(1)
        .returning(move || Ok(mock_users.clone()));

    let mut mock_cache = cache_miss_mock();
    mock_cache
        .expect_execute_set_cached_users()
        .times(1)
        .returning(|_| Ok(()));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_executor)),
        cache_pool: Arc::new(CachePool::Mock(mock_cache)),
    };

    let response = get_users(State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_users_error() {
    let mut mock_executor = MockDatabaseExecutor::new();

    mock_executor
        .expect_execute_get_users()
        .times(1)
        .returning(|| Err(anyhow!("Database error")));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_executor)),
        cache_pool: Arc::new(CachePool::Mock(cache_miss_mock())),
    };

    let response = get_users(State(state)).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_create_user_success() {
    let mut mock_executor = MockDatabaseExecutor::new();
    let new_user = NewUser {
        name: "Test User".to_string(),
    };

    mock_executor
        .expect_execute_create_user()
        .with(eq("Test User".to_string()))
        .times(1)
        .returning(|_| Ok("OK".to_string()));

    let mut mock_cache = MockCacheExecutor::new();
    mock_cache
        .expect_execute_invalidate_users_cache()
        .times(1)
        .returning(|| Ok(()));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_executor)),
        cache_pool: Arc::new(CachePool::Mock(mock_cache)),
    };

    let response = create_user(State(state), Json(new_user)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_user_error() {
    let mut mock_executor = MockDatabaseExecutor::new();
    let new_user = NewUser {
        name: "Test User".to_string(),
    };

    mock_executor
        .expect_execute_create_user()
        .times(1)
        .returning(|_| Err(anyhow!("Database error")));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_executor)),
        cache_pool: Arc::new(CachePool::Mock(MockCacheExecutor::new())),
    };

    let response = create_user(State(state), Json(new_user)).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
