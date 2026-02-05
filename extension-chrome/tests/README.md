# XUnfollow Chrome Extension - Test Suite

This directory contains unit and integration tests for the Chrome Extension.

## Test Files

- **`content.test.js`** - Tests for content script (X.com automation)
- **`background.test.js`** - Tests for background service worker
- **`manual-tests.md`** - Manual testing checklist

## Running Tests

### Option 1: Jest (Recommended)

```bash
# Install Jest
npm install --save-dev jest @types/jest

# Run all tests
npm test

# Run specific test file
npm test content.test.js

# Run with coverage
npm test -- --coverage
```

### Option 2: Browser Console

1. Load the extension in Chrome
2. Open DevTools (F12)
3. Go to Sources tab
4. Open test files
5. Run tests manually in console

### Option 3: Manual Testing

See `manual-tests.md` for manual testing checklist.

## Test Structure

Each test file follows this structure:

```javascript
describe('Component Name - Feature', () => {
  test('Should do something specific', () => {
    // Arrange
    const input = setupTestData();

    // Act
    const result = functionUnderTest(input);

    // Assert
    expect(result).toBe(expected);
  });
});
```

## Writing New Tests

When adding new features:

1. Write tests first (TDD approach)
2. Follow existing test structure
3. Test both happy path and error cases
4. Mock Chrome APIs appropriately
5. Clean up after tests

## Test Coverage Goals

- **Content Script**: >80% coverage
- **Background Script**: >80% coverage
- **Popup Logic**: >70% coverage

## Continuous Integration

Tests should be run:
- Before commits
- In pull requests
- Before releases

## Known Limitations

- Chrome API mocking is simplified
- DOM manipulation tests require browser environment
- Integration tests need live X.com pages

## Future Improvements

- [ ] Add E2E tests with Puppeteer
- [ ] Add visual regression tests
- [ ] Add performance benchmarks
- [ ] Add accessibility tests
- [ ] Automate test runs in CI/CD
