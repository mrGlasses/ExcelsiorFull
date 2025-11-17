use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug, Default)]
pub struct Message {
    pub code: i32,
    pub message_text: String,
}


#[derive(Deserialize)]
pub struct Params {
    pub param_1: u32,
    pub param_2: String,
}

#[derive(Deserialize)]
pub struct FilterParams {
    pub name: Option<String>,
    pub age: Option<u32>,
    pub active: Option<bool>,
}
