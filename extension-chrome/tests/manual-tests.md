# Manual Testing Checklist

## Pre-Testing Setup

- [ ] Extension loaded in Chrome (`chrome://extensions/`)
- [ ] Developer mode enabled
- [ ] Extension icon visible in toolbar
- [ ] Logged into X.com account with following list

## Installation Tests

### First Install
- [ ] Extension loads without errors
- [ ] Default settings are set (Daily: 50, Hourly: 30, Session: 25)
- [ ] Popup opens when clicking icon
- [ ] Icons display correctly (16px, 48px, 128px)

### Reload Extension
- [ ] Settings persist after reload
- [ ] History persists after reload
- [ ] Extension functions normally after reload

## Core Functionality Tests

### Content Script
- [ ] Content script loads on x.com
- [ ] Content script loads on twitter.com
- [ ] Content script does NOT load on other sites
- [ ] Console shows content script initialization message

### Finding Following Buttons
- [ ] Navigate to Following list (x.com/[username]/following)
- [ ] Extension finds "Following" buttons
- [ ] Extension counts correct number of buttons
- [ ] Works with different X.com UI variations

### Unfollow Process
- [ ] Click extension icon → popup opens
- [ ] Click "Start" button
- [ ] First "Following" button gets clicked
- [ ] Confirmation modal appears
- [ ] "Unfollow" in modal gets clicked
- [ ] User gets unfollowed successfully
- [ ] Process continues to next user

### Rate Limiting
- [ ] Session limit respected (stops after 25 default)
- [ ] Delay between unfollows (30-60s default)
- [ ] Daily limit tracked
- [ ] Hourly limit tracked

### Controls
- [ ] "Pause" button pauses unfollowing
- [ ] "Resume" button resumes from pause
- [ ] "Stop" button stops completely
- [ ] Controls update UI state correctly

## UI/UX Tests

### Popup Interface
- [ ] Popup opens without errors
- [ ] All UI elements visible
- [ ] Stats update in real-time
- [ ] Activity log shows entries
- [ ] Settings icon clickable

### Settings Modal
- [ ] Settings icon opens modal
- [ ] All settings fields visible
- [ ] Can modify all settings
- [ ] "Save" button saves changes
- [ ] "Cancel" button discards changes
- [ ] Modal closes properly

### Activity Log
- [ ] Success entries show in green
- [ ] Failure entries show in red
- [ ] Info entries show in blue
- [ ] Timestamps display correctly
- [ ] "Clear" button clears log
- [ ] Log auto-scrolls to newest entry

### Stats Display
- [ ] Unfollowed count increments
- [ ] Failed count increments on errors
- [ ] Total count = unfollowed + failed
- [ ] Current username displays during operation

## Edge Cases & Error Handling

### Network Issues
- [ ] Handles slow page loads gracefully
- [ ] Handles network timeouts
- [ ] Shows appropriate error messages

### X.com UI Changes
- [ ] Falls back to text-based button detection
- [ ] Logs errors for missing selectors
- [ ] Doesn't crash on unexpected DOM

### User Interactions
- [ ] Handles user scrolling page
- [ ] Handles user clicking elsewhere
- [ ] Handles user refreshing page mid-operation
- [ ] Handles user closing/reopening popup

### Browser Events
- [ ] Works after computer sleep/wake
- [ ] Works after browser restart
- [ ] Works in multiple tabs (only active tab)

## Data Persistence

### Settings Storage
- [ ] Settings save to chrome.storage.local
- [ ] Settings load correctly on startup
- [ ] Settings persist across sessions

### History Storage
- [ ] History saves successful unfollows
- [ ] History saves failed unfollows
- [ ] History limited to 1000 entries
- [ ] History includes timestamps

### State Persistence
- [ ] Running state tracked
- [ ] Stats tracked across sessions
- [ ] Can resume after browser restart (if applicable)

## Export Functionality

### History Export
- [ ] "Export History" button works
- [ ] CSV file downloads correctly
- [ ] CSV contains all history data
- [ ] CSV format is valid (opens in Excel/Sheets)
- [ ] Filename includes timestamp

## Performance Tests

### Resource Usage
- [ ] Extension doesn't slow down browser significantly
- [ ] Memory usage reasonable (<50MB)
- [ ] CPU usage reasonable during operation
- [ ] No memory leaks over extended use

### Speed
- [ ] Popup opens instantly (<100ms)
- [ ] Settings load quickly
- [ ] Button clicks respond immediately
- [ ] Unfollow process maintains consistent speed

## Security Tests

### Permissions
- [ ] Only requests necessary permissions
- [ ] Only runs on x.com/twitter.com
- [ ] Doesn't access other websites
- [ ] Doesn't send data externally

### Data Privacy
- [ ] No data sent to external servers
- [ ] All data stored locally
- [ ] Storage is user-specific
- [ ] Can clear all data easily

## Compatibility Tests

### Chrome Versions
- [ ] Works on Chrome 120+
- [ ] Works on Chrome Beta
- [ ] Works on Chromium

### Operating Systems
- [ ] Works on Windows
- [ ] Works on macOS
- [ ] Works on Linux

### Screen Sizes
- [ ] Popup fits on small screens
- [ ] Popup fits on large screens
- [ ] UI elements don't overlap
- [ ] Text is readable at all sizes

## Regression Tests

After making changes, verify:
- [ ] All previous functionality still works
- [ ] No new console errors
- [ ] Settings still save/load correctly
- [ ] History still exports correctly
- [ ] Unfoll process still completes successfully

## Stress Tests

### High Volume
- [ ] Test with 100+ users in following list
- [ ] Test with 1000+ users
- [ ] Extension handles large queues
- [ ] Performance doesn't degrade

### Long Duration
- [ ] Run for 1 hour continuously
- [ ] Run overnight (if safe)
- [ ] No crashes or hangs
- [ ] Memory usage stable

### Rapid Actions
- [ ] Click buttons rapidly
- [ ] Open/close popup rapidly
- [ ] Start/stop rapidly
- [ ] No crashes or race conditions

## Final Checklist

Before marking complete:
- [ ] All critical tests passing
- [ ] No unresolved errors in console
- [ ] Extension works end-to-end
- [ ] Documentation is accurate
- [ ] No security concerns
- [ ] Ready for release/distribution

## Test Results

| Test Category | Pass | Fail | Notes |
|--------------|------|------|-------|
| Installation | ☐ | ☐ | |
| Core Functionality | ☐ | ☐ | |
| UI/UX | ☐ | ☐ | |
| Edge Cases | ☐ | ☐ | |
| Data Persistence | ☐ | ☐ | |
| Performance | ☐ | ☐ | |
| Security | ☐ | ☐ | |
| Compatibility | ☐ | ☐ | |

---

**Tested By:** ________________
**Date:** ________________
**Version:** ________________
**Notes:**
