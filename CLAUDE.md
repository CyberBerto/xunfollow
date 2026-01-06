# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Frontend development (Vite dev server)
npm run dev

# Build frontend (TypeScript check + Vite build)
npm run build

# Run Tauri desktop app in development
npm run tauri dev

# Build Tauri app for production
npm run tauri build

# Build Rust backend only (from src-tauri/)
cd src-tauri && cargo build

# Check Rust code without building
cd src-tauri && cargo check
```

## Architecture

XUnfollow is a Tauri 2 desktop application for batch unfollowing X/Twitter accounts. It uses:
- **Frontend**: React 19 + TypeScript + Vite (port 1420)
- **Backend**: Rust with Tauri 2, SQLx for SQLite database

### Frontend (`src/`)
Single-page React app in `App.tsx`. Communicates with Rust backend via:
- `invoke()` for commands (from `@tauri-apps/api/core`)
- `listen()` for events (from `@tauri-apps/api/event`)

### Backend (`src-tauri/`)
- **Entry**: `main.rs` calls `xunfollow_lib::run()` from `lib.rs`
- **Commands** (`src/commands/`): Tauri commands exposed to frontend
  - `csv.rs`: CSV import and queue management
  - `unfollow.rs`: Webview creation and unfollow loop
  - `settings.rs`: App settings CRUD
- **Services** (`src/services/`): Business logic
  - `rate_limiter.rs`: Daily/hourly/session rate limiting
  - `state.rs`: Persistent app state

### Data Flow
1. User uploads CSV of usernames → `load_csv` command validates and stores in `queue` table
2. `start_unfollow` spawns async loop that:
   - Checks rate limits (daily/hourly/session)
   - Opens Twitter webview, navigates to profile
   - Injects `scripts/unfollow_profile.js` to click unfollow
   - Records results in `history` table
3. Progress events emitted to frontend via `app.emit()`

### Database
SQLite database at app data directory. Schema in `migrations/001_initial.sql`:
- `queue`: Usernames to unfollow (status: pending/processing/completed/failed)
- `history`: Completed unfollows
- `rate_limits`: Persisted rate limit counters
- `settings`: User preferences
- `app_state`: Resume state

### State Management
Three managed state objects in Tauri:
- `SqlitePool`: Database connection pool
- `Arc<Mutex<UnfollowState>>`: Current operation state
- `Arc<Mutex<RateLimiter>>`: Rate limiting state

### JS Injection Scripts (`src-tauri/scripts/`)
- `unfollow_profile.js`: CSS + text fallback strategy for clicking unfollow
- `check_login.js`: Login detection
- `selectors.json`: Configurable DOM selectors

## Build Outputs

- **macOS App**: `src-tauri/target/release/bundle/macos/XUnfollow.app`
- **macOS DMG**: `src-tauri/target/release/bundle/dmg/XUnfollow_0.1.0_aarch64.dmg`

## Testing

1. Run `npm run tauri dev`
2. Upload a CSV with Twitter usernames (one per line, with or without @)
3. Click Start - opens X.com webview
4. Log into X.com in the webview
5. App iterates through usernames, visiting profiles and clicking unfollow
