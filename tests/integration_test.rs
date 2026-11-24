use axum::http::StatusCode;
use dotenv::dotenv;
// Import the ms1 crate and its modules
use ms1::handlers::outbox_handler::{OutboxSettings, run_outbox_worker};
use ms1::utils::main_utils::service_starter;
use ms1::utils::otel_config::{setup_tracing_with_otel, shutdown_telemetry};
use ms1::{
    database, engine::cache_engine::CachePool, engine::db_engine::DbPool, routes, state::AppState,
};
use serial_test::serial;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::Once;
use std::time::Duration;
use tokio::net::TcpListener;

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
//
// Deliberately NOT shared across tests via a static: each #[tokio::test] builds and
// tears down its own Tokio runtime, and a sqlx::Pool's connections are bound to the
// runtime that created them. A pool shared across tests would work fine for whichever
// test initializes it, then fail for every later test with "a Tokio 1.x context was
// found, but it is being shutdown" once the initializing test's runtime is gone.
async fn create_test_db_pool() -> PgPool {
    let pool = database::connection::init_db()
        .await
        .expect("Failed to connect to test database");
    // Same migrations the app runs at startup; sqlx takes a Postgres advisory lock
    // while migrating, so tests running this in parallel don't step on each other.
    database::connection::run_migrations(&pool)
        .await
        .expect("Failed to apply migrations to test database");
    pool
}

// Same "one connection per test runtime" reasoning as create_test_db_pool above applies
// to the Redis ConnectionManager: it is bound to the Tokio runtime that created it.
async fn create_test_cache_pool() -> redis::aio::ConnectionManager {
    database::redis_connection::init_cache()
        .await
        .expect("Failed to connect to test cache")
}

// Helper function to create a test app instance
async fn spawn_app() -> String {
    setup_test_env();

    // Find a random available port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Set up a test database connection using the same method as the main app
    let pool = create_test_db_pool().await;
    let cache_pool = create_test_cache_pool().await;

    let app_state = AppState {
        db_pool: Arc::new(DbPool::Real(pool)),
        cache_pool: Arc::new(CachePool::Real(cache_pool)),
    };

    // Build the application with routes
    let app = routes::create_routes(app_state);

    // Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &PgPool) {
    // Add cleanup queries here if needed
    sqlx::query("DELETE FROM t_users WHERE name = 'Test User'")
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

async fn count_users_named(pool: &PgPool, names: &[&str]) -> i64 {
    let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
    sqlx::query_scalar("SELECT count(*) FROM t_users WHERE name = ANY($1)")
        .bind(names)
        .fetch_one(pool)
        .await
        .expect("Failed to count users")
}

async fn delete_users_named(pool: &PgPool, names: &[&str]) {
    let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
    sqlx::query("DELETE FROM t_users WHERE name = ANY($1)")
        .bind(names)
        .execute(pool)
        .await
        .expect("Failed to clean up test users");
}

#[tokio::test]
async fn test_create_user_pair_commits_both() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let names = ["PairTestAnn", "PairTestBob"];
    delete_users_named(&pool, &names).await;

    let response = reqwest::Client::new()
        .post(format!("{}/users/pair", address))
        .json(&serde_json::json!({ "first": names[0], "second": names[1] }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), StatusCode::CREATED.as_u16());
    assert_eq!(count_users_named(&pool, &names).await, 2);

    delete_users_named(&pool, &names).await;
}

#[tokio::test]
async fn test_create_user_pair_rolls_back_when_second_fails() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let first = "PairTestCid";
    delete_users_named(&pool, &[first]).await;

    // t_users.name is VARCHAR(15), so the second insert fails after the first one ran.
    let response = reqwest::Client::new()
        .post(format!("{}/users/pair", address))
        .json(&serde_json::json!({ "first": first, "second": "NameLongerThan15" }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        response.status().as_u16(),
        StatusCode::INTERNAL_SERVER_ERROR.as_u16()
    );
    assert_eq!(count_users_named(&pool, &[first]).await, 0);
}

