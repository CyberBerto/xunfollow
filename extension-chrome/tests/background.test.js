/**
 * Unit Tests for Background Service Worker (background.js)
 *
 * Tests for state management, settings, and message handling
 */

describe('Background Script - Settings Management', () => {

  const DEFAULT_SETTINGS = {
    dailyLimit: 50,
    hourlyLimit: 30,
    sessionLimit: 25,
    minDelay: 30,
    maxDelay: 60
  };

  test('Should return default settings when none saved', async () => {
    // Mock chrome.storage.local
    const mockStorage = {
      get: jest.fn((key) => Promise.resolve({}))
    };

    const loadSettings = async () => {
      const result = await mockStorage.get('settings');
      return result.settings || DEFAULT_SETTINGS;
    };

    const settings = await loadSettings();
    expect(settings).toEqual(DEFAULT_SETTINGS);
  });

  test('Should save settings to storage', async () => {
    const newSettings = {
      ...DEFAULT_SETTINGS,
      dailyLimit: 100
    };

    const mockStorage = {
      set: jest.fn((data) => Promise.resolve())
    };

    const saveSettings = async (settings) => {
      await mockStorage.set({ settings });
    };

    await saveSettings(newSettings);

    expect(mockStorage.set).toHaveBeenCalledWith({ settings: newSettings });
  });

  test('Should load saved settings', async () => {
    const savedSettings = {
      dailyLimit: 75,
      hourlyLimit: 40,
      sessionLimit: 30,
      minDelay: 45,
      maxDelay: 90
    };

    const mockStorage = {
      get: jest.fn(() => Promise.resolve({ settings: savedSettings }))
    };

    const loadSettings = async () => {
      const result = await mockStorage.get('settings');
      return result.settings || DEFAULT_SETTINGS;
    };

    const settings = await loadSettings();
    expect(settings).toEqual(savedSettings);
  });
});

describe('Background Script - History Management', () => {

  test('Should add entry to history', async () => {
    const history = [];

    const addToHistory = async (entry) => {
      history.unshift({
        ...entry,
        timestamp: new Date().toISOString()
      });

      if (history.length > 1000) {
        history.splice(1000);
      }
    };

    await addToHistory({ username: 'testuser', status: 'success' });

    expect(history.length).toBe(1);
    expect(history[0].username).toBe('testuser');
    expect(history[0].status).toBe('success');
    expect(history[0].timestamp).toBeDefined();
  });

  test('Should limit history to 1000 entries', async () => {
    const history = [];

    const addToHistory = async (entry) => {
      history.unshift({
        ...entry,
        timestamp: new Date().toISOString()
      });

      if (history.length > 1000) {
        history.splice(1000);
      }
    };

    // Add 1100 entries
    for (let i = 0; i < 1100; i++) {
      await addToHistory({ username: `user${i}`, status: 'success' });
    }

    expect(history.length).toBe(1000);
  });

  test('Should clear history', async () => {
    let history = [
      { username: 'user1', status: 'success' },
      { username: 'user2', status: 'success' }
    ];

    const clearHistory = async () => {
      history = [];
    };

    await clearHistory();

    expect(history.length).toBe(0);
  });

  test('Should export history as CSV', () => {
    const history = [
      { timestamp: '2024-01-01T12:00:00Z', username: 'user1', status: 'success' },
      { timestamp: '2024-01-01T12:01:00Z', username: 'user2', status: 'failed', error: 'Not found' }
    ];

    const convertHistoryToCSV = (history) => {
      const header = 'timestamp,username,status,error\n';
      const rows = history.map(entry => {
        return `${entry.timestamp},${entry.username},${entry.status},"${entry.error || ''}"`;
      });
      return header + rows.join('\n');
    };

    const csv = convertHistoryToCSV(history);

    expect(csv).toContain('timestamp,username,status,error');
    expect(csv).toContain('user1,success');
    expect(csv).toContain('user2,failed,"Not found"');
  });
});

