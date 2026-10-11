import { test, expect } from './fixtures';

test.describe('match detail page', () => {
    test('renders match detail page without errors', async ({ authedPage }) => {
        const page = authedPage;
        // A match ID that does not exist: the route must render (with an
        // error state), not crash.
        await page.goto('/match/test-match-id');
        await expect(page.locator('body')).toBeVisible();
        // Verify no WASM panics (page should not be frozen)
        await page.waitForTimeout(1000);
        await expect(page.getByRole('navigation', { name: 'Primary' })).toBeVisible();
    });
});
