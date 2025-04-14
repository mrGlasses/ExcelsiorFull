use axum::{Json, extract::State, http::StatusCode};
use axum::response::{IntoResponse, Response};
use crate::domain::database::NewUser;
use crate::engine;
use crate::engine::db_engine::create_user_db_call;
use crate::state::AppState;

pub async fn get_users(State(state): State<AppState>) -> Response { //Response can be a variant - https://docs.rs/axum/latest/axum/response/index.html#returning-different-response-types
    let vec_users = engine::db_engine::get_users_db_call(State(state)).await;
    
    match vec_users {
        Ok(users) => {
            (
                StatusCode::OK,
                Json(users)
            ).into_response()
        }
        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get users: {}", err).into_response()
            ).into_response()
        }
    }
}



pub async fn create_user(State(state): State<AppState>, Json(payload): Json<NewUser>) -> Response {
    match create_user_db_call(State(state), payload.name.clone()).await {
        Ok(_) => {
            (
                StatusCode::CREATED,
                "CREATED"
            ).into_response()
        }
        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to insert user: {}", err).into_response()
            ).into_response()
        }
    }
}