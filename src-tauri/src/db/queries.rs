/// Common database queries used across the application
use sqlx::SqlitePool;

/// Count pending items in queue
pub async fn count_pending(db: &SqlitePool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'pending'"
    )
    .fetch_one(db)
    .await?;
    Ok(count)
}

/// Count completed items in queue
pub async fn count_completed(db: &SqlitePool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'completed'"
    )
    .fetch_one(db)
    .await?;
    Ok(count)
}

/// Get next pending username from queue
pub async fn get_next_pending(db: &SqlitePool) -> Result<Option<(i64, String)>, sqlx::Error> {
    let row: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, username FROM queue WHERE status = 'pending' ORDER BY created_at LIMIT 1"
    )
    .fetch_optional(db)
    .await?;
    Ok(row)
}

/// Mark queue item as processing
pub async fn mark_processing(db: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE queue SET status = 'processing', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

/// Mark queue item as completed
pub async fn mark_completed(db: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE queue SET status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

/// Mark queue item as failed with error message
pub async fn mark_failed(db: &SqlitePool, id: i64, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE queue SET status = 'failed', error_message = ?, attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(error)
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

/// Add unfollow to history
pub async fn add_to_history(
    db: &SqlitePool,
    username: &str,
    method: &str,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO history (username, method, session_id) VALUES (?, ?, ?)"
    )
    .bind(username)
    .bind(method)
    .bind(session_id)
    .execute(db)
    .await?;
    Ok(())
}

/// Check if username is in history
pub async fn is_in_history(db: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM history WHERE username = ?"
    )
    .bind(username)
    .fetch_one(db)
    .await?;
    Ok(count > 0)
}

/// Clear queue table
pub async fn clear_queue(db: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM queue")
        .execute(db)
        .await?;
    Ok(result.rows_affected())
}
