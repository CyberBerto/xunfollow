-- XUnfollow Database Schema

CREATE TABLE IF NOT EXISTS queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    attempts INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    unfollowed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    method TEXT NOT NULL,
    session_id TEXT
);

CREATE TABLE IF NOT EXISTS rate_limits (
    key TEXT PRIMARY KEY,
    value INTEGER NOT NULL,
    reset_at DATETIME
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_queue_status ON queue(status);
CREATE INDEX IF NOT EXISTS idx_history_date ON history(unfollowed_at);

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value) VALUES ('daily_limit', '50');
INSERT OR IGNORE INTO settings (key, value) VALUES ('hourly_limit', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('session_limit', '25');
INSERT OR IGNORE INTO settings (key, value) VALUES ('min_delay', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('max_delay', '60');
