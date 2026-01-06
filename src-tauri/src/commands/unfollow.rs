use crate::services::{AppState, LimitStatus, RateLimiter};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl};
use tokio::sync::Mutex;
use uuid::Uuid;

// Include the unfollow script at compile time
const UNFOLLOW_SCRIPT: &str = include_str!("../../scripts/unfollow_profile.js");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfollowState {
    pub running: bool,
    pub paused: bool,
    pub current_position: u32,
    pub total: u32,
    pub session_id: String,
}

impl Default for UnfollowState {
    fn default() -> Self {
        UnfollowState {
            running: false,
            paused: false,
            current_position: 0,
            total: 0,
            session_id: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Progress {
    pub current: u32,
    pub total: u32,
    pub percentage: f32,
    pub eta_seconds: Option<u64>,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UnfollowEvent {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProgressEvent {
    pub current: u32,
    pub total: u32,
    pub username: String,
    pub eta: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RateLimitEvent {
    pub reason: String,
    pub resume_at: Option<String>,
}

#[tauri::command]
pub async fn create_twitter_webview(app: tauri::AppHandle) -> Result<(), String> {
    // Check if webview already exists
    if app.get_webview("twitter").is_some() {
        return Ok(());
    }

    // Get the main window (need Window, not WebviewWindow, to add child webview)
    let main_window = app.get_window("main")
        .ok_or("Main window not found")?;

    // Get scale factor to convert physical to logical pixels
    let scale_factor = main_window.scale_factor()
        .map_err(|e| format!("Failed to get scale factor: {}", e))?;

    // Get window size in physical pixels and convert to logical
    let physical_size = main_window.inner_size()
        .map_err(|e| format!("Failed to get window size: {}", e))?;

    let logical_width = physical_size.width as f64 / scale_factor;
    let logical_height = physical_size.height as f64 / scale_factor;

    // Control panel is 350px wide, webview fills the rest
    let webview_width = (logical_width - 350.0).max(100.0);

    // Create embedded webview (x=350 for right panel, full height)
    let webview_builder = tauri::webview::WebviewBuilder::new(
        "twitter",
        WebviewUrl::External("https://x.com".parse().unwrap()),
    );

    // Add webview as child of main window (350px from left for control panel)
    main_window.add_child(
        webview_builder,
        LogicalPosition::new(350.0, 0.0),
        LogicalSize::new(webview_width, logical_height),
    ).map_err(|e| format!("Failed to create webview: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn start_unfollow(
    app: tauri::AppHandle,
    db: State<'_, SqlitePool>,
    unfollow_state: State<'_, Arc<Mutex<UnfollowState>>>,
    rate_limiter: State<'_, Arc<Mutex<RateLimiter>>>,
) -> Result<(), String> {
    // Get the counts
    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'pending'"
    )
    .fetch_one(&*db)
    .await
    .map_err(|e| format!("Failed to count pending: {}", e))?;

    if pending.0 == 0 {
        return Err("No pending usernames in queue".to_string());
    }

    // Update state
    {
        let mut state = unfollow_state.lock().await;
        state.running = true;
        state.paused = false;
        state.session_id = Uuid::new_v4().to_string();
        state.total = pending.0 as u32;
        state.current_position = 0;
    }

    // Clone what we need for the spawned task
    let db_pool = (*db).clone();
    let state_clone = Arc::clone(&unfollow_state);
    let limiter_clone = Arc::clone(&rate_limiter);
    let app_clone = app.clone();

    // Spawn the unfollow loop
    tokio::spawn(async move {
        unfollow_loop(app_clone, db_pool, state_clone, limiter_clone).await;
    });

    Ok(())
}

async fn unfollow_loop(
    app: tauri::AppHandle,
    db: SqlitePool,
    state: Arc<Mutex<UnfollowState>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
) {
    loop {
        // Check if we should continue
        let (should_continue, session_id, _position) = {
            let s = state.lock().await;
            (s.running && !s.paused, s.session_id.clone(), s.current_position)
        };

        if !should_continue {
            break;
        }

        // Check rate limits
        let limit_status = {
            let mut limiter = rate_limiter.lock().await;
            limiter.check_limits()
        };

        match limit_status {
            LimitStatus::DailyStop => {
                let _ = app.emit("unfollow:rate_limited", RateLimitEvent {
                    reason: "daily_limit".to_string(),
                    resume_at: Some("tomorrow".to_string()),
                });
                let mut s = state.lock().await;
                s.running = false;
                break;
            }
            LimitStatus::HourlyWait(minutes) => {
                let _ = app.emit("unfollow:rate_limited", RateLimitEvent {
                    reason: "hourly_limit".to_string(),
                    resume_at: Some(format!("{} minutes", minutes)),
                });
                // Wait for the specified minutes
                tokio::time::sleep(tokio::time::Duration::from_secs(minutes as u64 * 60)).await;
                continue;
            }
            LimitStatus::SessionBreak => {
                let _ = app.emit("unfollow:rate_limited", RateLimitEvent {
                    reason: "session_break".to_string(),
                    resume_at: Some("30 minutes".to_string()),
                });
                // Take a 30-minute break
                tokio::time::sleep(tokio::time::Duration::from_secs(30 * 60)).await;
                {
                    let mut limiter = rate_limiter.lock().await;
                    limiter.reset_session();
                }
                continue;
            }
            LimitStatus::Ok => {}
        }

        // Get next pending username
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, username FROM queue WHERE status = 'pending' ORDER BY created_at LIMIT 1"
        )
        .fetch_optional(&db)
        .await
        .unwrap_or(None);

        let (id, username) = match row {
            Some(r) => r,
            None => {
                // No more pending users
                let total = {
                    let s = state.lock().await;
                    s.current_position
                };
                let _ = app.emit("unfollow:completed", serde_json::json!({
                    "total_unfollowed": total
                }));
                let mut s = state.lock().await;
                s.running = false;
                break;
            }
        };

        // Mark as processing
        let _ = sqlx::query(
            "UPDATE queue SET status = 'processing', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(id)
        .execute(&db)
        .await;

        // Navigate to user's profile and unfollow
        let result = execute_unfollow(&app, &username).await;

        match result {
            Ok(method) => {
                // Mark as completed
                let _ = sqlx::query(
                    "UPDATE queue SET status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                )
                .bind(id)
                .execute(&db)
                .await;

                // Add to history
                let _ = sqlx::query(
                    "INSERT INTO history (username, method, session_id) VALUES (?, ?, ?)"
                )
                .bind(&username)
                .bind(&method)
                .bind(&session_id)
                .execute(&db)
                .await;

                // Increment rate limiter
                {
                    let mut limiter = rate_limiter.lock().await;
                    limiter.increment();
                    let _ = limiter.save_to_database(&db).await;
                }

                // Emit success event
                let _ = app.emit("unfollow:success", UnfollowEvent {
                    username: username.clone(),
                    method: Some(method),
                    error: None,
                });
            }
            Err(error) => {
                // Mark as failed
                let _ = sqlx::query(
                    "UPDATE queue SET status = 'failed', error_message = ?, attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                )
                .bind(&error)
                .bind(id)
                .execute(&db)
                .await;

                // Emit failed event
                let _ = app.emit("unfollow:failed", UnfollowEvent {
                    username: username.clone(),
                    method: None,
                    error: Some(error),
                });
            }
        }

        // Update position and emit progress
        let (current, total) = {
            let mut s = state.lock().await;
            s.current_position += 1;
            (s.current_position, s.total)
        };

        // Save state to database
        {
            let s = state.lock().await;
            let app_state = AppState {
                current_position: s.current_position,
                is_running: s.running,
                is_paused: s.paused,
                last_session_id: s.session_id.clone(),
            };
            let _ = app_state.save(&db).await;
        }

        // Calculate ETA (assuming average delay of 45 seconds)
        let remaining = total.saturating_sub(current);
        let eta_seconds = remaining as u64 * 45;
        let eta = if eta_seconds > 3600 {
            format!("{}h {}m", eta_seconds / 3600, (eta_seconds % 3600) / 60)
        } else if eta_seconds > 60 {
            format!("{}m", eta_seconds / 60)
        } else {
            format!("{}s", eta_seconds)
        };

        let _ = app.emit("unfollow:progress", ProgressEvent {
            current,
            total,
            username,
            eta: Some(eta),
        });

        // Wait random delay
        let delay = {
            let limiter = rate_limiter.lock().await;
            limiter.get_random_delay()
        };
        tokio::time::sleep(delay).await;
    }
}

async fn execute_unfollow(app: &tauri::AppHandle, username: &str) -> Result<String, String> {
    let webview = app.get_webview("twitter")
        .ok_or("Twitter webview not found. Please open X.com first.")?;

    // Navigate to user's profile
    let nav_script = format!("window.location.href = 'https://x.com/{}'", username);
    webview.eval(&nav_script)
        .map_err(|e| format!("Failed to navigate: {}", e))?;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Inject the unfollow script with the username
    let script = UNFOLLOW_SCRIPT.replace("__USERNAME__", username);
    webview.eval(&script)
        .map_err(|e| format!("Failed to inject script: {}", e))?;

    // Wait for script to execute
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // For now, assume success. In a full implementation, we'd use events
    // to get the result back from the injected script.
    Ok("css".to_string())
}

#[tauri::command]
pub async fn pause_unfollow(
    app: tauri::AppHandle,
    db: State<'_, SqlitePool>,
    unfollow_state: State<'_, Arc<Mutex<UnfollowState>>>,
) -> Result<(), String> {
    let mut state = unfollow_state.lock().await;
    state.paused = true;

    // Save state
    let app_state = AppState {
        current_position: state.current_position,
        is_running: state.running,
        is_paused: state.paused,
        last_session_id: state.session_id.clone(),
    };
    app_state.save(&*db).await
        .map_err(|e| format!("Failed to save state: {}", e))?;

    let _ = app.emit("unfollow:paused", serde_json::json!({
        "position": state.current_position,
        "reason": "user_requested"
    }));

    Ok(())
}

#[tauri::command]
pub async fn resume_unfollow(
    app: tauri::AppHandle,
    db: State<'_, SqlitePool>,
    unfollow_state: State<'_, Arc<Mutex<UnfollowState>>>,
    rate_limiter: State<'_, Arc<Mutex<RateLimiter>>>,
) -> Result<(), String> {
    // Load state from database
    let saved_state = AppState::load(&*db).await
        .map_err(|e| format!("Failed to load state: {}", e))?;

    // Get current pending count
    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM queue WHERE status = 'pending'"
    )
    .fetch_one(&*db)
    .await
    .map_err(|e| format!("Failed to count pending: {}", e))?;

    if pending.0 == 0 {
        return Err("No pending usernames in queue".to_string());
    }

    // Update state
    {
        let mut state = unfollow_state.lock().await;
        state.running = true;
        state.paused = false;
        state.current_position = saved_state.current_position;
        state.total = saved_state.current_position + pending.0 as u32;
        if saved_state.last_session_id.is_empty() {
            state.session_id = Uuid::new_v4().to_string();
        } else {
            state.session_id = saved_state.last_session_id;
        }
    }

    // Clone for spawned task
    let db_pool = (*db).clone();
    let state_clone = Arc::clone(&unfollow_state);
    let limiter_clone = Arc::clone(&rate_limiter);
    let app_clone = app.clone();

    // Spawn the unfollow loop
    tokio::spawn(async move {
        unfollow_loop(app_clone, db_pool, state_clone, limiter_clone).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_unfollow(
    unfollow_state: State<'_, Arc<Mutex<UnfollowState>>>,
) -> Result<(), String> {
    let mut state = unfollow_state.lock().await;
    state.running = false;
    state.paused = false;
    Ok(())
}

#[tauri::command]
pub async fn get_progress(
    unfollow_state: State<'_, Arc<Mutex<UnfollowState>>>,
) -> Result<Progress, String> {
    let state = unfollow_state.lock().await;

    let percentage = if state.total > 0 {
        (state.current_position as f32 / state.total as f32) * 100.0
    } else {
        0.0
    };

    let remaining = state.total.saturating_sub(state.current_position);
    let eta_seconds = remaining as u64 * 45; // Assuming 45 sec average

    let status = if state.running && !state.paused {
        "running"
    } else if state.paused {
        "paused"
    } else if state.current_position >= state.total && state.total > 0 {
        "completed"
    } else {
        "idle"
    };

    Ok(Progress {
        current: state.current_position,
        total: state.total,
        percentage,
        eta_seconds: if remaining > 0 { Some(eta_seconds) } else { None },
        status: status.to_string(),
    })
}

#[tauri::command]
pub async fn resize_twitter_webview(app: tauri::AppHandle) -> Result<(), String> {
    let webview = match app.get_webview("twitter") {
        Some(w) => w,
        None => return Ok(()), // No webview to resize
    };

    let main_window = app.get_window("main")
        .ok_or("Main window not found")?;

    // Get scale factor to convert physical to logical pixels
    let scale_factor = main_window.scale_factor()
        .map_err(|e| format!("Failed to get scale factor: {}", e))?;

    let physical_size = main_window.inner_size()
        .map_err(|e| format!("Failed to get window size: {}", e))?;

    let logical_width = physical_size.width as f64 / scale_factor;
    let logical_height = physical_size.height as f64 / scale_factor;
    let webview_width = (logical_width - 350.0).max(100.0);

    // Resize webview to fill right panel (350px from left)
    webview.set_position(LogicalPosition::new(350.0, 0.0))
        .map_err(|e| format!("Failed to set position: {}", e))?;
    webview.set_size(LogicalSize::new(webview_width, logical_height))
        .map_err(|e| format!("Failed to set size: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn hide_twitter_webview(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview("twitter") {
        // Move webview off-screen to hide it
        webview.set_position(LogicalPosition::new(-10000.0, 0.0))
            .map_err(|e| format!("Failed to hide webview: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_twitter_webview(app: tauri::AppHandle) -> Result<(), String> {
    // Just call resize to restore proper position
    resize_twitter_webview(app).await
}

#[derive(Debug, Serialize)]
pub struct FetchFollowingResult {
    pub added: u32,
    pub skipped: u32,
}

#[tauri::command]
pub async fn fetch_following_list(
    app: tauri::AppHandle,
    db: State<'_, SqlitePool>,
) -> Result<FetchFollowingResult, String> {
    use regex::Regex;

    let webview = app.get_webview("twitter")
        .ok_or("Twitter webview not found. Please open X.com first.")?;

    // Navigate to the following page
    webview.eval(r#"
        (function() {
            const link = document.querySelector('a[data-testid="AppTabBar_Profile_Link"]');
            if (link) window.location.href = 'https://x.com' + link.getAttribute('href') + '/following';
        })()
    "#).map_err(|e| format!("Failed to navigate: {}", e))?;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Scrape script: collects usernames, scrolls, signals completion via URL hash
    // Only captures the first profile link per UserCell (ignores @mentions in bios)
    webview.eval(r#"
        (async () => {
            const users = new Set();
            let idle = 0;
            let lastScrollTop = -1;
            const excluded = ['home','explore','search','notifications','messages','settings','i','compose','lists'];

            while (idle < 3) {
                document.querySelectorAll('[data-testid="UserCell"]').forEach(cell => {
                    // Only get the FIRST valid profile link (avatar/name), skip bio @mentions
                    const links = cell.querySelectorAll('a[href^="/"]');
                    for (const a of links) {
                        const href = a.getAttribute('href');
                        const m = href && href.match(/^\/([a-zA-Z0-9_]{1,15})$/);
                        if (m && !excluded.includes(m[1].toLowerCase())) {
                            users.add(m[1].toLowerCase());
                            break;
                        }
                    }
                });

                const cells = document.querySelectorAll('[data-testid="UserCell"]');
                if (cells.length) cells[cells.length - 1].scrollIntoView({ behavior: 'smooth' });

                await new Promise(r => setTimeout(r, 1500 + Math.random() * 1000));

                // Check if we're stuck (no new users AND scroll position unchanged)
                const currentScroll = window.scrollY || document.documentElement.scrollTop;
                const noNewUsers = users.size === (window.__lastUserCount || 0);
                const scrollStuck = Math.abs(currentScroll - lastScrollTop) < 50;

                if (noNewUsers && scrollStuck) {
                    idle++;
                } else {
                    idle = 0;
                }

                window.__lastUserCount = users.size;
                lastScrollTop = currentScroll;
            }

            // Scroll back to top
            window.scrollTo({ top: 0, behavior: 'smooth' });

            // Signal completion via URL hash (Rust polls webview.url())
            window.location.hash = '__XUNFOLLOW_DONE__:' + [...users].join(',');
        })()
    "#).map_err(|e| format!("Failed to start scrape: {}", e))?;

    // Poll for completion via URL hash (max 2 minutes)
    let mut data = String::new();
    for _ in 0..240 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Ok(url) = webview.url() {
            let url_str = url.as_str();
            if let Some(hash_pos) = url_str.find("#__XUNFOLLOW_DONE__:") {
                data = url_str[hash_pos + 20..].to_string();
                // Clear the hash
                let _ = webview.eval("window.location.hash = ''");
                break;
            }
        }
    }

    if data.is_empty() {
        return Err("No accounts found. Make sure you're logged in and on the following page.".to_string());
    }

    // Process usernames into queue
    let usernames: Vec<&str> = data.split(',').filter(|s| !s.is_empty()).collect();
    let username_regex = Regex::new(r"^[a-zA-Z0-9_]{1,15}$").unwrap();

    // Clear existing queue before inserting fresh list
    sqlx::query("DELETE FROM queue")
        .execute(&*db)
        .await
        .map_err(|e| format!("Failed to clear queue: {}", e))?;

    let mut added: u32 = 0;
    let mut skipped: u32 = 0;

    for username in usernames {
        let username_str = username.trim().to_lowercase();
        if !username_regex.is_match(&username_str) {
            continue;
        }

        let result = sqlx::query(
            "INSERT OR IGNORE INTO queue (username, status) VALUES (?, 'pending')"
        )
        .bind(&username_str)
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

    Ok(FetchFollowingResult { added, skipped })
}
