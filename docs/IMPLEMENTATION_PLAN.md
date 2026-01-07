# XUnfollow Enhancement Plan

## Summary

Add account validation, activity checking, enhanced status display, configurable inactivity threshold, and a feature request button with voice dictation.

## Features

1. **Account Existence Check** - Verify account exists before unfollowing
2. **Activity Check** - Check for visible timeline activity within configurable threshold
3. **Enhanced Status Display** - Show detailed statuses (not_found, inactive, active, not_following)
4. **Configurable Inactivity Threshold** - New setting: `inactivity_months` (default: 3)
5. **Feature Request Button** - Submit GitHub issues with voice dictation support

## Critical Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/commands/unfollow.rs` | Capture JS results, handle new statuses, emit new events |
| `src-tauri/src/commands/settings.rs` | Add `inactivity_months` field |
| `src-tauri/scripts/unfollow_profile.js` | Add activity detection, signal results via URL hash |
| `src/App.tsx` | New event listeners, status icons, settings UI, feature request modal |
| `src/App.css` | New status colors, modal styles |
| `src-tauri/migrations/002_activity_check.sql` | New migration for schema updates |

---

## Implementation Steps

### Phase 1: Database & Settings Foundation

**1.1 Create migration file** `src-tauri/migrations/002_activity_check.sql`:
```sql
INSERT OR IGNORE INTO settings (key, value) VALUES ('inactivity_months', '3');
ALTER TABLE queue ADD COLUMN result_status TEXT DEFAULT NULL;
ALTER TABLE queue ADD COLUMN last_activity_date TEXT DEFAULT NULL;
```

**1.2 Update settings.rs** (lines 7-26):
- Add `inactivity_months: u32` to `Settings` struct
- Add default value `3` in `Default` impl
- Add parsing in `get_settings` match block (line 48-55)
- Add to pairs array in `update_settings` (line 76-82)
- Add validation: must be 1-24 months

**1.3 Update App.tsx AppSettings interface** (line 24-30):
- Add `inactivity_months: number`
- Add to default state (line 43-49)

---

### Phase 2: Capture JS Script Results

**2.1 Fix execute_unfollow in unfollow.rs** (lines 342-365):

The current implementation ignores JS results (line 364 returns hardcoded "css"). Fix by:

1. Update JS script to signal results via URL hash (like `fetch_following_list` does on line 609)
2. Poll for `#__XUNFOLLOW_RESULT__:` in URL hash
3. Parse JSON result containing: `success`, `error`, `result_status`, `method`, `last_activity`

**2.2 Add result struct:**
```rust
#[derive(Debug, Deserialize)]
struct ScriptResult {
    success: bool,
    error: Option<String>,
    method: Option<String>,
    result_status: Option<String>,  // "not_found", "not_following", "active", "inactive", "unfollowed"
    last_activity: Option<String>,  // ISO date YYYY-MM-DD
}
```

**2.3 Update execute_unfollow signature:**
```rust
async fn execute_unfollow(
    app: &tauri::AppHandle,
    username: &str,
    inactivity_months: u32,
) -> Result<(String, String, Option<String>), String>  // (method, result_status, last_activity)
```

---

### Phase 3: Activity Detection in JavaScript

**3.1 Update unfollow_profile.js** to add activity checking:

Add after line 8 (SELECTORS):
```javascript
const INACTIVITY_MONTHS = parseInt('__INACTIVITY_MONTHS__') || 3;

// Timeline selectors
tweetArticle: 'article[data-testid="tweet"]',
tweetTime: 'time[datetime]',
pinnedTweet: '[data-testid="socialContext"]'
```

