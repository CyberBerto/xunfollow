// Popup script for Chrome extension
console.log('[XUnfollow Popup] Loaded');

// State
let currentState = {
  isRunning: false,
  isPaused: false,
  stats: {
    total: 0,
    unfollowed: 0,
    failed: 0,
    current: null
  }
};

let settings = {
  dailyLimit: 50,
  hourlyLimit: 30,
  sessionLimit: 25,
  minDelay: 30,
  maxDelay: 60
};

// DOM Elements
const elements = {
  // Status
  statusCard: document.getElementById('statusCard'),
  statusIndicator: document.getElementById('statusIndicator'),
  statusText: document.getElementById('statusText'),

  // Stats
  unfollowedCount: document.getElementById('unfollowedCount'),
  failedCount: document.getElementById('failedCount'),
  totalCount: document.getElementById('totalCount'),

  // Current user
  currentUser: document.getElementById('currentUser'),
  currentUsername: document.getElementById('currentUsername'),

  // Controls
  startBtn: document.getElementById('startBtn'),
  pauseBtn: document.getElementById('pauseBtn'),
  resumeBtn: document.getElementById('resumeBtn'),
  stopBtn: document.getElementById('stopBtn'),

  // Quick settings
  sessionLimit: document.getElementById('sessionLimit'),
  minDelay: document.getElementById('minDelay'),
  maxDelay: document.getElementById('maxDelay'),

  // Log
  logContainer: document.getElementById('logContainer'),
  clearLogBtn: document.getElementById('clearLogBtn'),

  // Footer
  exportBtn: document.getElementById('exportBtn'),
  helpBtn: document.getElementById('helpBtn'),

  // Settings modal
  settingsBtn: document.getElementById('settingsBtn'),
  settingsModal: document.getElementById('settingsModal'),
  closeSettingsBtn: document.getElementById('closeSettingsBtn'),
  cancelSettingsBtn: document.getElementById('cancelSettingsBtn'),
  saveSettingsBtn: document.getElementById('saveSettingsBtn'),
  dailyLimit: document.getElementById('dailyLimit'),
  hourlyLimit: document.getElementById('hourlyLimit'),
  settingsSessionLimit: document.getElementById('settingsSessionLimit'),
  settingsMinDelay: document.getElementById('settingsMinDelay'),
  settingsMaxDelay: document.getElementById('settingsMaxDelay')
};

/**
 * Send message to background script
 */
async function sendMessage(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(message, (response) => {
      if (chrome.runtime.lastError) {
        reject(chrome.runtime.lastError);
      } else {
        resolve(response);
      }
    });
  });
}

/**
 * Update UI based on current state
 */
function updateUI() {
  // Update status
  if (currentState.isRunning) {
    if (currentState.isPaused) {
      setStatus('paused', 'Paused');
      showButtons(['resumeBtn', 'stopBtn']);
    } else {
      setStatus('running', 'Running...');
      showButtons(['pauseBtn', 'stopBtn']);
    }
  } else {
    setStatus('idle', 'Ready');
    showButtons(['startBtn']);
  }

  // Update stats
  elements.unfollowedCount.textContent = currentState.stats.unfollowed || 0;
  elements.failedCount.textContent = currentState.stats.failed || 0;
  elements.totalCount.textContent = currentState.stats.total || 0;

  // Update current user
  if (currentState.stats.current) {
    elements.currentUser.style.display = 'block';
    elements.currentUsername.textContent = '@' + currentState.stats.current;
  } else {
    elements.currentUser.style.display = 'none';
  }
}

/**
 * Set status indicator
 */
function setStatus(state, text) {
  elements.statusIndicator.className = `status-indicator ${state}`;
  elements.statusText.textContent = text;
}

/**
 * Show specific buttons
 */
function showButtons(buttonsToShow) {
  const allButtons = ['startBtn', 'pauseBtn', 'resumeBtn', 'stopBtn'];
  allButtons.forEach(btnId => {
    elements[btnId].style.display = buttonsToShow.includes(btnId) ? 'block' : 'none';
  });
}

/**
 * Add log entry
 */
function addLog(message, type = 'info') {
  // Remove empty message if present
  const empty = elements.logContainer.querySelector('.log-empty');
  if (empty) empty.remove();

  const logEntry = document.createElement('div');
  logEntry.className = `log-entry ${type}`;

  const time = document.createElement('span');
  time.className = 'time';
  time.textContent = new Date().toLocaleTimeString();

  const msg = document.createElement('span');
  msg.className = 'message';
  msg.textContent = message;

  logEntry.appendChild(time);
  logEntry.appendChild(msg);

  elements.logContainer.insertBefore(logEntry, elements.logContainer.firstChild);

  // Keep only last 50 entries
  while (elements.logContainer.children.length > 50) {
    elements.logContainer.removeChild(elements.logContainer.lastChild);
  }
}

/**
 * Clear log
 */
function clearLog() {
  elements.logContainer.innerHTML = '<div class="log-empty">No activity yet</div>';
}

/**
 * Load settings from background
 */
