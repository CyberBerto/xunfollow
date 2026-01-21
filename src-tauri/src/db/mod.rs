pub mod queries;

use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

/// Initialize the database connection pool and run migrations
pub async fn init_database(app_data_dir: &Path) -> Result<sqlx::SqlitePool, Box<dyn std::error::Error>> {
    let db_path = app_data_dir.join("xunfollow.db");

    // Ensure directory exists
    std::fs::create_dir_all(app_data_dir)?;

    // Create connection string
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations
    sqlx::query(include_str!("../../migrations/001_initial.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}