Add activity check function:
```javascript
async function checkTimelineActivity() {
    await wait(2000);
    const tweets = document.querySelectorAll('article[data-testid="tweet"]');

    if (tweets.length === 0) {
        return { hasActivity: false, lastActivity: null };
    }

    let mostRecentDate = null;
    for (const tweet of tweets) {
        // Skip pinned tweets
        const pinned = tweet.querySelector('[data-testid="socialContext"]');
        if (pinned && pinned.textContent.toLowerCase().includes('pinned')) continue;

        const timeEl = tweet.querySelector('time[datetime]');
        if (timeEl) {
            const datetime = timeEl.getAttribute('datetime');
            if (datetime) {
                const tweetDate = new Date(datetime);
                if (!mostRecentDate || tweetDate > mostRecentDate) {
                    mostRecentDate = tweetDate;
                }
            }
        }
    }

    if (!mostRecentDate) {
        return { hasActivity: false, lastActivity: null };
    }

    const thresholdDate = new Date();
    thresholdDate.setMonth(thresholdDate.getMonth() - INACTIVITY_MONTHS);

    return {
        hasActivity: mostRecentDate >= thresholdDate,
        lastActivity: mostRecentDate.toISOString().split('T')[0]
    };
}
```

**3.2 Update tryCSS function** (around line 45):

After checking for 404, add:
```javascript
// Return early with result_status for not_found
if (notFound || errorPage) {
    return { success: false, error: 'user_not_found', result_status: 'not_found' };
}

// Check timeline activity BEFORE unfollowing
const activityCheck = await checkTimelineActivity();

// After finding follow button and confirming following status...
if (activityCheck.hasActivity) {
    return {
        success: false,
        error: 'Account is active',
        result_status: 'active',
        last_activity: activityCheck.lastActivity
    };
}

// Proceed with unfollow (account is inactive)
// ... existing unfollow logic ...

return {
    success: true,
    method: 'css',
    result_status: activityCheck.lastActivity ? 'inactive' : 'unfollowed',
    last_activity: activityCheck.lastActivity
};
```

**3.3 Add result signaling** (replace line 168):
```javascript
function signalResult(result) {
    window.location.hash = '__XUNFOLLOW_RESULT__:' + encodeURIComponent(JSON.stringify(result));
}

// At end of script, after getting result
signalResult(result);
return JSON.stringify(result);
```

---

### Phase 4: Backend Result Handling

**4.1 Update unfollow_loop in unfollow.rs** (around line 241):

Load inactivity_months from settings:
```rust
let inactivity_months: u32 = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'inactivity_months'")
    .fetch_optional(&db).await
    .ok().flatten()
    .map(|v: String| v.parse().unwrap_or(3))
    .unwrap_or(3);
```

Update result handling after execute_unfollow call:
```rust
match result {
    Ok((method, result_status, last_activity)) => {
        match result_status.as_str() {
            "unfollowed" | "inactive" => {
                // Existing success logic - update queue, add to history, increment rate limiter
                // Emit "unfollow:success" for unfollowed, "unfollow:inactive" for inactive
            }
            "not_found" => {
                // Mark completed with result_status, emit "unfollow:not_found"
            }
            "not_following" => {
                // Mark completed with result_status, emit "unfollow:not_following"
            }
            "active" => {
                // Mark as "skipped" status (not completed), emit "unfollow:active"
                // DON'T increment rate limiter
            }
            _ => { /* handle as error */ }
        }
    }
    Err(error) => { /* existing error handling */ }
}
```

**4.2 Add new events:**
- `unfollow:not_found` - Account doesn't exist
- `unfollow:not_following` - Not following this account
- `unfollow:active` - Account is active, skipped
- `unfollow:inactive` - Unfollowed due to inactivity (different from regular success)

---

### Phase 5: Frontend Status Display

**5.1 Update LogEntry interface** (App.tsx line 9-14):
```typescript
interface LogEntry {
  username: string;
  status: "success" | "failed" | "pending" | "not_found" | "not_following" | "inactive" | "active";
  timestamp: string;
  error?: string;
  lastActivity?: string;
}
```