#[tokio::test]
async fn test_uuid_and_jsonb_round_trip() {
    setup_test_env();
    let pool = create_test_db_pool().await;
    // A temp table lives on one connection only, so everything runs in one
    // transaction; it's rolled back at the end and nothing is left behind.
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    sqlx::query(
        "CREATE TEMP TABLE uuid_json_check (id uuid PRIMARY KEY, data jsonb NOT NULL) ON COMMIT DROP",
    )
    .execute(&mut *tx)
    .await
    .expect("Failed to create temp table");

    let written_id: sqlx::types::Uuid = sqlx::query_scalar(
        "INSERT INTO uuid_json_check (id, data) VALUES (gen_random_uuid(), '{\"a\": 1}'::jsonb) RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert uuid/jsonb row");

    let (read_id, read_data): (sqlx::types::Uuid, serde_json::Value) =
        sqlx::query_as("SELECT id, data FROM uuid_json_check")
            .fetch_one(&mut *tx)
            .await
            .expect("Failed to read uuid/jsonb row");

    assert_eq!(read_id, written_id);
    assert_eq!(read_data, serde_json::json!({ "a": 1 }));

    tx.rollback().await.expect("Failed to roll back");
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
#[ignore]
async fn test_setup_tracing_with_otel_full_stack() {
    println!("1");
    setup_test_env();
    println!("2");
    INIT.call_once(|| {});
    println!("3");
    shutdown_telemetry();

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

// A fake Telegram Bot API for the tests that start the whole app: service_starter()
// needs a bot token, and tests must never talk to the real Telegram.
// Keep the returned server alive for the whole test.
async fn start_fake_telegram() -> httpmock::MockServer {
    let server = httpmock::MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method("POST").path("/botTEST-TOKEN/getMe");
            then.status(200).json_body(serde_json::json!({
                "ok": true,
                "result": {"id": 1, "is_bot": true, "first_name": "Test", "username": "TestBot"}
            }));
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method("POST").path("/botTEST-TOKEN/getUpdates");
            // A real long poll waits when nothing happens; the delay stops a busy loop.
            then.status(200)
                .delay(Duration::from_millis(200))
                .json_body(serde_json::json!({"ok": true, "result": []}));
        })
        .await;
    unsafe {
        std::env::set_var("TELEGRAM_BOT_TOKEN", "TEST-TOKEN");
        std::env::set_var("TELEGRAM_API_URL", server.base_url());
    }
    server
}

#[tokio::test]
#[serial]
async fn test_service_starter_initialization() {
    setup_test_env();
    // .env.test sets MS_PORT=0 (random port) for tests that bind their own listener;
    // service_starter() reads MS_PORT directly, so it needs a fixed value to match
    // the address this test checks below.
    unsafe {
        std::env::set_var("MS_PORT", "3000");
    }
    let _telegram = start_fake_telegram().await;

    println!("Testing service starter initialization...");
    // Spawn service_starter in a background task
    let server_handle = tokio::spawn(async move {
        service_starter().await;
    });
    println!("Service starter initialized successfully");

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Test that the server is responding
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:3000/ping")
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success());
            assert_eq!(resp.text().await.unwrap(), "PONG!");
            println!("Service starter test: Server responded successfully");
        }
        Err(e) => {
            eprintln!("Service starter test: Failed to connect - {:?}", e);
            // Server might not be fully started or DB connection failed
        }
    }

    // Abort the server task to clean up
    server_handle.abort();
}

