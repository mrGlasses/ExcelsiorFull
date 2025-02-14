use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::state::AppState;


#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub code: i32,
    pub message_text: String,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            code: Default::default(),
            message_text: Default::default(),
        }
    }
}

pub async fn get_pong(State(_state): State<AppState>) -> Json<Message> {
    let mut msg= Message::default();
    msg.code = 200;
    msg.message_text = "PONG!".to_string();
    Json(msg)
}