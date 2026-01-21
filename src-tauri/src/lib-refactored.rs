mod commands;
mod services;
mod constants;
mod errors;
mod db;

use commands::{
    clear_queue, create_twitter_webview, export_history, export_queue_csv, fetch_following_list,
    get_history, get_progress, get_queue, get_queue_count, get_settings, hide_twitter_webview,
    load_csv, pause_unfollow, resize_twitter_webview, resume_unfollow, show_twitter_webview,
    start_unfollow, stop_unfollow, update_settings, UnfollowState,
};
use services::RateLimiter;
use db::init_database;
use constants::*;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

/// Load settings from database
async fn load_settings_from_db(pool: &sqlx::SqlitePool) -> commands::Settings {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, value FROM settings"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut s = commands::Settings::default();
    for (key, value) in rows {
        match key.as_str() {
            "daily_limit" => s.daily_limit = value.parse().unwrap_or(DEFAULT_DAILY_LIMIT),
            "hourly_limit" => s.hourly_limit = value.parse().unwrap_or(DEFAULT_HOURLY_LIMIT),
            "session_limit" => s.session_limit = value.parse().unwrap_or(DEFAULT_SESSION_LIMIT),
            "min_delay" => s.min_delay = value.parse().unwrap_or(DEFAULT_MIN_DELAY),
            "max_delay" => s.max_delay = value.parse().unwrap_or(DEFAULT_MAX_DELAY),
            _ => {}
        }
    }
    s
}

/// Clear queue on app startup
async fn clear_queue_on_startup(pool: &sqlx::SqlitePool) {
    let _ = sqlx::query("DELETE FROM queue")
        .execute(pool)
        .await;
}

/// Initialize rate limiter from database
async fn init_rate_limiter(pool: &sqlx::SqlitePool, settings: &commands::Settings) -> RateLimiter {
    RateLimiter::load_from_database(pool, settings).await
        .unwrap_or_else(|_| RateLimiter::new(
            settings.daily_limit,
            settings.hourly_limit,
            settings.session_limit,
            settings.min_delay,
            settings.max_delay,
        ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize database (blocking in setup)
            let pool = tauri::async_runtime::block_on(async {
                init_database(&app_data_dir).await
            }).expect("Failed to initialize database");

            // Clear queue on fresh app start
            tauri::async_runtime::block_on(async {
                clear_queue_on_startup(&pool).await;
            });

            // Load settings for rate limiter
            let settings = tauri::async_runtime::block_on(async {
                load_settings_from_db(&pool).await
            });

            // Initialize rate limiter from database
            let rate_limiter = tauri::async_runtime::block_on(async {
                init_rate_limiter(&pool, &settings).await
            });

            // Manage state
            app.manage(pool);
            app.manage(Arc::new(Mutex::new(UnfollowState::default())));
            app.manage(Arc::new(Mutex::new(rate_limiter)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // CSV commands
            load_csv,
            get_queue,
            clear_queue,
            get_queue_count,
            export_queue_csv,
            fetch_following_list,
            // Unfollow commands
            create_twitter_webview,
            resize_twitter_webview,
            hide_twitter_webview,
            show_twitter_webview,
            start_unfollow,
            pause_unfollow,
            resume_unfollow,
            stop_unfollow,
            get_progress,
            // Settings commands
            get_settings,
            update_settings,
            get_history,
            export_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
