use sqlx::{Pool, MySql};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<Pool<MySql>>,
}