**5.2 Add new event listeners** (after line 139):
```typescript
// unfollow:not_found
const unsub7 = await listen<{ username: string }>("unfollow:not_found", (event) => {
  addLog({ username: event.payload.username, status: "not_found", timestamp: new Date().toLocaleTimeString() });
  loadQueueCount();
});

// unfollow:not_following
const unsub8 = await listen<{ username: string }>("unfollow:not_following", (event) => {
  addLog({ username: event.payload.username, status: "not_following", timestamp: new Date().toLocaleTimeString() });
  loadQueueCount();
});

// unfollow:active
const unsub9 = await listen<{ username: string; last_activity: string }>("unfollow:active", (event) => {
  addLog({ username: event.payload.username, status: "active", timestamp: new Date().toLocaleTimeString(), lastActivity: event.payload.last_activity });
  loadQueueCount();
});

// unfollow:inactive
const unsub10 = await listen<{ username: string; last_activity: string }>("unfollow:inactive", (event) => {
  addLog({ username: event.payload.username, status: "inactive", timestamp: new Date().toLocaleTimeString(), lastActivity: event.payload.last_activity });
  loadQueueCount();
});
```

**5.3 Add icons** (line 6):
```typescript
import { ..., AlertTriangle, UserX, Activity, MessageSquare, Mic } from "lucide-react";
```

**5.4 Update log item rendering** (lines 458-474):
```tsx
<div key={i} className="log-item" title={log.lastActivity ? `Last activity: ${log.lastActivity}` : undefined}>
  {log.status === "success" && <Check size={14} className="log-icon success" />}
  {log.status === "failed" && <X size={14} className="log-icon error" />}
  {log.status === "pending" && <Clock size={14} className="log-icon pending" />}
  {log.status === "not_found" && <UserX size={14} className="log-icon warning" />}
  {log.status === "not_following" && <AlertTriangle size={14} className="log-icon warning" />}
  {log.status === "inactive" && <Check size={14} className="log-icon inactive" />}
  {log.status === "active" && <Activity size={14} className="log-icon active" />}
  <span className="log-username">@{log.username}</span>
  {["inactive", "active", "not_found", "not_following"].includes(log.status) && (
    <span className="log-status-badge" data-status={log.status}>
      {log.status === "inactive" && "inactive"}
      {log.status === "active" && "active"}
      {log.status === "not_found" && "404"}
      {log.status === "not_following" && "n/a"}
    </span>
  )}
  <span className="log-time">{log.timestamp}</span>
</div>
```

---

### Phase 6: Settings UI

**6.1 Add inactivity threshold input** in settings modal (after line 542):
```tsx
<div className="form-group">
  <label className="form-label">Inactivity Threshold (months)</label>
  <p className="form-hint">Skip unfollowing accounts active within this period</p>
  <input
    type="number"
    className="form-input"
    value={settings.inactivity_months}
    min={1}
    max={24}
    onChange={(e) => setSettings({
      ...settings,
      inactivity_months: Math.min(24, Math.max(1, Number(e.target.value)))
    })}
  />
</div>
```

---

### Phase 7: Feature Request Button

**7.1 Add state variables** (after line 53):
```typescript
const [showFeatureRequest, setShowFeatureRequest] = useState(false);
const [featureText, setFeatureText] = useState("");
const [isRecording, setIsRecording] = useState(false);
```

**7.2 Add feature request button to header** (line 341-348):
```tsx
<div className="header-actions">
  <button className="icon-btn" onClick={() => setShowFeatureRequest(true)} title="Request Feature">
    <MessageSquare size={20} />
  </button>
  {/* existing buttons */}
</div>
```

