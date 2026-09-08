use crate::domain::database::User;
use crate::engine::db_engine::*;
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;

#[tokio::test]
async fn test_get_users_db_call() {
    let mut mock_db = MockDatabaseExecutor::new();

    // Setup mock expectations
    mock_db.expect_execute_get_users().times(1).returning(|| {
        Ok(vec![User {
            uid: 1,
            name: "Test User".to_string(),
        }])
    });

    // Create AppState with our mock wrapped in DbPool
    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    // Test the actual function
    let result = get_users_db_call(State(state)).await.unwrap();

    // Assertions
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Test User");
    assert_eq!(result[0].uid, 1);
}

#[tokio::test]
async fn test_get_users_db_call_empty() {
    let mut mock_db = MockDatabaseExecutor::new();

    mock_db
        .expect_execute_get_users()
        .times(1)
        .returning(|| Ok(vec![]));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    let result = get_users_db_call(State(state)).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_get_users_db_call_multiple_users() {
    let mut mock_db = MockDatabaseExecutor::new();

    mock_db.expect_execute_get_users().times(1).returning(|| {
        Ok(vec![
            User {
                uid: 1,
                name: "Alice".to_string(),
            },
            User {
                uid: 2,
                name: "Bob".to_string(),
            },
            User {
                uid: 3,
                name: "Carol".to_string(),
            },
        ])
    });

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    let result = get_users_db_call(State(state)).await.unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "Alice");
    assert_eq!(result[1].name, "Bob");
    assert_eq!(result[2].name, "Carol");
}

#[tokio::test]
async fn test_get_users_db_call_propagates_error() {
    let mut mock_db = MockDatabaseExecutor::new();

    mock_db
        .expect_execute_get_users()
        .times(1)
        .returning(|| Err(anyhow::anyhow!("connection lost")));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    let result = get_users_db_call(State(state)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "connection lost");
}

#[tokio::test]
async fn test_create_user_db_call() {
    let mut mock_db = MockDatabaseExecutor::new();

    // Setup mock expectations
    mock_db
        .expect_execute_create_user()
        .with(mockall::predicate::eq("Test User".to_string()))
        .times(1)
        .returning(|_| Ok("OK".to_string()));

    // Create AppState with our mock wrapped in DbPool
    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    // Test the actual function
    let result = create_user_db_call(State(state), "Test User".to_string())
        .await
        .unwrap();

    // Assertions
    assert_eq!(result, "OK");
}

#[tokio::test]
async fn test_create_user_db_call_propagates_error() {
    let mut mock_db = MockDatabaseExecutor::new();

    mock_db
        .expect_execute_create_user()
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("duplicate user")));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    let result = create_user_db_call(State(state), "Test User".to_string()).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "duplicate user");
}

#[tokio::test]
async fn test_create_user_db_call_passes_name_through_unmodified() {
    // Regression guard for the SQL-injection fix in execute_create_user: this only
    // verifies the engine layer forwards the raw string as-is (no accidental escaping
    // or truncation of its own) rather than sqlx's own parameter binding, which needs
    // a real database connection to exercise and is covered separately in
    // tests/integration_test.rs.
    let malicious_name = r#"Robert'); DROP TABLE t_Users; --"#.to_string();
    let mut mock_db = MockDatabaseExecutor::new();

    mock_db
        .expect_execute_create_user()
        .with(mockall::predicate::eq(malicious_name.clone()))
        .times(1)
        .returning(|_| Ok("OK".to_string()));

    let state = AppState {
        db_pool: Arc::new(DbPool::Mock(mock_db)),
    };

    let result = create_user_db_call(State(state), malicious_name).await;

    assert_eq!(result.unwrap(), "OK");
}
