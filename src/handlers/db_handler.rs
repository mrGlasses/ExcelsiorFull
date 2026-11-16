use crate::domain::database::{NewUser, User};
use crate::engine::cache_engine::{
    get_cached_users_call, invalidate_users_cache_call, set_cached_users_call,
};
use crate::engine::db_engine::{create_user_db_call, get_users_db_call};
use crate::state::AppState;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State, http::StatusCode};
use tracing::{error, info, warn};

pub async fn get_users(State(state): State<AppState>) -> Response {
    info!("get_users called");

    match get_cached_users_call(State(state.clone())).await {
        Ok(Some(cached_json)) => {
            if let Ok(users) = serde_json::from_str::<Vec<User>>(&cached_json) {
                info!("Users returned from cache");
                return (StatusCode::OK, Json(users)).into_response();
            }
            warn!("Cached users payload could not be parsed, falling back to the database");
        }
        Ok(None) => info!("Cache miss for users, falling back to the database"),
        Err(err) => warn!("Cache lookup failed, falling back to the database: {}", err),
    }

    let vec_users = get_users_db_call(State(state.clone())).await;
    match vec_users {
        Ok(users) => {
            warn!("Users returned");
            if let Ok(users_json) = serde_json::to_string(&users)
                && let Err(err) = set_cached_users_call(State(state), users_json).await
            {
                warn!("Failed to populate users cache: {}", err);
            }
            (StatusCode::OK, Json(users)).into_response()
        }
        Err(err) => {
            error!("Error occurred: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get users: {}", err).into_response(),
            )
                .into_response()
        }
    }
}

pub async fn create_user(State(state): State<AppState>, Json(payload): Json<NewUser>) -> Response {
    info!("create_user called with params: {:?}", payload);
    match create_user_db_call(State(state.clone()), payload.name.clone()).await {
        Ok(_) => {
            warn!("New user created");
            if let Err(err) = invalidate_users_cache_call(State(state)).await {
                warn!("Failed to invalidate users cache: {}", err);
            }
            (StatusCode::CREATED, "CREATED").into_response()
        }
        Err(err) => {
            error!("Error occurred: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to insert user: {}", err).into_response(),
            )
                .into_response()
        }
    }
}
