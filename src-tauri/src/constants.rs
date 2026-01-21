/// Configuration constants for the unfollow process

/// Time to wait for page navigation (in seconds)
pub const PAGE_LOAD_WAIT_SECS: u64 = 3;

/// Time to wait for script execution (in seconds)
pub const SCRIPT_EXECUTION_WAIT_SECS: u64 = 3;

/// Default break time after session limit (in minutes)
pub const SESSION_BREAK_MINUTES: u64 = 30;

/// Average delay for ETA calculations (in seconds)
pub const AVERAGE_DELAY_SECS: u64 = 45;

/// Maximum logs to keep in memory
pub const MAX_LOG_ENTRIES: usize = 100;

/// Width of the control panel (in pixels)
pub const CONTROL_PANEL_WIDTH: f64 = 350.0;

/// Minimum webview width (in pixels)
pub const MIN_WEBVIEW_WIDTH: f64 = 100.0;

/// Default database settings
pub const DEFAULT_DAILY_LIMIT: u32 = 50;
pub const DEFAULT_HOURLY_LIMIT: u32 = 30;
pub const DEFAULT_SESSION_LIMIT: u32 = 25;
pub const DEFAULT_MIN_DELAY: u32 = 30;
pub const DEFAULT_MAX_DELAY: u32 = 60;

/// Maximum allowed daily limit
pub const MAX_DAILY_LIMIT: u32 = 100;
