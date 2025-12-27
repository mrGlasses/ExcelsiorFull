use axum::http::StatusCode;
use dotenv::dotenv;
use serial_test::serial;
use sqlx::MySqlPool;
use std::env;
use std::net::TcpListener;
use std::sync::Arc;
// Import the ms1 crate and its modules
use ms1::utils::otel_config::{setup_tracing_with_otel, shutdown_telemetry};
use ms1::{database, engine::db_engine::DbPool, routes, state::AppState};
use std::sync::Once;

// Helper function to set up the test environment
static INIT: Once = Once::new();
fn setup_test_env() {
    // Skip dotenv loading if running in CI
    if std::env::var("CI").is_ok() {
        println!("Running in CI - using environment variables from workflow");
        return;
    }

    if std::path::Path::new(".env.test").exists() {
        dotenv::from_filename(".env.test").ok();
    } else {
        dotenv().ok();
        println!("Warning: .env.test not found, using .env file");
    }
}

// Helper function to create a test database connection
async fn create_test_db_pool() -> MySqlPool {
    database::connection::init_db()
        .await
        .expect("Failed to connect to test database")
}

// Helper function to create a test app instance
async fn spawn_app() -> String {
    setup_test_env();

    // Find a random available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // Set up a test database connection using the same method as the main app
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
        .get(format!("{}/ping", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "PONG!");
}

#[tokio::test]
async fn test_its_a_rainy_day_route() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/its-a-rainy-day", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert!(body.contains("Ladaradiradadada! !!"));
}

#[tokio::test]
async fn test_protected_route_unauthorized() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/protected-enter", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_protected_route_authorized() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/protected-enter", address))
        .header("X-Custom-Header", "secret-value")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::UNAUTHORIZED),
        StatusCode::OK
    );
}

#[tokio::test]
async fn test_get_users() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/users", address))
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
        .post(format!("{}/users", address))
        .json(&serde_json::json!({
            "name": "Test User"
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::UNAUTHORIZED),
        StatusCode::CREATED
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_params() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/params/42/another_p/test-param", address))
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
        .get(format!("{}/question_separator", address))
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

#[tokio::test]
async fn test_post_body_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/body-data", address))
        .json(&serde_json::json!({
            "code": 2007,
            "message_text": "Test message"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert!(body.contains("Test message"));
    assert!(body.contains("2007"));
}

#[tokio::test]
#[serial]
async fn test_setup_tracing_with_otel_full_stack() {
    setup_test_env();
    INIT.call_once(|| {
        // Initialize any global test setup here
        env::set_var("RUST_LOG", "info");
    });

    println!("Testing setup_tracing_with_otel with real collector...");

    // This function calls init_telemetry internally and sets up the subscriber
    // Note: This can only be called ONCE per test process due to global subscriber
    let result = std::panic::catch_unwind(|| {
        setup_tracing_with_otel();
    });

    match result {
        Ok(_) => {
            println!("Successfully set up tracing with OpenTelemetry");

            // Test that tracing works
            tracing::info!("Test log message from integration test");
            tracing::debug!("Debug message - should respect RUST_LOG");
            tracing::warn!("Warning message");

            println!("Tracing messages sent successfully");

            // Give time for spans to flush
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            shutdown_telemetry();
            println!("Successfully shut down telemetry");
        }
        Err(e) => {
            // The function might panic if collector is not available
            eprintln!("setup_tracing_with_otel panicked: {:?}", e);
            eprintln!("Make sure OTLP collector is running on localhost:4317");
            panic!("Tracing setup failed");
        }
    }
}
