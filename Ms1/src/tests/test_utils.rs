use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;

/// Builds a sample app with standard middleware stack
pub fn app_with_middleware(handler: Box<dyn axum::handler::Handler<(), axum::body::Body, Future=()>>) -> Router {
    Router::new()
        .route("/", get(handler))
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(64))
}