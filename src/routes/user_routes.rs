// Users (database + cache-aside): GET/POST /users.
use crate::handlers::db_handler::{create_user, get_users};
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new().route("/users", get(get_users).post(create_user))
}
