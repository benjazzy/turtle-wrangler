use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{ConnectOptions, SqlitePool};
use tracing::log::debug;

pub mod turtle_operations;

pub async fn setup_database(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    if !sqlx::Sqlite::database_exists(db_path).await? {
        create_db(db_path).await
    }
    let pool = SqlitePoolOptions::new().connect(db_path).await?;

    debug!("Database initialized");
    Ok(pool)
}

async fn create_db(db_path: &str) {
    debug!("Creating new database");
    let mut connection = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(db_path)
        .connect()
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE turtles (\
        name TEXT PRIMARY KEY, \
        x INTEGER NOT NULL, \
        y INTEGER NOT NULL, \
        z INTEGER NOT NULL, \
        heading TEXT NOT NULL,\
        type TEXT NOT NULL,\
        fuel INTEGER NOT NULL)",
    )
    .execute(&mut connection)
    .await
    .unwrap();
}
