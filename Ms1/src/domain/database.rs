use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct NewUser {
    pub name: String,
}

#[derive(Serialize, Deserialize, FromRow)]
#[derive(Clone)]
pub struct User {
    pub uid: i32,
    pub name: String,
}