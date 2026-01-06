use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::fs::File;
use std::io::Write;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub daily_limit: u32,
    pub hourly_limit: u32,
    pub session_limit: u32,
    pub min_delay: u32,
    pub max_delay: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            daily_limit: 50,
            hourly_limit: 30,
            session_limit: 25,
            min_delay: 30,
            max_delay: 60,
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct HistoryItem {
    pub id: i64,
    pub username: String,
    pub unfollowed_at: String,
    pub method: String,
    pub session_id: Option<String>,
}

#[tauri::command]
pub async fn get_settings(db: State<'_, SqlitePool>) -> Result<Settings, String> {
    let mut settings = Settings::default();

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, value FROM settings"
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| format!("Failed to fetch settings: {}", e))?;

    for (key, value) in rows {
        match key.as_str() {
            "daily_limit" => settings.daily_limit = value.parse().unwrap_or(50),
            "hourly_limit" => settings.hourly_limit = value.parse().unwrap_or(30),
            "session_limit" => settings.session_limit = value.parse().unwrap_or(25),
            "min_delay" => settings.min_delay = value.parse().unwrap_or(30),
            "max_delay" => settings.max_delay = value.parse().unwrap_or(60),
            _ => {}
        }
    }

    Ok(settings)
}

#[tauri::command]
pub async fn update_settings(
    settings: Settings,
    db: State<'_, SqlitePool>,
) -> Result<(), String> {
    // Validate settings
    if settings.daily_limit > 100 {
        return Err("Daily limit cannot exceed 100".to_string());
    }
    if settings.min_delay > settings.max_delay {
        return Err("Min delay cannot be greater than max delay".to_string());
    }

    // Update each setting
    let pairs = [
        ("daily_limit", settings.daily_limit.to_string()),
        ("hourly_limit", settings.hourly_limit.to_string()),
        ("session_limit", settings.session_limit.to_string()),
        ("min_delay", settings.min_delay.to_string()),
        ("max_delay", settings.max_delay.to_string()),
    ];

    for (key, value) in pairs {
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)"
        )
        .bind(key)
        .bind(value)
        .execute(&*db)
        .await
        .map_err(|e| format!("Failed to update setting {}: {}", key, e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_history(
    limit: i32,
    db: State<'_, SqlitePool>,
) -> Result<Vec<HistoryItem>, String> {
    let items: Vec<HistoryItem> = sqlx::query_as(
        "SELECT id, username, datetime(unfollowed_at) as unfollowed_at, method, session_id
         FROM history
         ORDER BY unfollowed_at DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&*db)
    .await
    .map_err(|e| format!("Failed to fetch history: {}", e))?;

    Ok(items)
}

#[tauri::command]
pub async fn export_history(
    path: String,
    db: State<'_, SqlitePool>,
) -> Result<u32, String> {
    let items: Vec<HistoryItem> = sqlx::query_as(
        "SELECT id, username, datetime(unfollowed_at) as unfollowed_at, method, session_id
         FROM history
         ORDER BY unfollowed_at"
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| format!("Failed to fetch history: {}", e))?;

    let mut file = File::create(&path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // Write CSV header
    writeln!(file, "username,unfollowed_at,method")
        .map_err(|e| format!("Failed to write header: {}", e))?;

    // Write rows
    for item in &items {
        writeln!(file, "{},{},{}", item.username, item.unfollowed_at, item.method)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }

    Ok(items.len() as u32)
}
