use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Debug)]
pub struct NewUser {
    pub name: String,
}

#[derive(Serialize, Deserialize, FromRow, Clone, Debug)]
pub struct User {
    pub uid: i32,
    pub name: String,
}
