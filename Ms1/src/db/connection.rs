use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub async fn init_db() -> Result<Pool<MySql>, sqlx::Error> {
    let database_builder = &format!(
        "mysql://{}:{}@{}:{}/{}",
        std::env::var("DATABASE_USER").expect("DATABASE_USER must be set."),
        std::env::var("DATABASE_PSWD").expect("DATABASE_PSWD must be set."),
        std::env::var("DATABASE_HOST").expect("DATABASE_HOST must be set."),
        std::env::var("DATABASE_PORT").expect("DATABASE_PORT must be set."),
        std::env::var("DATABASE_NAME").expect("DATABASE_NAME must be set."),
    );

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_builder)
        .await
}

/*################################################################################################

        ----------------------------------------TESTS-----------------------------------

#################################################################################################*/

// Note: Unit testing database connections are generally not recommended because:
// 1. It requires a real database connection which makes it an integration test, not unit test
// 2. It depends on external resources and environment variables
// 3. It can be flaky and slow down the test suite
//
// Instead, consider:
// - Writing integration tests in a separate directory
// - Using a test database or Docker container for integration testing
// - Mocking the database connection for unit tests of components that use this connection
// - Testing the connection in your deployment pipeline


