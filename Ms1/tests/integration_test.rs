use std::net::TcpListener;
use axum::http::StatusCode;
use tower::ServiceExt;
use std::sync::Arc;
use sqlx::MySqlPool;
use dotenv::dotenv;

// Import the ms1 crate and its modules
use ms1::{
    db,
    routes,
    state::AppState,
    engine::db_engine::DbPool,
};

// Helper function to set up the test environment
fn setup_test_env() {
    // Try to load .env.test file, fall back to .env if not found
    if std::path::Path::new(".env.test").exists() {
        dotenv::from_filename(".env.test").ok();
    } else {
        dotenv().ok();
        println!("Warning: .env.test not found, using .env file");
    }
}

// Helper function to create a test database connection
async fn create_test_db_pool() -> MySqlPool {
    db::connection::init_db()
        .await
        .expect("Failed to connect to test database")
}

// Helper function to create a test app instance
async fn spawn_app() -> String {
    setup_test_env();

    // Find a random available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    
    // Set up test database connection using the same method as main app
    let pool = create_test_db_pool().await;

    let app_state = AppState {
        db_pool: Arc::new(DbPool::Real(pool)),
    };

    // Build the application with routes
    let app = routes::create_routes(app_state);

    // Spawn the server in the background
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &MySqlPool) {
    // Add cleanup queries here if needed
    sqlx::query("DELETE FROM t_Users WHERE name = 'Test User'")
        .execute(pool)
        .await
        .expect("Failed to clean up test data");
}

#[tokio::test]
async fn test_server_health_check() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/ping", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "PONG!");
}

#[tokio::test]
async fn test_protected_route_unauthorized() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/protected-enter", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_authorized() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/protected-enter", address))
        .header("X-Custom-Header", "secret-value")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_users() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/users", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_user() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/users", address))
        .json(&serde_json::json!({
            "name": "Test User"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::CREATED);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_params() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/params/42/another_p/test-param", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert!(body.contains("42"));
    assert!(body.contains("test-param"));
}

#[tokio::test]
async fn test_get_question_with_filters() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/question_separator", address))
        .query(&[("name", "John"), ("age", "30"), ("active", "true")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert!(body.contains("John"));
    assert!(body.contains("30"));
    assert!(body.contains("true"));
}

//#[tokio::test]
// async fn test_graceful_shutdown() {
//     let address = spawn_app().await;
//     let client = reqwest::Client::new();
// 
//     // Make sure server is up
//     let response = client
//         .get(&format!("{}/pong", address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//     assert!(response.status().is_success());
// 
//     // Server will be dropped at the end of this test
//     // and should shut down gracefully
// } 