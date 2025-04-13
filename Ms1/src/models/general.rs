use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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