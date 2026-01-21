# X Unfollow Helper - Chrome Extension

A Chrome extension to batch unfollow users on X (Twitter) with smart rate limiting and easy controls.

## 🎯 Why Chrome Extension > Tauri

| Feature | Tauri (Old) | Chrome Extension (New) |
|---------|-------------|------------------------|
| Setup | Complex (Rust + GTK) | Simple (just load extension) |
| Development | Hard to debug | Easy (Chrome DevTools) |
| Distribution | Manual builds | Chrome Web Store |
| Cross-platform | Needs compilation | Works everywhere |
| Testing | Complicated | Instant reload |
| Updates | Manual | Automatic |

## ✨ Features

- ✅ Batch unfollow users on X.com
- ✅ Smart rate limiting (daily/hourly/session)
- ✅ Random delays to appear human
- ✅ Pause/resume functionality
- ✅ Activity log
- ✅ Export history to CSV
- ✅ Clean, modern UI
- ✅ No external dependencies

## 🚀 Installation

### Option 1: Load Unpacked (Development)

1. **Clone this repository**
   ```bash
   git clone https://github.com/CyberBerto/xunfollow.git
   cd xunfollow/extension-chrome
   ```

2. **Add icons** (optional, see `icons/README.md`)
   ```bash
   cd icons
   # Add your icon16.png, icon48.png, icon128.png
   # Or use placeholders for testing
   ```

3. **Load in Chrome**
   - Open Chrome and go to `chrome://extensions/`
   - Enable "Developer mode" (toggle in top right)
   - Click "Load unpacked"
   - Select the `extension-chrome` folder
   - Done! Extension is installed 🎉

### Option 2: Chrome Web Store (Coming Soon)

Once published, you'll be able to install with one click from the Chrome Web Store.

## 📖 How to Use

### Step 1: Navigate to X.com

Go to https://x.com or https://twitter.com and log in to your account.

### Step 2: Go to Your Following List

Navigate to your profile and click on "Following" to see the list of users you follow.

### Step 3: Open the Extension

Click the extension icon in your Chrome toolbar (top right).

### Step 4: Configure Settings (Optional)

Click the ⚙️ gear icon to adjust:
- **Daily Limit**: Maximum unfollows per day (default: 50)
- **Hourly Limit**: Maximum unfollows per hour (default: 30)
- **Session Limit**: Maximum unfollows per session (default: 25)
- **Min/Max Delay**: Random delay range between unfollows (default: 30-60 seconds)

### Step 5: Start Unfollowing

1. Click the "▶ Start" button
2. The extension will automatically:
   - Find "Following" buttons on the page
   - Click them with random delays
   - Confirm the unfollow action
   - Log the results
3. You can pause, resume, or stop at any time

### Step 6: Monitor Progress

Watch the extension popup for:
- Live stats (unfollowed/failed/total)
- Current user being processed
- Activity log with timestamps
- Status updates

## ⚙️ Settings Explained

### Session Limit
Maximum number of unfollows in one session. When reached, the extension stops automatically. This helps you stay under rate limits.

**Recommended**: 20-25

### Daily/Hourly Limits
Total unfollows allowed per day/hour. The extension tracks these across sessions.

**Recommended**:
- Daily: 40-50
- Hourly: 25-30

### Delay Range
Random delay (in seconds) between each unfollow action. Randomization makes the automation appear more human.

**Recommended**: 30-60 seconds
**Conservative**: 60-120 seconds
**Aggressive**: 20-40 seconds (higher risk)

## 🛡️ Safety Features

The extension includes several safety features to prevent account issues:

1. **Rate Limiting**: Respects daily, hourly, and session limits
2. **Random Delays**: Varies timing to appear human
3. **Confirmation**: Always confirms unfollow actions
4. **Activity Log**: Tracks all actions for review
5. **Pause/Stop**: Can interrupt at any time

## 📊 Activity Log

The popup shows a real-time activity log with:
- ✅ Successful unfollows (green)
- ❌ Failed unfollows (red)
- ℹ️ Status updates (blue)
- Timestamps for each action

## 💾 Export History

Click the "📥 Export History" button to download a CSV file with:
- Timestamp
- Username
- Status (success/failed)
- Error message (if failed)

## 🔧 Development

### File Structure

