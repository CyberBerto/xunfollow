# XUnfollow Architecture

This document explains the repository structure and different implementations.

## 📁 Repository Structure

```
xunfollow/
│
├── extension-chrome/          ⭐ RECOMMENDED: Chrome Extension
│   ├── manifest.json          Chrome extension config
│   ├── content.js             Runs on X.com, does the unfollowing
│   ├── background.js          Service worker, manages state
│   ├── popup.html/css/js      Extension UI (popup when you click icon)
│   └── README.md              Complete usage guide
│
├── poc-demo/                  📺 Proof of Concept Demo
│   └── unfollow-demo.html     Standalone interactive demo
│
├── src/                       🔧 React Frontend (for Tauri)
│   ├── App.tsx                Main component (original)
│   ├── App-refactored.tsx     Refactored version (experimental)
│   ├── components/            Extracted components (experimental)
│   ├── hooks/                 Custom React hooks (experimental)
│   └── types/                 TypeScript types (experimental)
│
├── src-tauri/                 🦀 Rust Backend (Tauri Desktop App)
│   ├── src/
│   │   ├── main.rs            Entry point
│   │   ├── lib.rs             Main library
│   │   ├── lib-refactored.rs  Refactored version (experimental)
│   │   ├── commands/          Tauri commands
│   │   ├── services/          Business logic
│   │   ├── db/                Database module (experimental)
│   │   ├── constants.rs       Config constants (experimental)
│   │   └── utils.rs           Helper functions (experimental)
│   ├── Cargo.toml             Rust dependencies
│   └── migrations/            Database migrations
│
└── docs/                      📚 Documentation
    ├── README.md              Main readme (you're here!)
    ├── CHROME_EXTENSION_GUIDE.md   Why Chrome Extension?
    ├── REFACTORING.md         Code refactoring details
    └── CLAUDE.md              Development guidelines
```

## 🎯 Three Implementations

### 1. Chrome Extension (Recommended)

**Location:** `extension-chrome/`
**Status:** ✅ Complete and production-ready
**Tech:** JavaScript (vanilla)
**Use when:** You want the easiest, fastest solution

**Architecture:**
```
┌─────────────────┐
│  Popup UI       │  User interacts here
│  (popup.js)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Background     │  Coordinates everything
│  (background.js)│  Stores state & settings
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Content Script │  Runs on X.com
│  (content.js)   │  Does the unfollowing
└─────────────────┘
```

### 2. Proof of Concept Demo

**Location:** `poc-demo/unfollow-demo.html`
**Status:** ✅ Complete
**Tech:** Standalone HTML
**Use when:** You want to see how it works without installing anything

**Purpose:**
- Demonstrates the concept visually
- Shows the core logic
- No installation required

### 3. Tauri Desktop App (Legacy)

**Location:** `src/` + `src-tauri/`
**Status:** ⚠️ Legacy, harder to maintain
**Tech:** React + TypeScript + Rust
**Use when:** You specifically need a desktop app

**Architecture:**
```
┌──────────────────┐
│  React Frontend  │  UI in browser webview
│  (src/)          │
└────────┬─────────┘
         │ Tauri IPC
         ▼
┌──────────────────┐
│  Rust Backend    │  Commands, database, logic
│  (src-tauri/)    │
└──────────────────┘
```

## 🔄 Implementation Comparison

| Aspect | Chrome Extension | POC Demo | Tauri Desktop |
|--------|-----------------|----------|---------------|
| **Complexity** | Low | Very Low | High |
| **Setup Time** | 2 min | 0 min | 30+ min |
| **Languages** | JavaScript | JavaScript | JS + Rust |
| **Build Required** | No | No | Yes (complex) |
| **Distribution** | Chrome Web Store | File | Platform binaries |
| **Size** | 50 KB | 11 KB | 10+ MB |
| **Updates** | Auto via store | N/A | Manual |
| **Platform** | Any with Chrome | Any browser | OS-specific builds |

## 🧩 Shared Concepts

All implementations share these core concepts:

### Rate Limiting
- **Daily Limit**: Max unfollows per day
- **Hourly Limit**: Max unfollows per hour
- **Session Limit**: Unfollows before break
- **Random Delays**: 30-60s between actions

### Workflow
1. Find "Following" buttons on X.com
2. Click button
3. Wait for confirmation modal
4. Click "Unfollow" in modal
5. Wait random delay
6. Repeat

### Safety Features
- Rate limiting to avoid bans
- Random delays to appear human
- Activity logging
- Pause/resume controls

## 📂 Experimental Code

Files marked "experimental" are refactored versions that were created but not fully integrated:

**Frontend Refactoring:**
- `src/App-refactored.tsx` - Cleaner component structure
- `src/components/` - Extracted UI components
- `src/hooks/` - Custom React hooks
- `src/types/` - TypeScript definitions

**Backend Refactoring:**
- `src-tauri/src/lib-refactored.rs` - Improved setup
- `src-tauri/src/db/` - Database module
- `src-tauri/src/constants.rs` - Configuration
- `src-tauri/src/utils.rs` - Helper functions

These show how the Tauri app could be improved but aren't currently in use since the Chrome Extension is the recommended path forward.

## 🚀 Which Should You Use?

### Use Chrome Extension if:
- ✅ You want the simplest solution
- ✅ You're okay with Chrome-only (for now)
- ✅ You want easy updates
- ✅ You want to avoid Rust/build complexity

### Use POC Demo if:
- ✅ You want to see how it works first
- ✅ You're evaluating the tool
- ✅ You want to understand the logic

### Use Tauri Desktop App if:
- ✅ You specifically need a native desktop app
- ✅ You need offline functionality
- ✅ You have Rust toolchain set up
- ✅ You're willing to deal with build complexity

## 📚 Documentation Guide

- **[README.md](README.md)** - Start here! Quick overview and setup
- **[extension-chrome/README.md](extension-chrome/README.md)** - Chrome Extension guide
- **[CHROME_EXTENSION_GUIDE.md](CHROME_EXTENSION_GUIDE.md)** - Why Chrome Extension?
- **[REFACTORING.md](REFACTORING.md)** - Code improvements made
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - This file!
- **[CLAUDE.md](CLAUDE.md)** - Development guidelines

## 🤝 Contributing

When contributing, focus on:

1. **Chrome Extension** - Primary implementation
2. **Documentation** - Keep guides up to date
3. **Testing** - Verify on real X.com pages

The Tauri implementation is in maintenance mode - focus new features on the Chrome Extension.

## ❓ Questions?

- **Where should I start?** → [README.md](README.md)
- **How do I install?** → [extension-chrome/README.md](extension-chrome/README.md)
- **Why Chrome Extension?** → [CHROME_EXTENSION_GUIDE.md](CHROME_EXTENSION_GUIDE.md)
- **What was refactored?** → [REFACTORING.md](REFACTORING.md)

---

**Last Updated:** 2026-01-21
**Current Focus:** Chrome Extension (recommended)