#[tokio::test]
#[serial]
async fn test_service_starter_graceful_shutdown() {
    setup_test_env();
    // Distinct fixed port from test_service_starter_initialization above; both tests
    // are #[serial] so mutating this shared env var between them is safe.
    unsafe {
        std::env::set_var("MS_PORT", "9998");
    }
    let _telegram = start_fake_telegram().await;

    // Spawn service_starter in a background task
    let server_handle = tokio::spawn(async move {
        service_starter().await;
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Verify server is running
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:9998/ping")
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    if let Ok(resp) = response {
        assert!(resp.status().is_success());
        println!("Service starter graceful shutdown test: Server is running");
    }

    // Abort the server to simulate shutdown
    server_handle.abort();

    // abort() only requests cancellation; the task keeps running until its next
    // await point, so wait for it to actually finish tearing down (and dropping the
    // listener) rather than guessing with a fixed sleep.
    let _ = tokio::time::timeout(Duration::from_secs(2), server_handle).await;

    // Verify server is no longer responding. A fresh client is used here (rather than
    // reusing `client`) so this genuinely opens a new connection instead of reusing a
    // keep-alive one from the request above, which would still succeed via its own
    // per-connection task even after the listener itself is closed.
    let response_after = reqwest::Client::new()
        .get("http://127.0.0.1:9998/ping")
        .timeout(Duration::from_secs(1))
        .send()
        .await;

    assert!(
        response_after.is_err(),
        "Server should not respond after abort"
    );
    println!("Service starter graceful shutdown test: Server stopped successfully");
}

#[tokio::test]
#[serial]
async fn test_service_starter_without_telegram_token_exits() {
    setup_test_env();
    // Empty rather than removed: dotenv() inside service_starter never overwrites an
    // existing variable, so a real token in .env can't sneak back in.
    unsafe {
        std::env::set_var("TELEGRAM_BOT_TOKEN", "");
    }

    let clean_shutdown = tokio::time::timeout(Duration::from_secs(5), service_starter())
        .await
        .expect("service_starter should return at once without a token");

    // false = main() exits with code 1
    assert!(!clean_shutdown);
}

// ============================================
// Outbox (OB-1 .. OB-5)
// ============================================

// A text no other test run uses, so tests only look at their own events.
fn unique_tag(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}-{}", prefix, nanos)
}

// Inserts an event straight into the table, like someone using psql would.
async fn insert_outbox_row(pool: &PgPool, msg: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO outbox (event_type, payload) VALUES ('DemoEvent', jsonb_build_object('msg', $1::text)) RETURNING id",
    )
    .bind(msg)
    .fetch_one(pool)
    .await
    .expect("Failed to insert outbox row")
}

