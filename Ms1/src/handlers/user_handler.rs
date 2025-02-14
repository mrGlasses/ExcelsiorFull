use axum::{Json, extract::State};
use sqlx::{FromRow};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub uid: i32,
    pub name: String,
}

pub async fn get_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("CALL sp_Return_USERS()")
        .fetch_all(&*state.db_pool)
        .await
        .unwrap_or_else(|_| vec![]);

    Json(users)
}

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
}

pub async fn create_user(State(state): State<AppState>, Json(payload): Json<NewUser>) -> Json<User> {
    let user = sqlx::query_as::<_, User>(&format!("CALL InsertUser({})", payload.name))
        .fetch_one(&*state.db_pool)
        .await
        .expect("Failed to insert user");

    Json(user)
}