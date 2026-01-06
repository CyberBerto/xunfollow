# XUnfollow

A desktop app for batch unfollowing accounts on X (Twitter).

## Features

- **CSV Import** - Upload a list of usernames to unfollow
- **Fetch Following** - Auto-scrape your following list from X.com
- **Rate Limiting** - Configurable daily/hourly/session limits
- **History Tracking** - Skips previously unfollowed accounts
- **Export** - Save queue or history to CSV

## Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2
- **Database**: SQLite

## Development

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

## Usage

1. Launch the app
2. Click "Open X.com" and log into your account
3. Either:
   - Upload a CSV file with usernames (one per line)
   - Click "Fetch Following List" to auto-import
4. Click "Start" to begin unfollowing
5. Monitor progress in the activity log

## Settings

- **Daily Limit**: Max unfollows per day (default: 50)
- **Hourly Limit**: Max unfollows per hour (default: 30)
- **Session Limit**: Unfollows before break (default: 25)
- **Delay**: Random delay between actions (30-60 seconds)

## License

MIT
