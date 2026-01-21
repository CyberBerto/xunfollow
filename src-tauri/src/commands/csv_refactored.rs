use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::fs;
use tauri::State;

use crate::db::queries;
use crate::errors::AppError;

#[derive(Debug, Serialize)]
pub struct LoadResult {
    pub added: u32,
    pub skipped: u32,
    pub invalid: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QueueItem {
    pub id: i64,
    pub username: String,
    pub status: String,
    pub error_message: Option<String>,
    pub attempts: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Validate and normalize a Twitter username
///
/// Returns the normalized username (lowercase, without @) if valid, None otherwise
fn validate_username(username: &str) -> Option<String> {
    // Strip @ prefix if present
    let cleaned = username.trim().trim_start_matches('@');

    if cleaned.is_empty() {
        return None;
    }

    // X usernames: 1-15 characters, alphanumeric and underscore only
    let re = Regex::new(r"^[a-zA-Z0-9_]{1,15}$").unwrap();

    if re.is_match(cleaned) {
        Some(cleaned.to_lowercase())
    } else {
        None
    }
}

/// Insert username into queue if not already in history
async fn insert_username(
    db: &SqlitePool,
    username: &str,
) -> Result<bool, AppError> {
    // Skip if already unfollowed (in history)
    if queries::is_in_history(db, username).await? {
        return Ok(false);
    }

    // Try to insert into queue
    let result = sqlx::query(
        "INSERT OR IGNORE INTO queue (username, status) VALUES (?, 'pending')"
    )
    .bind(username)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[tauri::command]
pub async fn load_csv(
    path: String,
    db: State<'_, SqlitePool>,
) -> Result<LoadResult, String> {
    // Read file contents
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Clear existing queue before importing fresh list
    queries::clear_queue(&*db)
        .await
        .map_err(|e| format!("Failed to clear queue: {}", e))?;

    let mut added: u32 = 0;
    let mut skipped: u32 = 0;
    let mut invalid: Vec<String> = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match validate_username(line) {
            Some(username) => {
                match insert_username(&*db, &username).await {
                    Ok(true) => added += 1,
                    Ok(false) => skipped += 1,
                    Err(_) => skipped += 1,
                }
            }
            None => {
                invalid.push(line.to_string());
            }
        }
    }

    Ok(LoadResult { added, skipped, invalid })
}

#[tauri::command]
pub async fn get_queue(db: State<'_, SqlitePool>) -> Result<Vec<QueueItem>, String> {
    let items: Vec<QueueItem> = sqlx::query_as(
        "SELECT id, username, status, error_message, attempts,
         datetime(created_at) as created_at, datetime(updated_at) as updated_at
         FROM queue ORDER BY created_at"
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| format!("Failed to fetch queue: {}", e))?;

    Ok(items)
}

#[tauri::command]
pub async fn clear_queue(db: State<'_, SqlitePool>) -> Result<u32, String> {
    let count = queries::clear_queue(&*db)
        .await
        .map_err(|e| format!("Failed to clear queue: {}", e))?;

    Ok(count as u32)
}

#[tauri::command]
pub async fn get_queue_count(db: State<'_, SqlitePool>) -> Result<(u32, u32), String> {
    let pending = queries::count_pending(&*db)
        .await
        .map_err(|e| format!("Failed to count: {}", e))?;

    let completed = queries::count_completed(&*db)
        .await
        .map_err(|e| format!("Failed to count: {}", e))?;

    Ok((pending as u32, completed as u32))
}

#[tauri::command]
pub async fn export_queue_csv(
    path: String,
    db: State<'_, SqlitePool>,
) -> Result<u32, String> {
    let usernames: Vec<(String,)> = sqlx::query_as(
        "SELECT username FROM queue WHERE status = 'pending' ORDER BY created_at"
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| format!("Failed to fetch queue: {}", e))?;

    let content = usernames.iter()
        .map(|(u,)| u.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(usernames.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert_eq!(validate_username("@username"), Some("username".to_string()));
        assert_eq!(validate_username("username"), Some("username".to_string()));
        assert_eq!(validate_username("User_123"), Some("user_123".to_string()));
        assert_eq!(validate_username(""), None);
        assert_eq!(validate_username("user@name"), None);
        assert_eq!(validate_username("this_is_too_long_username"), None);
    }
}
