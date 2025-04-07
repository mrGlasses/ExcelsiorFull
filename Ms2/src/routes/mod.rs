use axum::{Router, routing::{get}};
use crate::{state::AppState};
use crate::handlers::simple_handler;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/pong", get(simple_handler::get_answer1))
        .with_state(state)
}