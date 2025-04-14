use crate::routes::create_routes;
use axum::{routing::get, Router};
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info; //#-#

mod db;
mod handlers;
mod routes;
mod state;
mod domain;
mod engine;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_pool = db::connection::init_db()
        .await
        .expect("Failed to connect to DB");
    let app_state = state::AppState {
        db_pool: Arc::new(db_pool),
    };

    let app = create_routes(app_state).layer((TraceLayer::new_for_http(), TimeoutLayer::new(Duration::from_secs(60)))); //?

    let pre_port = std::env::var("MS_PORT").expect("MS_PORT must be set.");
    let port = pre_port.parse().expect("MS_PORT must be a number.");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::debug!("listening on {}", addr);

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    let graceful = server.with_graceful_shutdown(shutdown_signal()); //#-#

    if let Err(err) = graceful.await {
        eprintln!("server error: {}", err); //#-#
    }
}

async fn shutdown_signal() {
    //#-#
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
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
    println!("signal received, starting graceful shutdown"); //#-#
}
