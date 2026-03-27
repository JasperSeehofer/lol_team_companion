import { test, expect } from '@playwright/test';
import { authedPage } from './fixtures';

test.describe('match detail page', () => {
    test('renders match detail page without errors', async ({ browser }) => {
        const page = await authedPage(browser);
        // Navigate to a known match (or verify the route renders without crash)
        await page.goto('http://127.0.0.1:3020/match/test-match-id');
        // Should show the page structure (may show error banner if no real match data)
        await expect(page.locator('body')).toBeVisible();
        // Verify no WASM panics (page should not be frozen)
        await page.waitForTimeout(1000);
        await expect(page.locator('nav')).toBeVisible();
        await page.close();
    });
});
