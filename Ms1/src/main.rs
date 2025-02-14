use std::net::SocketAddr;
use std::sync::Arc;
use dotenv::dotenv;
use tower_http::trace::TraceLayer;
use tracing::info;
use crate::routes::create_routes;

mod state;
mod routes;
mod db;
mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_pool = db::connection::init_db().await.expect("Failed to connect to DB");
    let app_state = state::AppState { db_pool: Arc::from(db_pool.clone()) };

    let app = create_routes(app_state).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}