use axum::{Json, extract::State};
use sqlx::Row;
use crate::models::{general::Message, database::User, database::NewUser};
use crate::state::AppState;



pub async fn get_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = sqlx::query("CALL sp_Return_USERS();")
        .map(|row: sqlx::mysql::MySqlRow| {
            User {
                uid: row.get(0),
                name: row.get(1)
            }
        })
        .fetch_all(&*state.db_pool)
        .await
        .unwrap_or_else(|_| vec![]);

    Json(users)
}

pub async fn create_user(State(state): State<AppState>, Json(payload): Json<NewUser>) -> Message {
    let _ = sqlx::query(&format!("CALL sp_Insert_User(\"{}\")", payload.name))
        .fetch_optional(&*state.db_pool)
        .await
        .expect("Failed to insert user");


    Message {
        code: 200,
        message_text: "OK".to_string()
    }
}