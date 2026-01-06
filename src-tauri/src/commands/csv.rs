use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::fs;
use tauri::State;

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

#[tauri::command]
pub async fn load_csv(
    path: String,
    db: State<'_, SqlitePool>,
) -> Result<LoadResult, String> {
    // Read file contents
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Clear existing queue before importing fresh list
    sqlx::query("DELETE FROM queue")
        .execute(&*db)
        .await
        .map_err(|e| format!("Failed to clear queue: {}", e))?;

    let mut added: u32 = 0;
    let mut skipped: u32 = 0;
    let mut invalid: Vec<String> = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match validate_username(line) {
            Some(username) => {
                // Skip if already unfollowed (in history)
                let in_history: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM history WHERE username = ?"
                )
                .bind(&username)
                .fetch_one(&*db)
                .await
                .unwrap_or((0,));

                if in_history.0 > 0 {
                    skipped += 1;
                    continue;
                }

                // Try to insert into queue
                let result = sqlx::query(
                    "INSERT OR IGNORE INTO queue (username, status) VALUES (?, 'pending')"
                )
                .bind(&username)
                .execute(&*db)
                .await;

                match result {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            added += 1;
                        } else {
                            skipped += 1;
                        }
                    }
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
    let result = sqlx::query("DELETE FROM queue")
        .execute(&*db)
        .await
        .map_err(|e| format!("Failed to clear queue: {}", e))?;

    Ok(result.rows_affected() as u32)
}

#[tauri::command]
pub async fn get_queue_count(db: State<'_, SqlitePool>) -> Result<(u32, u32), String> {
    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'pending'"
    )
    .fetch_one(&*db)
    .await
    .map_err(|e| format!("Failed to count: {}", e))?;

    let completed: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'completed'"
    )
    .fetch_one(&*db)
    .await
    .map_err(|e| format!("Failed to count: {}", e))?;

    Ok((pending.0 as u32, completed.0 as u32))
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