async function loadSettings() {
  try {
    settings = await sendMessage({ type: 'get_settings' });

    // Update UI
    elements.sessionLimit.value = settings.sessionLimit;
    elements.minDelay.value = settings.minDelay;
    elements.maxDelay.value = settings.maxDelay;

    // Update modal
    elements.dailyLimit.value = settings.dailyLimit;
    elements.hourlyLimit.value = settings.hourlyLimit;
    elements.settingsSessionLimit.value = settings.sessionLimit;
    elements.settingsMinDelay.value = settings.minDelay;
    elements.settingsMaxDelay.value = settings.maxDelay;
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

/**
 * Save settings to background
 */
async function saveSettings() {
  // Get values from modal
  settings = {
    dailyLimit: parseInt(elements.dailyLimit.value),
    hourlyLimit: parseInt(elements.hourlyLimit.value),
    sessionLimit: parseInt(elements.settingsSessionLimit.value),
    minDelay: parseInt(elements.settingsMinDelay.value),
    maxDelay: parseInt(elements.settingsMaxDelay.value)
  };

  try {
    await sendMessage({
      type: 'save_settings',
      settings
    });

    // Update quick settings
    elements.sessionLimit.value = settings.sessionLimit;
    elements.minDelay.value = settings.minDelay;
    elements.maxDelay.value = settings.maxDelay;

    addLog('Settings saved', 'success');
  } catch (error) {
    console.error('Failed to save settings:', error);
    addLog('Failed to save settings', 'error');
  }
}

/**
 * Load state from background
 */
async function loadState() {
  try {
    currentState = await sendMessage({ type: 'get_state' });
    updateUI();
  } catch (error) {
    console.error('Failed to load state:', error);
  }
}

/**
 * Start unfollowing
 */
async function start() {
  // Get current settings from quick settings
  settings.sessionLimit = parseInt(elements.sessionLimit.value);
  settings.minDelay = parseInt(elements.minDelay.value);
  settings.maxDelay = parseInt(elements.maxDelay.value);

  try {
    await sendMessage({ type: 'start', settings });
    addLog('Started unfollowing', 'success');
  } catch (error) {
    addLog('Failed to start: ' + error.message, 'error');
  }
}

/**
 * Pause unfollowing
 */
async function pause() {
  try {
    await sendMessage({ type: 'pause' });
    addLog('Paused', 'info');
  } catch (error) {
    addLog('Failed to pause: ' + error.message, 'error');
  }
}

/**
 * Resume unfollowing
 */
async function resume() {
  try {
    await sendMessage({ type: 'resume' });
    addLog('Resumed', 'success');
  } catch (error) {
    addLog('Failed to resume: ' + error.message, 'error');
  }
}

/**
 * Stop unfollowing
 */
async function stop() {
  try {
    await sendMessage({ type: 'stop' });
    addLog('Stopped', 'info');
  } catch (error) {
    addLog('Failed to stop: ' + error.message, 'error');
  }
}

/**
 * Export history
 */
async function exportHistory() {
  try {
    const response = await sendMessage({ type: 'export_history' });
    const csv = response.csv;

    // Download CSV file
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `xunfollow-history-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);

    addLog('History exported', 'success');
  } catch (error) {
    addLog('Failed to export: ' + error.message, 'error');
  }
}

/**
 * Show help
 */
function showHelp() {
  const helpText = `
X Unfollow Helper - How to Use:

1. Navigate to x.com or twitter.com
2. Go to your Following list
3. Click "Start" in the popup
4. The extension will automatically unfollow users

Settings:
- Session Limit: Max unfollows per session
- Delay: Random delay between unfollows (seconds)
- Daily/Hourly limits: For safe rate limiting

Tips:
- Use longer delays to be safer
- Don't set limits too high
- Extension respects X's rate limits
  `;

  alert(helpText.trim());
}

// Event listeners
elements.startBtn.addEventListener('click', start);
elements.pauseBtn.addEventListener('click', pause);
elements.resumeBtn.addEventListener('click', resume);
elements.stopBtn.addEventListener('click', stop);
elements.clearLogBtn.addEventListener('click', clearLog);
elements.exportBtn.addEventListener('click', exportHistory);
elements.helpBtn.addEventListener('click', showHelp);

// Settings modal
elements.settingsBtn.addEventListener('click', () => {
  elements.settingsModal.style.display = 'flex';
});

elements.closeSettingsBtn.addEventListener('click', () => {
  elements.settingsModal.style.display = 'none';
});

elements.cancelSettingsBtn.addEventListener('click', () => {
  elements.settingsModal.style.display = 'none';
});

elements.saveSettingsBtn.addEventListener('click', async () => {
  await saveSettings();
  elements.settingsModal.style.display = 'none';
});

// Listen for messages from background/content script
chrome.runtime.onMessage.addListener((message) => {
  console.log('[XUnfollow Popup] Received message:', message.type);

  switch (message.type) {
    case 'unfollow_success':
      addLog(`✓ Unfollowed @${message.username}`, 'success');
      currentState.stats = message.stats;
      updateUI();
      break;

    case 'unfollow_failed':
      addLog(`✗ Failed: @${message.username} - ${message.error}`, 'error');
      currentState.stats = message.stats;
      updateUI();
      break;

    case 'status_update':
      currentState.isRunning = message.status === 'running' || message.status === 'waiting';
      if (message.message) {
        addLog(message.message, 'info');
      }
      if (message.stats) {
        currentState.stats = message.stats;
      }
      updateUI();
      break;
  }
});

// Initialize on load
(async () => {
  await loadSettings();
  await loadState();
  addLog('Extension ready', 'info');
})();