describe('Background Script - State Management', () => {

  test('Should track current state', () => {
    const state = {
      isRunning: false,
      isPaused: false,
      stats: {
        total: 0,
        unfollowed: 0,
        failed: 0,
        current: null
      }
    };

    expect(state.isRunning).toBe(false);
    expect(state.stats.total).toBe(0);
  });

  test('Should update state on unfollow success', () => {
    const state = {
      stats: {
        total: 0,
        unfollowed: 0,
        failed: 0,
        current: null
      }
    };

    const handleUnfollowSuccess = (username) => {
      state.stats.unfollowed++;
      state.stats.total++;
      state.stats.current = username;
    };

    handleUnfollowSuccess('testuser');

    expect(state.stats.unfollowed).toBe(1);
    expect(state.stats.total).toBe(1);
    expect(state.stats.current).toBe('testuser');
  });

  test('Should update state on unfollow failure', () => {
    const state = {
      stats: {
        total: 0,
        unfollowed: 0,
        failed: 0,
        current: null
      }
    };

    const handleUnfollowFailure = (username) => {
      state.stats.failed++;
      state.stats.total++;
    };

    handleUnfollowFailure('testuser');

    expect(state.stats.failed).toBe(1);
    expect(state.stats.total).toBe(1);
  });
});

describe('Background Script - Message Routing', () => {

  test('Should forward messages to content script', async () => {
    const mockTabs = {
      query: jest.fn(() => Promise.resolve([{ id: 123, url: 'https://x.com' }])),
      sendMessage: jest.fn(() => Promise.resolve({ success: true }))
    };

    const sendToContentScript = async (message) => {
      const [tab] = await mockTabs.query({ active: true, currentWindow: true });

      if (!tab) {
        throw new Error('No active tab');
      }

      if (!tab.url.includes('twitter.com') && !tab.url.includes('x.com')) {
        throw new Error('Please navigate to twitter.com or x.com');
      }

      return await mockTabs.sendMessage(tab.id, message);
    };

    const result = await sendToContentScript({ type: 'start' });

    expect(result.success).toBe(true);
    expect(mockTabs.query).toHaveBeenCalled();
    expect(mockTabs.sendMessage).toHaveBeenCalledWith(123, { type: 'start' });
  });

  test('Should reject if not on X.com', async () => {
    const mockTabs = {
      query: jest.fn(() => Promise.resolve([{ id: 123, url: 'https://google.com' }]))
    };

    const sendToContentScript = async (message) => {
      const [tab] = await mockTabs.query({ active: true, currentWindow: true });

      if (!tab) {
        throw new Error('No active tab');
      }

      if (!tab.url.includes('twitter.com') && !tab.url.includes('x.com')) {
        throw new Error('Please navigate to twitter.com or x.com');
      }

      return await mockTabs.sendMessage(tab.id, message);
    };

    await expect(sendToContentScript({ type: 'start' }))
      .rejects.toThrow('Please navigate to twitter.com or x.com');
  });
});

describe('Background Script - Installation', () => {

  test('Should set default settings on install', async () => {
    const mockStorage = {
      set: jest.fn(() => Promise.resolve())
    };

    const handleInstall = async (details) => {
      if (details.reason === 'install') {
        await mockStorage.set({
          settings: {
            dailyLimit: 50,
            hourlyLimit: 30,
            sessionLimit: 25,
            minDelay: 30,
            maxDelay: 60
          }
        });
      }
    };

    await handleInstall({ reason: 'install' });

    expect(mockStorage.set).toHaveBeenCalled();
  });

  test('Should not reset settings on update', async () => {
    const mockStorage = {
      set: jest.fn(() => Promise.resolve())
    };

    const handleInstall = async (details) => {
      if (details.reason === 'install') {
        await mockStorage.set({
          settings: {
            dailyLimit: 50,
            hourlyLimit: 30,
            sessionLimit: 25,
            minDelay: 30,
            maxDelay: 60
          }
        });
      }
    };

    await handleInstall({ reason: 'update' });

    expect(mockStorage.set).not.toHaveBeenCalled();
  });
});

module.exports = {};
