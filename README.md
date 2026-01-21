# XUnfollow - Batch Unfollow for X (Twitter)

> **🎉 Now Available as a Chrome Extension!** No complex setup, just install and go.

Automatically unfollow users on X (Twitter) with smart rate limiting and easy controls.

## 🚀 Quick Start

### Chrome Extension (Recommended)

The easiest way to use XUnfollow:

1. **Install the extension:**
   - Open Chrome and go to `chrome://extensions/`
   - Enable "Developer mode" (top right toggle)
   - Click "Load unpacked"
   - Select the `extension-chrome` folder from this repo

2. **Use it:**
   - Go to https://x.com and log in
   - Navigate to your Following list
   - Click the extension icon
   - Click "Start" and watch it work!

📖 **[Full Extension Guide →](extension-chrome/README.md)**

### Try the Demo First

Not ready to install? Try the interactive demo:

```bash
open poc-demo/unfollow-demo.html
```

This shows how the automation works without any installation.

## ✨ Features

- ✅ **Batch unfollow** - Automate unfollowing on X.com
- ✅ **Smart rate limiting** - Daily, hourly, and session limits
- ✅ **Random delays** - Appear human (30-60s between actions)
- ✅ **Pause/Resume** - Full control at any time
- ✅ **Activity log** - Track all actions with timestamps
- ✅ **Export history** - Download your activity as CSV
- ✅ **No external servers** - Everything runs locally

## 📁 What's in This Repo

```
xunfollow/
├── extension-chrome/    ⭐ Chrome Extension (recommended)
├── poc-demo/           📺 Interactive demo
├── src/                🔧 React frontend (Tauri version)
├── src-tauri/          🦀 Rust backend (Tauri version)
└── docs/               📚 Documentation
```

## 🎯 Chrome Extension vs Tauri

| Feature | Chrome Extension | Tauri Desktop App |
|---------|-----------------|-------------------|
| **Setup** | 2 minutes | 30+ minutes |
| **Requirements** | Just Chrome | Rust + GTK + Build tools |
| **Size** | 50 KB | 10+ MB |
| **Updates** | Automatic | Manual rebuild |
| **Debugging** | Chrome DevTools | Complex |
| **Works on** | All platforms | Needs platform builds |

**Recommendation:** Use the Chrome Extension unless you specifically need a desktop app.

## 📖 Documentation

- **[Chrome Extension Guide](extension-chrome/README.md)** - How to install and use
- **[POC Demo](poc-demo/unfollow-demo.html)** - Interactive demo
- **[Migration Guide](CHROME_EXTENSION_GUIDE.md)** - Why we moved from Tauri
- **[Refactoring Guide](REFACTORING.md)** - Code improvements

## 🛠️ Development

### Chrome Extension

```bash
# Edit the code
cd extension-chrome
vim popup.js

# Reload in Chrome
# Go to chrome://extensions/ → Click reload icon

# Test
# Click extension icon on x.com
```

No build step needed - just edit and reload!

### Tauri App (Legacy)

If you still want to build the Tauri version:

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

**Note:** Requires Rust toolchain and system dependencies. See [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites) for setup.

## ⚙️ Settings

Customize in the extension popup:

- **Daily Limit**: 50 (max unfollows per day)
- **Hourly Limit**: 30 (max unfollows per hour)
- **Session Limit**: 25 (unfollows before stopping)
- **Min/Max Delay**: 30-60 seconds (random delay between actions)

## 🛡️ Safety

- ✅ Built-in rate limiting
- ✅ Random delays to appear human
- ✅ Can pause/stop anytime
- ✅ Activity logging
- ✅ No data sent to external servers

**Warning:** Batch unfollowing may violate X's Terms of Service. Use conservative settings and at your own risk.

## 🤝 Contributing

Contributions welcome!

1. Fork the repo
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📋 Branches

- **`main`** - Original Tauri app
- **`claude/test-refactor-code-cXjvC`** - Chrome Extension + Refactored code (you are here)

## 📄 License

MIT License - See LICENSE file for details

## 🆘 Support

- **Issues:** [GitHub Issues](https://github.com/CyberBerto/xunfollow/issues)
- **Questions:** Open a discussion or issue

## ⚡ Quick Links

- [Install Chrome Extension](extension-chrome/) - Start here!
- [View Demo](poc-demo/unfollow-demo.html) - See how it works
- [Read Full Guide](extension-chrome/README.md) - Complete documentation
- [Migration Story](CHROME_EXTENSION_GUIDE.md) - Why Chrome Extension?

---

**Made with ❤️ for people tired of manually unfollowing**
