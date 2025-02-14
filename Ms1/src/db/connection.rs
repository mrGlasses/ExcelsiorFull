use sqlx::{mysql::MySqlPoolOptions, Pool, MySql};

pub async fn init_db() -> Result<Pool<MySql>, sqlx::Error> {
    let database_builder = &format!("mysql://{}:{}@{}:{}/{}",
                                    std::env::var("DATABASE_USER").expect("DATABASE_USER must be set."),
                                    std::env::var("DATABASE_PSWD").expect("DATABASE_PSWD must be set."),
                                    std::env::var("DATABASE_HOST").expect("DATABASE_HOST must be set."),
                                    std::env::var("DATABASE_PORT").expect("DATABASE_PORT must be set."),
                                    std::env::var("DATABASE_NAME").expect("DATABASE_NAME must be set."),
    );

    MySqlPoolOptions::new().connect(&database_builder).await
}

