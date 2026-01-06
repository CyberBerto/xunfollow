(async function unfollowUser() {
    const USERNAME = '__USERNAME__';
    const SELECTORS = {
        followButton: '[data-testid$="-unfollow"], [data-testid$="-follow"]',
        confirmButton: '[data-testid="confirmationSheetConfirm"]',
        notFound: '[data-testid="empty_state_header_text"]',
        errorPage: '[data-testid="error-detail"]'
    };
    const TIMEOUT = 10000;

    function wait(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    function waitForElement(selector, timeout = TIMEOUT) {
        return new Promise((resolve, reject) => {
            const element = document.querySelector(selector);
            if (element) return resolve(element);

            const observer = new MutationObserver(() => {
                const el = document.querySelector(selector);
                if (el) {
                    observer.disconnect();
                    resolve(el);
                }
            });

            observer.observe(document.body, { childList: true, subtree: true });
            setTimeout(() => {
                observer.disconnect();
                reject(new Error('Element not found: ' + selector));
            }, timeout);
        });
    }

    async function tryCSS() {
        try {
            await wait(2000);

            // Check if user exists (not 404 or error page)
            const notFound = document.querySelector(SELECTORS.notFound);
            const errorPage = document.querySelector(SELECTORS.errorPage);
            if (notFound || errorPage) {
                return { success: false, error: 'user_not_found' };
            }

            // Find Following button (normal accounts)
            let followBtn = document.querySelector(SELECTORS.followButton);

            // Check if this is a subscription account (has Subscribe button + small unfollow icon)
            // The icon button has aria-label like "Unfollow @username"
            if (!followBtn || !(followBtn.textContent || '').toLowerCase().includes('following')) {
                // Try to find the unfollow icon button for subscription accounts
                const unfollowIcon = document.querySelector('button[aria-label^="Unfollow @"], button[aria-label^="unfollow @"]');
                if (unfollowIcon) {
                    // Hover over the icon to reveal the dropdown menu
                    unfollowIcon.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
                    unfollowIcon.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
                    await wait(300);

                    // Click the icon to open dropdown/popover
                    unfollowIcon.click();
                    await wait(500);

                    // Look for "Unfollow" option in dropdown menu
                    const menuItems = document.querySelectorAll('[role="menuitem"], [role="menu"] button, [data-testid="Dropdown"] span, div[data-testid] span');
                    let unfollowMenuItem = null;
                    for (const item of menuItems) {
                        const text = (item.textContent || '').toLowerCase();
                        if (text.includes('unfollow')) {
                            unfollowMenuItem = item.closest('div[role="menuitem"]') || item.closest('button') || item;
                            break;
                        }
                    }

                    if (unfollowMenuItem) {
                        unfollowMenuItem.click();
                        await wait(500);
                    }

                    // Check for confirmation dialog
                    const confirmBtn = document.querySelector(SELECTORS.confirmButton);
                    if (confirmBtn) {
                        confirmBtn.click();
                        await wait(500);
                    }

                    return { success: true, method: 'icon' };
                }
            }

            if (!followBtn) {
                return { success: false, error: 'follow_button_not_found' };
            }

            // Check if we're actually following (button shows "Following")
            const buttonText = followBtn.textContent || followBtn.innerText || '';
            if (!buttonText.toLowerCase().includes('following')) {
                return { success: false, error: 'not_following' };
            }

            // Click Following button to open the unfollow dialog
            followBtn.click();
            await wait(500);

            // Wait for and click confirm button
            const confirmBtn = await waitForElement(SELECTORS.confirmButton);
            confirmBtn.click();
            await wait(500);

            return { success: true, method: 'css' };
        } catch (e) {
            return { success: false, error: e.message, method: 'css' };
        }
    }

    async function tryText() {
        try {
            await wait(2000);

            // Find all buttons
            const buttons = Array.from(document.querySelectorAll('button'));

            // Find "Following" button by text or aria-label
            const followBtn = buttons.find(btn => {
                const text = (btn.textContent || btn.innerText || '').toLowerCase();
                const ariaLabel = (btn.getAttribute('aria-label') || '').toLowerCase();
                return text.includes('following') || ariaLabel.includes('following');
            });

            if (!followBtn) {
                return { success: false, error: 'following_button_not_found' };
            }

            followBtn.click();
            await wait(500);

            // Find "Unfollow" confirm button
            const allButtons = Array.from(document.querySelectorAll('button'));
            const confirmBtn = allButtons.find(btn => {
                const text = (btn.textContent || btn.innerText || '').toLowerCase();
                return text.includes('unfollow');
            });

            if (!confirmBtn) {
                return { success: false, error: 'confirm_button_not_found' };
            }

            confirmBtn.click();
            await wait(500);

            return { success: true, method: 'text' };
        } catch (e) {
            return { success: false, error: e.message, method: 'text' };
        }
    }

    // Execute with fallback
    let result = await tryCSS();

    if (!result.success && result.error !== 'user_not_found' && result.error !== 'not_following') {
        result = await tryText();
    }

    // Log result for debugging (visible in browser console)
    console.log('XUnfollow result:', JSON.stringify(result));

    return JSON.stringify(result);
})();
