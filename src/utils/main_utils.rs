use crate::database::connection::init_db;
use crate::database::redis_connection::init_cache;
use crate::engine::cache_engine::CachePool;
use crate::engine::db_engine::DbPool;
use crate::engine::telegram_engine::{DEFAULT_TELEGRAM_API_URL, TelegramApi};
use crate::handlers::telegram_handler::{POLL_TIMEOUT_SECS, run_telegram_bot};
use crate::handlers::worker_handler::{heartbeat_loop, stuck_loop};
use crate::routes::create_routes;
use crate::state;
use crate::utils::otel_config::{setup_tracing_with_otel, shutdown_telemetry};
use crate::utils::un_utils::start_message;
use crate::utils::workers::{Workers, shutdown_timeout_from};
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

// Returns true for a clean shutdown, false if a worker failed (main exits with code 1).
pub async fn service_starter() -> bool {
    dotenv().ok();
    setup_tracing_with_otel();

    // Checked first, so a missing token stops the app before anything else starts.
    let telegram_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => {
            error!("TELEGRAM_BOT_TOKEN is missing");
            return false;
        }
    };
    let telegram_api_url =
        std::env::var("TELEGRAM_API_URL").unwrap_or_else(|_| DEFAULT_TELEGRAM_API_URL.to_string());
    let telegram_api = TelegramApi::new(&telegram_api_url, &telegram_token, POLL_TIMEOUT_SECS)
        .expect("Failed to build the Telegram HTTP client");

    let db_pool = init_db().await.expect("Failed to connect to DB");
    let cache_pool = init_cache().await.expect("Failed to connect to Redis");
    let app_state = state::AppState {
        db_pool: Arc::new(DbPool::Real(db_pool)),
        cache_pool: Arc::new(CachePool::Real(cache_pool)),
    };

    let app = create_routes(app_state);

    let pre_port = std::env::var("MS_PORT").expect("MS_PORT must be set.");
    let port = pre_port.parse().expect("MS_PORT must be a number.");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let server = TcpListener::bind(&addr).await.unwrap();

    // One token tells everyone to stop: cancelled by Ctrl+C/SIGTERM, or by a failing worker.
    let shutdown = CancellationToken::new();

    let shutdown_timeout =
        shutdown_timeout_from(std::env::var("WORKER_SHUTDOWN_TIMEOUT_SECS").ok());
    let mut workers = Workers::new(shutdown.clone(), shutdown_timeout);
    workers.spawn("heartbeat", heartbeat_loop);
    workers.spawn("telegram", move |shutdown| {
        run_telegram_bot(Box::new(telegram_api), shutdown)
    });
    if std::env::var("DEMO_STUCK_WORKER").is_ok_and(|value| value == "true") {
        workers.spawn("stuck", stuck_loop);
    }

    //info!("Excelsior listening on {}", addr); //tracing mode startup
    start_message(addr.to_string()).await; //default mode startup

    let wait_for_signal = async {
        tokio::select! {
            _ = shutdown_signal() => shutdown.cancel(),
            // A worker failed and cancelled the token first: stop listening for signals.
            _ = shutdown.cancelled() => {},
        }
    };
    let http_server = async {
        axum::serve(server, app)
            .with_graceful_shutdown(shutdown.clone().cancelled_owned())
            .await
            .expect("server error");
    };

    // All three run together, and all three are dropped together if this function is
    // cancelled (as the integration tests do), so nothing keeps running in the background.
    let (_, _, any_worker_failed) =
        tokio::join!(wait_for_signal, http_server, workers.run_until_shutdown());
    info!("server stopped");

    info!("Shutting down OpenTelemetry...");
    shutdown_telemetry();

    !any_worker_failed
}
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    warn!("signal received, starting graceful shutdown");
}
