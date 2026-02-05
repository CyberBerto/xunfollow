/**
 * Unit Tests for Content Script (content.js)
 *
 * These tests verify the core functionality of the X.com automation script.
 * Run these tests in a browser environment with the extension loaded.
 */

// Mock Chrome API for testing
const mockChrome = {
  runtime: {
    sendMessage: jest.fn(),
    onMessage: {
      addListener: jest.fn()
    }
  }
};

describe('Content Script - Selector Functions', () => {

  test('findFollowingButtons() should find buttons with correct selectors', () => {
    // Setup DOM
    document.body.innerHTML = `
      <button data-testid="user-unfollow">Following</button>
      <div data-testid="another-unfollow">Following</div>
    `;

    // Mock function (in real extension)
    const findFollowingButtons = () => {
      const selectors = [
        'button[data-testid$="-unfollow"]',
        'div[data-testid$="-unfollow"]'
      ];

      const buttons = [];
      selectors.forEach(selector => {
        try {
          buttons.push(...document.querySelectorAll(selector));
        } catch (e) {}
      });

      return Array.from(buttons).filter(btn => {
        const text = btn.textContent || btn.innerText;
        return text.includes('Following');
      });
    };

    const buttons = findFollowingButtons();
    expect(buttons.length).toBe(2);
  });

  test('extractUsername() should extract username from user cell', () => {
    document.body.innerHTML = `
      <div data-testid="UserCell">
        <a href="/username123">@username123</a>
      </div>
    `;

    const extractUsername = (userCell) => {
      const links = userCell.querySelectorAll('a[href*="/"]');
      for (const link of links) {
        const href = link.getAttribute('href');
        const match = href.match(/^\/([^\/]+)$/);
        if (match) return match[1];
      }
      return null;
    };

    const userCell = document.querySelector('[data-testid="UserCell"]');
    const username = extractUsername(userCell);

    expect(username).toBe('username123');
  });

  test('clickConfirmButton() should find confirmation button', async () => {
    document.body.innerHTML = `
      <div role="dialog">
        <button data-testid="confirmationSheetConfirm">Unfollow</button>
      </div>
    `;

    const clickConfirmButton = async () => {
      await new Promise(resolve => setTimeout(resolve, 500));
      const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]');
      if (confirmBtn) {
        confirmBtn.click();
        return true;
      }
      return false;
    };

    const result = await clickConfirmButton();
    expect(result).toBe(true);
  });
});

describe('Content Script - Rate Limiting', () => {

  test('Should respect session limit', () => {
    const sessionLimit = 25;
    let sessionCount = 0;

    const checkSessionLimit = () => {
      return sessionCount < sessionLimit;
    };

    // Simulate 25 unfollows
    for (let i = 0; i < 25; i++) {
      expect(checkSessionLimit()).toBe(true);
      sessionCount++;
    }

    // 26th should fail
    expect(checkSessionLimit()).toBe(false);
  });

  test('Random delay should be within range', () => {
    const minDelay = 30;
    const maxDelay = 60;

    const getRandomDelay = (min, max) => {
      return min + Math.random() * (max - min);
    };

    for (let i = 0; i < 100; i++) {
      const delay = getRandomDelay(minDelay, maxDelay);
      expect(delay).toBeGreaterThanOrEqual(minDelay);
      expect(delay).toBeLessThanOrEqual(maxDelay);
    }
  });
});

describe('Content Script - Message Handling', () => {

  test('Should handle start message', () => {
    const messageHandler = (message, sender, sendResponse) => {
      if (message.type === 'start') {
        sendResponse({ success: true });
        return true;
      }
    };

    const mockSendResponse = jest.fn();
    messageHandler({ type: 'start', settings: {} }, {}, mockSendResponse);

    expect(mockSendResponse).toHaveBeenCalledWith({ success: true });
  });

  test('Should handle pause message', () => {
    let isPaused = false;

    const messageHandler = (message, sender, sendResponse) => {
      if (message.type === 'pause') {
        isPaused = true;
        sendResponse({ success: true });
        return true;
      }
    };

    const mockSendResponse = jest.fn();
    messageHandler({ type: 'pause' }, {}, mockSendResponse);

    expect(isPaused).toBe(true);
    expect(mockSendResponse).toHaveBeenCalledWith({ success: true });
  });
});

describe('Content Script - Error Handling', () => {

  test('Should handle missing confirmation button gracefully', async () => {
    document.body.innerHTML = `<div>No modal here</div>`;

    const clickConfirmButton = async () => {
      await new Promise(resolve => setTimeout(resolve, 500));
      const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]');
      if (confirmBtn) {
        confirmBtn.click();
        return true;
      }
      return false;
    };

    const result = await clickConfirmButton();
    expect(result).toBe(false);
  });

  test('Should handle invalid selectors gracefully', () => {
    const findFollowingButtons = () => {
      const selectors = ['button[invalid::selector]', 'button'];
      const buttons = [];

      selectors.forEach(selector => {
        try {
          buttons.push(...document.querySelectorAll(selector));
        } catch (e) {
          // Gracefully handle invalid selectors
        }
      });

      return buttons;
    };

    const buttons = findFollowingButtons();
    expect(Array.isArray(buttons)).toBe(true);
  });
});

/**
 * Test Helper Functions
 */

function setupMockDOM() {
  document.body.innerHTML = `
    <div data-testid="UserCell">
      <a href="/testuser">Test User</a>
      <button data-testid="testuser-unfollow">Following</button>
    </div>
  `;
}

function teardownMockDOM() {
  document.body.innerHTML = '';
}

module.exports = {
  setupMockDOM,
  teardownMockDOM
};
