use sqlx::Row;
use anyhow::Result;
use axum::extract::State;
use crate::domain::database::User;
use crate::state::AppState;
pub async fn get_users_db_call(State(state): State<AppState>) -> Result<Vec<User>> {
    let users = sqlx::query("CALL sp_Return_USERS();")
        .map(|row: sqlx::mysql::MySqlRow| {
            User {
                uid: row.get(0),
                name: row.get(1)
            }
        })
        .fetch_all(&*state.db_pool)
        .await?;

    Ok(users)
}

pub async fn create_user_db_call(State(state): State<AppState>, name: String) -> Result<String> {
    let _ = sqlx::query(&format!("CALL sp_Insert_User(\"{}\")", name))
        .execute(&*state.db_pool)
        .await?;

    Ok("OK".to_string())
}