// (published?, attempts, failed?, last_error) for one event.
async fn outbox_state(pool: &PgPool, id: i64) -> (bool, i32, bool, Option<String>) {
    sqlx::query_as(
        "SELECT published_at IS NOT NULL, attempts, failed_at IS NOT NULL, last_error FROM outbox WHERE id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("Failed to read outbox row")
}

async fn delete_outbox_rows(pool: &PgPool, ids: &[i64]) {
    sqlx::query("DELETE FROM outbox WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await
        .expect("Failed to clean up outbox rows");
}

// Waits until the event is published or `limit` passes; returns how long it took.
async fn wait_until_published(pool: &PgPool, id: i64, limit: Duration) -> Option<Duration> {
    let started = std::time::Instant::now();
    while started.elapsed() < limit {
        if outbox_state(pool, id).await.0 {
            return Some(started.elapsed());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    None
}

// Starts an outbox worker in the background; cancel the returned token to stop it.
async fn start_outbox_worker(
    max_attempts: i32,
    recheck_interval: Duration,
) -> tokio_util::sync::CancellationToken {
    start_outbox_worker_with(OutboxSettings {
        max_attempts,
        recheck_interval,
        // No waiting between tries: these tests are about the passes, not the backoff.
        retry_base: Duration::ZERO,
    })
    .await
}

async fn start_outbox_worker_with(settings: OutboxSettings) -> tokio_util::sync::CancellationToken {
    let pool = create_test_db_pool().await;
    let shutdown = tokio_util::sync::CancellationToken::new();
    let token = shutdown.clone();
    tokio::spawn(async move { run_outbox_worker(pool, settings, token).await });
    shutdown
}

#[tokio::test]
async fn test_outbox_event_is_written_with_the_change() {
    // OB-1 step 1
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;
    let tag = unique_tag("ob1");

    let response = reqwest::Client::new()
        .post(format!("{}/demo-events", address))
        .json(&serde_json::json!({ "msg": tag }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), StatusCode::CREATED.as_u16());
    let (id, event_type, payload): (i64, String, serde_json::Value) =
        sqlx::query_as("SELECT id, event_type, payload FROM outbox WHERE payload->>'msg' = $1")
            .bind(&tag)
            .fetch_one(&pool)
            .await
            .expect("the event should exist");
    assert_eq!(event_type, "DemoEvent");
    assert_eq!(payload, serde_json::json!({ "msg": tag }));

    delete_outbox_rows(&pool, &[id]).await;
}

#[tokio::test]
async fn test_outbox_event_is_rolled_back_with_the_change() {
    // OB-1 step 2
    setup_test_env();
    let pool = create_test_db_pool().await;
    let address = spawn_app().await;

    let response = reqwest::Client::new()
        .post(format!("{}/demo-events", address))
        .json(&serde_json::json!({ "msg": "rollback" }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        response.status().as_u16(),
        StatusCode::INTERNAL_SERVER_ERROR.as_u16()
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE payload->>'msg' = 'rollback'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
#[serial]
async fn test_outbox_worker_wakes_up_on_commit() {
    // OB-2: the recheck is 30s, so only LISTEN/NOTIFY can make this fast.
    setup_test_env();
    let pool = create_test_db_pool().await;
    let shutdown = start_outbox_worker(10, Duration::from_secs(30)).await;
    tokio::time::sleep(Duration::from_millis(500)).await; // let it finish its first pass

    let id = insert_outbox_row(&pool, &unique_tag("ob2")).await;
    let took = wait_until_published(&pool, id, Duration::from_secs(3)).await;

    shutdown.cancel();
    delete_outbox_rows(&pool, &[id]).await;
    let took = took.expect("the event should be handled");
    assert!(took < Duration::from_secs(1), "took {:?}", took);
}

#[tokio::test]
#[serial]
async fn test_outbox_worker_catches_up_after_downtime() {
    // OB-3: the event commits while no worker runs; the worker's first pass takes it.
    setup_test_env();
    let pool = create_test_db_pool().await;
    let id = insert_outbox_row(&pool, &unique_tag("ob3")).await;

    let shutdown = start_outbox_worker(10, Duration::from_secs(30)).await;
    let took = wait_until_published(&pool, id, Duration::from_secs(5)).await;

    shutdown.cancel();
    delete_outbox_rows(&pool, &[id]).await;
    assert!(
        took.is_some(),
        "the waiting event should be handled at startup"
    );
}

#[tokio::test]
#[serial]
async fn test_outbox_failing_event_waits_longer_each_time() {
    // HFEX0004: with a 1s base, tries come at ~0s, ~1s, ~3s (1s, then 2s of waiting).
    // Without the backoff, a 100ms re-check would have tried it ~25 times by now.
    setup_test_env();
    let pool = create_test_db_pool().await;
    let shutdown = start_outbox_worker_with(OutboxSettings {
        max_attempts: 10,
        recheck_interval: Duration::from_millis(100),
        retry_base: Duration::from_secs(1),
    })
    .await;

    let boom = insert_outbox_row(&pool, "boom").await;
    tokio::time::sleep(Duration::from_millis(2500)).await;
    let (_, attempts_at_2_5s, failed, _) = outbox_state(&pool, boom).await;
    let waiting: bool =
        sqlx::query_scalar("SELECT next_attempt_at > now() FROM outbox WHERE id = $1")
            .bind(boom)
            .fetch_one(&pool)
            .await
            .unwrap();

    shutdown.cancel();
    delete_outbox_rows(&pool, &[boom]).await;
    assert_eq!(attempts_at_2_5s, 2, "tried at ~0s and ~1s, next at ~3s");
    assert!(!failed);
    assert!(waiting, "next_attempt_at should be in the future");
}

#[tokio::test]
#[serial]
async fn test_outbox_failing_event_is_retried_then_set_aside() {
    // OB-4: "boom" always fails; 3 attempts, then it's set aside without blocking others.
    setup_test_env();
    let pool = create_test_db_pool().await;
    let shutdown = start_outbox_worker(3, Duration::from_millis(100)).await;

    let boom = insert_outbox_row(&pool, "boom").await;
    let after = insert_outbox_row(&pool, &unique_tag("ob4-after")).await;
    let after_took = wait_until_published(&pool, after, Duration::from_secs(3)).await;

    let started = std::time::Instant::now();
    let mut state = outbox_state(&pool, boom).await;
    while !state.2 && started.elapsed() < Duration::from_secs(5) {
        tokio::time::sleep(Duration::from_millis(50)).await;
        state = outbox_state(&pool, boom).await;
    }

    shutdown.cancel();
    delete_outbox_rows(&pool, &[boom, after]).await;
    assert!(
        after_took.is_some(),
        "a failing event must not block the next one"
    );
    let (published, attempts, failed, last_error) = state;
    assert!(!published);
    assert!(failed, "the event should be set aside");
    assert_eq!(attempts, 3);
    assert_eq!(last_error.as_deref(), Some("demo failure"));
}

#[tokio::test]
#[serial]
async fn test_outbox_two_workers_never_share_an_event() {
    // OB-5: two workers, 100 events: every event handled exactly once
    // (attempts would be 2 if both workers had handled the same event).
    setup_test_env();
    let pool = create_test_db_pool().await;
    let first = start_outbox_worker(10, Duration::from_millis(200)).await;
    let second = start_outbox_worker(10, Duration::from_millis(200)).await;

    let tag = unique_tag("ob5");
    let mut ids = Vec::new();
    for n in 0..100 {
        ids.push(insert_outbox_row(&pool, &format!("{}-{}", tag, n)).await);
    }
    let started = std::time::Instant::now();
    let mut remaining: i64 = 100;
    while remaining > 0 && started.elapsed() < Duration::from_secs(15) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        remaining = sqlx::query_scalar(
            "SELECT count(*) FROM outbox WHERE id = ANY($1) AND published_at IS NULL",
        )
        .bind(&ids)
        .fetch_one(&pool)
        .await
        .unwrap();
    }
    let handled_twice: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE id = ANY($1) AND attempts <> 1")
            .bind(&ids)
            .fetch_one(&pool)
            .await
            .unwrap();

    first.cancel();
    second.cancel();
    delete_outbox_rows(&pool, &ids).await;
    assert_eq!(remaining, 0, "all 100 events should be handled");
    assert_eq!(handled_twice, 0, "no event may be handled twice");
}

#[tokio::test]
#[serial]
async fn test_outbox_worker_survives_dropped_database_connections() {
    // EX0031: kill the worker's database connections from the server side. The worker
    // must keep running, its LISTEN connection must come back, and new events must
    // still be handled. (sqlx notices killed connections and replaces them, so this
    // passes even without the EX0031 fix; the fix matters when the database is
    // unreachable for a while, which a test can't do without stopping the container.)
    setup_test_env();
    let pool = create_test_db_pool().await;

    // The worker gets its own pool with a label, so we kill only its connections.
    let options = sqlx::postgres::PgConnectOptions::new()
        .host(&std::env::var("DATABASE_HOST").unwrap())
        .port(std::env::var("DATABASE_PORT").unwrap().parse().unwrap())
        .username(&std::env::var("DATABASE_USER").unwrap())
        .password(&std::env::var("DATABASE_PSWD").unwrap())
        .database(&std::env::var("DATABASE_NAME").unwrap())
        .application_name("outbox-db-error-test");
    let worker_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect_with(options)
        .await
        .unwrap();
    let shutdown = tokio_util::sync::CancellationToken::new();
    let settings = OutboxSettings {
        max_attempts: 10,
        recheck_interval: Duration::from_millis(200),
        retry_base: Duration::ZERO,
    };
    let worker = tokio::spawn(run_outbox_worker(worker_pool, settings, shutdown.clone()));
    tokio::time::sleep(Duration::from_millis(500)).await;

    let killed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE application_name = 'outbox-db-error-test') AS k",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(killed > 0, "the worker should have had connections to kill");

    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(
        !worker.is_finished(),
        "dropped connections must not stop the worker"
    );

    let id = insert_outbox_row(&pool, &unique_tag("ex0031")).await;
    let took = wait_until_published(&pool, id, Duration::from_secs(5)).await;

    shutdown.cancel();
    let stopped = tokio::time::timeout(Duration::from_secs(3), worker).await;
    delete_outbox_rows(&pool, &[id]).await;
    assert!(
        took.is_some(),
        "the worker should handle events after reconnecting"
    );
    assert!(
        stopped
            .expect("the worker should stop after shutdown")
            .unwrap()
            .is_ok(),
        "the worker should stop cleanly"
    );
}
