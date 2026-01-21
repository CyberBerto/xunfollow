# XUnfollow - Chrome Extension for X (Twitter)

Automatically batch unfollow users on X (Twitter) with smart rate limiting and easy controls.

**Simple. Fast. No setup required.**

![Chrome Extension](https://img.shields.io/badge/Chrome-Extension-blue?style=flat-square&logo=googlechrome)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)
![Size](https://img.shields.io/badge/Size-56KB-orange?style=flat-square)

---

## 🚀 Quick Start

### 1. Install the Extension

```bash
# Clone this repository
git clone https://github.com/CyberBerto/xunfollow.git
cd xunfollow
```

In Chrome:
1. Open `chrome://extensions/`
2. Enable **"Developer mode"** (toggle in top right)
3. Click **"Load unpacked"**
4. Select the `extension-chrome` folder
5. Done! ✅

### 2. Use It

1. Go to [x.com](https://x.com) and log in
2. Navigate to your **Following** list
3. Click the **XUnfollow** extension icon in your toolbar
4. Click **"Start"** and watch it work!

That's it! No build process, no dependencies, no complicated setup.

---

## ✨ Features

- 🎯 **Batch Unfollow** - Automate unfollowing on X.com
- 🛡️ **Smart Rate Limiting** - Daily, hourly, and session limits to stay safe
- ⏱️ **Random Delays** - 30-60s delays to appear human
- ⏸️ **Pause/Resume/Stop** - Full control at any time
- 📊 **Activity Log** - Real-time tracking with timestamps
- 💾 **Export History** - Download your activity as CSV
- 🔒 **100% Local** - No external servers, all data stays in your browser
- ⚙️ **Customizable** - Adjust limits and delays to your preference

---

## 🎮 Try the Demo First

Not ready to install? See how it works with the interactive demo:

```bash
open poc-demo/unfollow-demo.html
```

Or just drag `poc-demo/unfollow-demo.html` into your browser!

---

## 📖 How It Works

1. **Content Script** runs on X.com and finds "Following" buttons
2. **Clicks buttons** with random delays between actions
3. **Confirms unfollows** in the modal that appears
4. **Logs everything** to the activity log
5. **Respects limits** - stops when daily/hourly/session limits are reached

Simple, effective, and safe.

---

## ⚙️ Settings

Customize the extension to your needs:

| Setting | Default | Description |
|---------|---------|-------------|
| **Daily Limit** | 50 | Max unfollows per day |
| **Hourly Limit** | 30 | Max unfollows per hour |
| **Session Limit** | 25 | Unfollows before automatic break |
| **Min Delay** | 30s | Minimum delay between unfollows |
| **Max Delay** | 60s | Maximum delay between unfollows |

**Tip:** Use longer delays (60-120s) for safer operation.

---

## 🛡️ Safety Features

- ✅ **Built-in rate limiting** - Prevents hitting X's limits
- ✅ **Random delays** - Makes automation appear human
- ✅ **Pause/stop anytime** - Full control over the process
- ✅ **Activity logging** - See exactly what happened
- ✅ **Local storage only** - No data sent anywhere

**⚠️ Warning:** Batch unfollowing may violate X's Terms of Service. Use conservative settings and at your own risk.

---

## 📁 Repository Structure

```
xunfollow/
├── extension-chrome/          Chrome Extension
│   ├── manifest.json         Extension configuration
│   ├── content.js           X.com automation script
│   ├── background.js        Service worker
│   ├── popup.html/css/js    Extension UI
│   └── README.md            Detailed user guide
│
├── poc-demo/                Interactive demo
│   └── unfollow-demo.html   Standalone demo page
│
└── Documentation
    ├── README.md            This file
    ├── ARCHITECTURE.md      Technical overview
    └── CHROME_EXTENSION_GUIDE.md  Why Chrome Extension?
```

---

## 🛠️ Development

Want to modify the extension?

```bash
# Edit any file in extension-chrome/
vim extension-chrome/popup.js

# Reload the extension
# Go to chrome://extensions/ → Click the reload icon

# Test your changes
# Click the extension icon on x.com
```

**No build process!** Just edit JavaScript and reload. That's it.

### Key Files

- `content.js` - Runs on X.com, does the unfollowing
- `background.js` - Manages state and settings
- `popup.js` - Extension popup UI logic
- `popup.html/css` - Extension popup structure and styling

---

## 📚 Documentation

- **[Extension User Guide](extension-chrome/README.md)** - Complete installation and usage instructions
- **[Architecture Overview](ARCHITECTURE.md)** - How the extension works
- **[Chrome Extension Guide](CHROME_EXTENSION_GUIDE.md)** - Why we built this as a Chrome Extension
- **[Interactive Demo](poc-demo/unfollow-demo.html)** - See it in action without installing

---

## 🤝 Contributing

Contributions are welcome! Here's how:

1. **Fork** this repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Test** thoroughly on X.com
5. **Commit** your changes (`git commit -m 'Add amazing feature'`)
6. **Push** to your branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

### Contribution Ideas

- 🎨 Improve the UI/UX
- 🐛 Fix bugs or edge cases
- 📝 Improve documentation
- ✨ Add new features (with conservative defaults)
- 🌐 Add Firefox support (WebExtensions)
- 🧪 Add automated tests

---

## 🔧 Troubleshooting

### Extension doesn't start
- Make sure you're on x.com or twitter.com
- Refresh the page
- Check console for errors (F12)

### No "Following" buttons found
- Navigate to your Following list page
- Scroll down to load more users
- X may have changed their UI - check for selector updates needed

### Rate limit errors from X
- Stop the extension immediately
- Use more conservative settings (longer delays, lower limits)
- Wait 24 hours before trying again

### Need help?
Open an [issue](https://github.com/CyberBerto/xunfollow/issues) with:
- What you were trying to do
- What happened
- Browser console errors (F12 → Console tab)

---

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details

Free to use, modify, and distribute. No warranty provided.

---

## ⚠️ Disclaimer

This tool is for educational purposes. Use at your own risk.

- Batch unfollowing may violate X's Terms of Service
- Your account could be restricted or banned
- We recommend conservative settings (long delays, low limits)
- The authors are not responsible for any account issues

**Use responsibly.**

---

## 🌟 Why This Extension?

- **No Rust or complex build tools** - Pure JavaScript
- **No external dependencies** - Self-contained
- **No data collection** - Everything stays local
- **Easy to audit** - Simple, readable code
- **Easy to modify** - No compilation needed
- **Works everywhere** - Any OS with Chrome

Built for simplicity and transparency.

---

## 📊 Stats

- **Size:** 56 KB (extension) + 15 KB (demo)
- **Files:** 10 files
- **Languages:** JavaScript, HTML, CSS
- **Dependencies:** None (uses browser APIs only)
- **Build Time:** 0 seconds (no build needed!)

---

## 🚀 Coming Soon

- [ ] Publish to Chrome Web Store
- [ ] Firefox support
- [ ] Whitelist specific users
- [ ] Import/export following lists
- [ ] Scheduling (unfollow at specific times)
- [ ] Analytics dashboard

---

## 💬 Support

- 🐛 **Report bugs:** [GitHub Issues](https://github.com/CyberBerto/xunfollow/issues)
- 💡 **Feature requests:** [GitHub Issues](https://github.com/CyberBerto/xunfollow/issues)
- 📖 **Documentation:** Check the [docs](extension-chrome/README.md)
- 💬 **Questions:** Open a [discussion](https://github.com/CyberBerto/xunfollow/discussions)

---

## 🎉 Acknowledgments

Built with ❤️ for people tired of manually unfollowing hundreds of accounts.

**Star this repo if it helped you!** ⭐

---

**[Get Started →](extension-chrome/README.md)** | **[View Demo →](poc-demo/unfollow-demo.html)** | **[Report Issue →](https://github.com/CyberBerto/xunfollow/issues)**
