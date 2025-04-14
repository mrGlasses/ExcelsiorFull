use axum::{Router, routing::{get, post}};
use crate::{handlers::db_handler, state::AppState};
use crate::handlers::simple_handler;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/users", get(db_handler::get_users))
        .route("/users", post(db_handler::create_user))
        .route("/ping", get(simple_handler::get_pong))
        .route("/itsaRainyDay", get(simple_handler::call_external_service))
        .with_state(state)
}

//more than 1 route file? search for "axum merge routes"