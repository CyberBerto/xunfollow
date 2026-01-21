// Content script that runs on X.com
console.log('[XUnfollow] Content script loaded');

// Selectors for X.com UI elements
const SELECTORS = {
  // Following button on profile
  followingButton: [
    'button[data-testid$="-unfollow"]',
    'div[data-testid$="-unfollow"]',
    'button:has-text("Following")',
    'div[role="button"]:has-text("Following")'
  ],
  // Confirmation modal button
  confirmButton: 'button[data-testid="confirmationSheetConfirm"]',
  // User cards on following page
  userCell: 'div[data-testid="UserCell"]',
  // Following list items
  followingItem: 'div[data-testid="UserCell"]',
};

// State
let isRunning = false;
let isPaused = false;
let stats = {
  total: 0,
  unfollowed: 0,
  failed: 0,
  current: null
};

/**
 * Delay helper
 */
function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Random delay between min and max seconds
 */
function randomDelay(minSec = 30, maxSec = 60) {
  const ms = (minSec + Math.random() * (maxSec - minSec)) * 1000;
  return delay(ms);
}

/**
 * Find following buttons on the current page
 */
function findFollowingButtons() {
  const buttons = [];

  // Try each selector
  for (const selector of SELECTORS.followingButton) {
    try {
      const found = document.querySelectorAll(selector);
      buttons.push(...found);
    } catch (e) {
      // Selector might not be supported (like :has-text)
    }
  }

  // Filter to buttons that actually say "Following"
  return Array.from(buttons).filter(btn => {
    const text = btn.textContent || btn.innerText;
    return text.includes('Following');
  });
}

/**
 * Find and click the confirmation button in the unfollow modal
 */
async function clickConfirmButton() {
  await delay(500); // Wait for modal to appear

  const confirmBtn = document.querySelector(SELECTORS.confirmButton);
  if (confirmBtn) {
    confirmBtn.click();
    return true;
  }

  // Fallback: look for button with "Unfollow" text
  const buttons = document.querySelectorAll('button');
  for (const btn of buttons) {
    if (btn.textContent.includes('Unfollow') && btn.closest('[role="dialog"]')) {
      btn.click();
      return true;
    }
  }

  return false;
}

/**
 * Extract username from user cell
 */
function extractUsername(userCell) {
  try {
    // Try to find username in the user cell
    const links = userCell.querySelectorAll('a[href*="/"]');
    for (const link of links) {
      const href = link.getAttribute('href');
      const match = href.match(/^\/([^\/]+)$/);
      if (match) {
        return match[1];
      }
    }
  } catch (e) {
    console.error('[XUnfollow] Error extracting username:', e);
  }
  return null;
}

/**
 * Unfollow a single user
 */
async function unfollowUser(button, username = 'unknown') {
  try {
    console.log(`[XUnfollow] Unfollowing @${username}...`);

    // Click the "Following" button
    button.click();

    // Wait and click confirm
    const confirmed = await clickConfirmButton();

    if (!confirmed) {
      throw new Error('Could not find confirmation button');
    }

    await delay(1000); // Wait for unfollow to complete

    stats.unfollowed++;
    console.log(`[XUnfollow] ✓ Successfully unfollowed @${username}`);

    // Notify popup
    sendMessage({
      type: 'unfollow_success',
      username,
      stats
    });

    return { success: true, username };

  } catch (error) {
    stats.failed++;
    console.error(`[XUnfollow] ✗ Failed to unfollow @${username}:`, error);

    sendMessage({
      type: 'unfollow_failed',
      username,
      error: error.message,
      stats
    });

    return { success: false, username, error: error.message };
  }
}

/**
 * Main unfollow loop
 */
async function startUnfollowLoop(settings) {
  isRunning = true;
  isPaused = false;

  console.log('[XUnfollow] Starting unfollow loop with settings:', settings);

  const {
    dailyLimit = 50,
    sessionLimit = 25,
    minDelay = 30,
    maxDelay = 60
  } = settings;

  let sessionCount = 0;

  sendMessage({
    type: 'status_update',
    status: 'running',
    stats
  });

  while (isRunning && sessionCount < sessionLimit) {
    // Check if paused
    while (isPaused && isRunning) {
      await delay(1000);
    }

    if (!isRunning) break;

    // Find following buttons
    const buttons = findFollowingButtons();

    if (buttons.length === 0) {
      console.log('[XUnfollow] No more following buttons found');
      sendMessage({
        type: 'status_update',
        status: 'completed',
        message: 'No more users to unfollow on this page',
        stats
      });
      break;
    }

    console.log(`[XUnfollow] Found ${buttons.length} following buttons`);

    // Get the first button and its username
    const button = buttons[0];
    const userCell = button.closest(SELECTORS.userCell);
    const username = extractUsername(userCell) || 'unknown';

    stats.current = username;

    // Unfollow
    const result = await unfollowUser(button, username);

    sessionCount++;
    stats.total++;

    // Check session limit
    if (sessionCount >= sessionLimit) {
      console.log('[XUnfollow] Session limit reached');
      sendMessage({
        type: 'status_update',
        status: 'session_limit',
        message: `Session limit reached (${sessionLimit} unfollows)`,
        stats
      });
      break;
    }

    // Random delay before next unfollow
    if (isRunning) {
      const delayMs = (minDelay + Math.random() * (maxDelay - minDelay)) * 1000;
      const delaySec = Math.round(delayMs / 1000);

      console.log(`[XUnfollow] Waiting ${delaySec}s before next unfollow...`);

      sendMessage({
        type: 'status_update',
        status: 'waiting',
        message: `Waiting ${delaySec}s...`,
        stats
      });

      await delay(delayMs);
    }
  }

  isRunning = false;

  sendMessage({
    type: 'status_update',
    status: 'stopped',
    stats
  });

  console.log('[XUnfollow] Unfollow loop completed');
}

/**
 * Send message to popup/background
 */
function sendMessage(message) {
  chrome.runtime.sendMessage(message).catch(err => {
    console.log('[XUnfollow] Could not send message:', err);
  });
}

/**
 * Handle messages from popup/background
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('[XUnfollow] Received message:', message);

  switch (message.type) {
    case 'start':
      if (!isRunning) {
        startUnfollowLoop(message.settings);
        sendResponse({ success: true });
      } else {
        sendResponse({ success: false, error: 'Already running' });
      }
      break;

    case 'pause':
      isPaused = true;
      sendResponse({ success: true });
      break;

    case 'resume':
      isPaused = false;
      sendResponse({ success: true });
      break;

    case 'stop':
      isRunning = false;
      isPaused = false;
      sendResponse({ success: true });
      break;

    case 'get_status':
      sendResponse({
        isRunning,
        isPaused,
        stats
      });
      break;

    case 'get_following_count':
      const buttons = findFollowingButtons();
      sendResponse({ count: buttons.length });
      break;

    default:
      sendResponse({ success: false, error: 'Unknown message type' });
  }

  return true; // Keep channel open for async response
});

// Notify that content script is ready
sendMessage({
  type: 'content_ready',
  url: window.location.href
});
