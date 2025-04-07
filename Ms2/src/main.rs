use std::env;
use std::net::SocketAddr;
use dotenv::dotenv;
use tower_http::trace::TraceLayer;
use tracing::info;
use crate::routes::create_routes;

mod state;
mod routes;
mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let app_state = state::AppState {noda: "fuck".parse().unwrap() };

    let app = create_routes(app_state).layer(TraceLayer::new_for_http());

    let pre_port = env::var("MS_PORT").expect("MS_PORT must be set.");
    let port = pre_port.parse().expect("MS_PORT must be a number.");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}