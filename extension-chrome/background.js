// Background service worker for Chrome extension
console.log('[XUnfollow Background] Service worker initialized');

// Default settings
const DEFAULT_SETTINGS = {
  dailyLimit: 50,
  hourlyLimit: 30,
  sessionLimit: 25,
  minDelay: 30,
  maxDelay: 60
};

// State
let currentState = {
  isRunning: false,
  isPaused: false,
  stats: {
    total: 0,
    unfollowed: 0,
    failed: 0,
    current: null
  },
  history: []
};

/**
 * Load settings from storage
 */
async function loadSettings() {
  const result = await chrome.storage.local.get('settings');
  return result.settings || DEFAULT_SETTINGS;
}

/**
 * Save settings to storage
 */
async function saveSettings(settings) {
  await chrome.storage.local.set({ settings });
}

/**
 * Load state from storage
 */
async function loadState() {
  const result = await chrome.storage.local.get('state');
  if (result.state) {
    currentState = result.state;
  }
}

/**
 * Save state to storage
 */
async function saveState() {
  await chrome.storage.local.set({ state: currentState });
}

/**
 * Get history
 */
async function getHistory() {
  const result = await chrome.storage.local.get('history');
  return result.history || [];
}

/**
 * Add to history
 */
async function addToHistory(entry) {
  const history = await getHistory();
  history.unshift({
    ...entry,
    timestamp: new Date().toISOString()
  });

  // Keep last 1000 entries
  if (history.length > 1000) {
    history.splice(1000);
  }

  await chrome.storage.local.set({ history });
  currentState.history = history;
}

/**
 * Clear history
 */
async function clearHistory() {
  await chrome.storage.local.set({ history: [] });
  currentState.history = [];
}

/**
 * Send message to active tab's content script
 */
async function sendToContentScript(message) {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

  if (!tab) {
    throw new Error('No active tab');
  }

  if (!tab.url.includes('twitter.com') && !tab.url.includes('x.com')) {
    throw new Error('Please navigate to twitter.com or x.com');
  }

  return chrome.tabs.sendMessage(tab.id, message);
}

/**
 * Handle messages from content script and popup
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('[XUnfollow Background] Received message:', message.type);

  // Handle async operations
  (async () => {
    try {
      switch (message.type) {
        case 'start':
          const settings = await loadSettings();
          const response = await sendToContentScript({
            type: 'start',
            settings
          });
          currentState.isRunning = true;
          currentState.isPaused = false;
          await saveState();
          sendResponse(response);
          break;

        case 'pause':
          await sendToContentScript({ type: 'pause' });
          currentState.isPaused = true;
          await saveState();
          sendResponse({ success: true });
          break;

        case 'resume':
          await sendToContentScript({ type: 'resume' });
          currentState.isPaused = false;
          await saveState();
          sendResponse({ success: true });
          break;

        case 'stop':
          await sendToContentScript({ type: 'stop' });
          currentState.isRunning = false;
          currentState.isPaused = false;
          await saveState();
          sendResponse({ success: true });
          break;

        case 'get_settings':
          const loadedSettings = await loadSettings();
          sendResponse(loadedSettings);
          break;

        case 'save_settings':
          await saveSettings(message.settings);
          sendResponse({ success: true });
          break;

        case 'get_state':
          sendResponse(currentState);
          break;

        case 'get_history':
          const history = await getHistory();
          sendResponse(history);
          break;

        case 'clear_history':
          await clearHistory();
          sendResponse({ success: true });
          break;

        case 'export_history':
          const exportHistory = await getHistory();
          const csv = convertHistoryToCSV(exportHistory);
          sendResponse({ csv });
          break;

        // Messages from content script
        case 'unfollow_success':
          await addToHistory({
            username: message.username,
            status: 'success'
          });
          currentState.stats = message.stats;
          await saveState();
          // Forward to popup if open
          chrome.runtime.sendMessage(message).catch(() => {});
          sendResponse({ success: true });
          break;

        case 'unfollow_failed':
          await addToHistory({
            username: message.username,
            status: 'failed',
            error: message.error
          });
          currentState.stats = message.stats;
          await saveState();
          // Forward to popup
          chrome.runtime.sendMessage(message).catch(() => {});
          sendResponse({ success: true });
          break;

        case 'status_update':
          currentState.isRunning = message.status === 'running' || message.status === 'waiting';
          currentState.stats = message.stats || currentState.stats;
          await saveState();
          // Forward to popup
          chrome.runtime.sendMessage(message).catch(() => {});
          sendResponse({ success: true });
          break;

        case 'content_ready':
          console.log('[XUnfollow Background] Content script ready on:', message.url);
          sendResponse({ success: true });
          break;

        default:
          sendResponse({ success: false, error: 'Unknown message type' });
      }
    } catch (error) {
      console.error('[XUnfollow Background] Error handling message:', error);
      sendResponse({ success: false, error: error.message });
    }
  })();

  return true; // Keep channel open for async response
});

/**
 * Convert history to CSV format
 */
function convertHistoryToCSV(history) {
  const header = 'timestamp,username,status,error\n';
  const rows = history.map(entry => {
    return `${entry.timestamp},${entry.username},${entry.status},"${entry.error || ''}"`;
  });
  return header + rows.join('\n');
}

/**
 * Handle extension installation
 */
chrome.runtime.onInstalled.addListener((details) => {
  console.log('[XUnfollow Background] Extension installed:', details.reason);

  if (details.reason === 'install') {
    // Set default settings
    saveSettings(DEFAULT_SETTINGS);

    // Open welcome page (optional)
    // chrome.tabs.create({ url: 'welcome.html' });
  }
});

// Load state on startup
loadState();
