// Outbox demo: POST /demo-events with {"msg": "hi"}.
use crate::handlers::outbox_handler::create_demo_event;
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new().route("/demo-events", post(create_demo_event))
}