**7.3 Add handlers:**
```typescript
const handleVoiceDictation = () => {
  const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
  if (!SpeechRecognition) {
    alert("Voice dictation not supported in this browser.");
    return;
  }

  if (isRecording) {
    setIsRecording(false);
    return;
  }

  const recognition = new SpeechRecognition();
  recognition.continuous = true;
  recognition.interimResults = true;
  recognition.lang = 'en-US';

  recognition.onstart = () => setIsRecording(true);
  recognition.onresult = (event: any) => {
    let transcript = '';
    for (let i = event.resultIndex; i < event.results.length; i++) {
      transcript += event.results[i][0].transcript;
    }
    setFeatureText(prev => prev + ' ' + transcript);
  };
  recognition.onerror = () => setIsRecording(false);
  recognition.onend = () => setIsRecording(false);
  recognition.start();
};

const handleSubmitFeatureRequest = async () => {
  if (!featureText.trim()) return;

  const title = featureText.trim().substring(0, 80);
  const body = `## Feature Request\n\n${featureText.trim()}\n\n---\n*Submitted via XUnfollow app*`;
  const issueUrl = `https://github.com/CyberBerto/xunfollow/issues/new?title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}&labels=enhancement`;

  const { open } = await import("@tauri-apps/plugin-opener");
  await open(issueUrl);

  setFeatureText("");
  setShowFeatureRequest(false);
};
```

**7.4 Add feature request modal** (after settings modal, line 579):
```tsx
{showFeatureRequest && (
  <div className="modal-overlay" onClick={() => setShowFeatureRequest(false)}>
    <div className="modal modal-wide" onClick={(e) => e.stopPropagation()}>
      <div className="modal-header">
        <h2>Request a Feature</h2>
        <button className="modal-close" onClick={() => setShowFeatureRequest(false)}>
          <X size={20} />
        </button>
      </div>

      <div className="form-group">
        <label className="form-label">Describe your feature request</label>
        <textarea
          className="form-textarea"
          value={featureText}
          onChange={(e) => setFeatureText(e.target.value)}
          placeholder="Tell us what feature you'd like to see..."
          rows={6}
        />
      </div>

      <div className="form-row">
        <button
          className={`btn btn-secondary ${isRecording ? 'btn-recording' : ''}`}
          onClick={handleVoiceDictation}
        >
          <Mic size={16} />
          {isRecording ? "Stop Recording" : "Voice Input"}
        </button>
        <button
          className="btn btn-primary"
          onClick={handleSubmitFeatureRequest}
          disabled={!featureText.trim()}
        >
          Submit on GitHub
        </button>
      </div>
    </div>
  </div>
)}
```

---

### Phase 8: CSS Updates

**Add to App.css:**
```css
/* New status colors */
.log-icon.warning { color: #facc15; }
.log-icon.inactive { color: #a78bfa; }
.log-icon.active { color: #4ade80; }

/* Status badges */
.log-status-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background-color: var(--bg-tertiary);
  text-transform: uppercase;
  font-weight: 500;
}
.log-status-badge[data-status="inactive"] { background-color: rgba(167, 139, 250, 0.2); color: #a78bfa; }
.log-status-badge[data-status="active"] { background-color: rgba(74, 222, 128, 0.2); color: #4ade80; }
.log-status-badge[data-status="not_found"] { background-color: rgba(248, 113, 113, 0.2); color: #f87171; }
.log-status-badge[data-status="not_following"] { background-color: rgba(250, 204, 21, 0.2); color: #facc15; }

/* Feature request modal */
.modal-wide { width: 32rem; }
.form-textarea {
  width: 100%;
  padding: 0.75rem;
  background-color: var(--bg-tertiary);
  border: none;
  border-radius: 0.375rem;
  color: var(--text-primary);
  font-family: inherit;
  resize: vertical;
  min-height: 120px;
}
.form-textarea:focus { outline: 2px solid var(--blue-500); }
.form-hint { font-size: 0.75rem; color: var(--text-muted); margin-bottom: 0.5rem; }
.btn-secondary { background-color: var(--bg-tertiary); color: var(--text-primary); }
.btn-secondary:hover { background-color: #4b5563; }
.btn-recording { background-color: #dc2626; animation: pulse 1.5s ease-in-out infinite; }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
```

---

## Testing Checklist

- [ ] Upload CSV and verify existing unfollow flow still works
- [ ] Test account that doesn't exist (404) - should show "not_found" status
- [ ] Test account user isn't following - should show "not_following" status
- [ ] Test active account (posted recently) - should show "active" and be skipped
- [ ] Test inactive account - should unfollow and show "inactive" status
- [ ] Verify rate limiter only increments for actual unfollows
- [ ] Test inactivity threshold setting (1-24 months validation)
- [ ] Test feature request button opens GitHub issue page
- [ ] Test voice dictation in feature request modal
- [ ] Verify all new status icons display correctly with proper colors