```
extension-chrome/
├── manifest.json          # Extension configuration
├── content.js            # Runs on X.com, does the unfollowing
├── background.js         # Service worker, manages state
├── popup.html            # Extension popup UI
├── popup.css             # Popup styling
├── popup.js              # Popup logic
├── icons/                # Extension icons
│   ├── icon16.png
│   ├── icon48.png
│   └── icon128.png
└── README.md             # This file
```

### How It Works

1. **Content Script** (`content.js`):
   - Runs on x.com/twitter.com pages
   - Finds "Following" buttons
   - Clicks them and confirms
   - Reports results to background script

2. **Background Script** (`background.js`):
   - Coordinates between popup and content script
   - Manages settings and state
   - Stores history in Chrome storage
   - Handles data persistence

3. **Popup** (`popup.html/js/css`):
   - User interface for controls
   - Shows live stats and logs
   - Settings management
   - Communicates with background script

### Testing

1. Make changes to the code
2. Go to `chrome://extensions/`
3. Click the refresh icon on the extension card
4. Test on x.com

### Debugging

- **Content script**: Open DevTools on x.com → Console
- **Background script**: Extensions page → "Inspect views: service worker"
- **Popup**: Right-click extension icon → "Inspect popup"

## 🚨 Troubleshooting

### Extension doesn't start

**Problem**: Click "Start" but nothing happens

**Solutions**:
- Make sure you're on x.com or twitter.com
- Refresh the page
- Check if you're on a Following list page
- Open DevTools console for errors

### No "Following" buttons found

**Problem**: Extension says "No more following buttons found"

**Solutions**:
- Make sure you're on the Following list page
- Scroll down to load more users
- Refresh the page
- Check if X's UI changed (selectors may need updating)

### Unfollow not confirming

**Problem**: Following button clicked but not confirming

**Solutions**:
- Wait a few seconds (modal needs to load)
- Check if X's modal UI changed
- Open DevTools console to see errors
- May need to update selectors in `content.js`

### Rate limit errors

**Problem**: X is blocking actions

**Solutions**:
- Stop the extension immediately
- Wait 24 hours
- Use more conservative settings (longer delays, lower limits)
- Don't use other automation tools simultaneously

## 📝 Updating Selectors

X.com frequently changes their UI. If the extension stops working:

1. Open `content.js`
2. Find the `SELECTORS` object at the top
3. Update selectors to match current X.com HTML
4. How to find selectors:
   - Right-click "Following" button → "Inspect"
   - Note the `data-testid` or other attributes
   - Update in `SELECTORS` object

## 🔒 Privacy & Permissions

### Permissions Required

- **storage**: Save settings and history locally
- **activeTab**: Access current tab to check if on X.com
- **scripting**: Inject content script on X.com

### Data Storage

All data is stored locally in your browser:
- Settings (limits, delays)
- Unfollow history
- Session state

**Nothing is sent to external servers.**

### Host Permissions

The extension only runs on:
- `https://twitter.com/*`
- `https://x.com/*`

It cannot access any other websites.

## 🤝 Contributing

Contributions welcome! To contribute:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file

## ⚠️ Disclaimer

This extension is for educational purposes. Use at your own risk. Batch unfollowing may violate X's Terms of Service. The authors are not responsible for any account restrictions or bans.

**Recommendations**:
- Use conservative settings
- Don't unfollow too many users at once
- Add longer delays between actions
- Monitor your account for any warnings

## 🆚 Comparison with Tauri Version

| Aspect | Tauri Version | Chrome Extension |
|--------|---------------|------------------|
| **Setup** | Install Rust, GTK, build tools | Load extension in Chrome |
| **Build Time** | 5-10 minutes | Instant |
| **File Size** | ~10 MB | ~50 KB |
| **Updates** | Rebuild and redistribute | Reload extension |
| **Debugging** | Complex, limited tools | Chrome DevTools |
| **Distribution** | Manual download/install | Chrome Web Store |
| **Cross-platform** | Need builds for each OS | Works on all platforms |
| **Development** | Rust + TypeScript | Just JavaScript |
| **Database** | SQLite with SQLx | Chrome Storage API |
| **Testing** | Need full build | Instant reload |

**Result**: Chrome Extension is **10x easier** to develop, test, and distribute!

## 🎉 Success!

You've successfully set up the X Unfollow Chrome Extension! No more Tauri complications. Just load, click, and unfollow.

Happy unfollowing! 🚀
