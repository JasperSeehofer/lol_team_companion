import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
// navigateTo waits for the page to hydrate: a click before it is lost.
import { navigateTo } from './helpers';

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

// M141 (bug reports xvqg64mb, 12wea0hr, 96mhp4u3): the seed tool's fictional
// demo game, opened as the seed player who "played" it. CI seeds the database
// before the server starts; locally the spec is skipped without the seed.
const DEMO = '/match/SEED1_1000000001';

async function signInSeedTop(page: Page): Promise<boolean> {
    await page.goto('/auth/login');
    await page.fill('input[name=email]', 'seed-alpha-top@example.com');
    await page.fill('input[name=password]', 'seedpass123');
    await page.click('main button[type=submit]');
    return page
        .waitForURL((u) => new URL(u).pathname === '/', { timeout: 15000 })
        .then(() => true)
        .catch(() => false);
}

test.describe('match detail (seeded demo game)', () => {
    test.beforeEach(async ({ page }) => {
        const ok = await signInSeedTop(page);
        if (process.env.CI) expect(ok, 'CI must seed the database before starting the server').toBe(true);
        test.skip(!ok, 'needs the seed data: stop the server, run the seed tool, start it again');
        await page.waitForTimeout(500);
    });

    for (const width of [390, 1280, 1536]) {
        test(`both scoreboards have the same box and never scroll at ${width}`, async ({ page }) => {
            await page.setViewportSize({ width, height: 900 });
            await navigateTo(page, DEMO);
            const blue = page.locator('[data-scoreboard=blue] table');
            const red = page.locator('[data-scoreboard=red] table');
            await expect(blue).toBeVisible();
            await expect(red).toBeVisible();
            const b = await blue.boundingBox();
            const r = await red.boundingBox();
            expect(b && r).toBeTruthy();
            expect(Math.abs(b!.width - r!.width)).toBeLessThanOrEqual(1);
            expect(Math.abs(b!.height - r!.height)).toBeLessThanOrEqual(1);
            await expect(page.locator('[data-scoreboard] tbody tr')).toHaveCount(10);
            // No scroll region around the tables, and no page-wide horizontal scroll.
            await expect(page.locator('[data-scoreboard] [role=region]')).toHaveCount(0);
            const overflow = await page.evaluate(() => {
                const doc = document.documentElement;
                const tables = Array.from(document.querySelectorAll('[data-scoreboard] table'));
                return {
                    page: doc.scrollWidth - doc.clientWidth,
                    tables: tables.map((t) => (t.parentElement!.scrollWidth - t.parentElement!.clientWidth)),
                };
            });
            expect(overflow.page).toBeLessThanOrEqual(0);
            for (const o of overflow.tables) expect(o).toBeLessThanOrEqual(0);
            // Hovering a row shows no scrollbar either.
            await page.locator('[data-scoreboard=blue] tbody tr').nth(2).hover();
            const after = await page.evaluate(() =>
                Array.from(document.querySelectorAll('[data-scoreboard] table')).map(
                    (t) => t.parentElement!.scrollWidth - t.parentElement!.clientWidth,
                ),
            );
            for (const o of after) expect(o).toBeLessThanOrEqual(0);
        });
    }

    test('hovering a row highlights the same role in both teams; a tap pins it', async ({ page }) => {
        await page.setViewportSize({ width: 1280, height: 900 });
        await navigateTo(page, DEMO);
        const blueMid = page.locator('[data-scoreboard=blue] tbody tr').nth(2);
        const redMid = page.locator('[data-scoreboard=red] tbody tr').nth(2);
        await blueMid.hover();
        await expect(blueMid).toHaveClass(/bg-ink-3/);
        await expect(redMid).toHaveClass(/bg-ink-3/);
        await page.mouse.move(0, 0);
        await page.locator('h1').hover();
        await expect(redMid).not.toHaveClass(/bg-ink-3/);
        // Tap / click equivalent: the position button pins the pair.
        const pin = page.locator('[data-scoreboard=red]').getByRole('button', { name: 'Highlight Mid in both teams' });
        await pin.click();
        await expect(pin).toHaveAttribute('aria-pressed', 'true');
        await page.locator('h1').hover();
        await expect(blueMid).toHaveClass(/bg-ink-3/);
        await page.keyboard.press('Escape');
        await expect(pin).toHaveAttribute('aria-pressed', 'false');
    });

    test('the player is marked by an accent, not "(you)"; K/D/A on one line', async ({ page }) => {
        await page.setViewportSize({ width: 390, height: 844 });
        await navigateTo(page, DEMO);
        await expect(page.locator('[data-scoreboard=blue] table')).toBeVisible();
        await expect(page.locator('main')).not.toContainText('(you)');
        await expect(page.locator('[data-you-accent]')).toHaveCount(1);
        await expect(page.locator('tr[aria-current=true]')).toHaveCount(1);
        const lines = await page.locator('[data-kda]').evaluateAll((els) =>
            els.map((el) => el.getClientRects().length),
        );
        expect(lines.length).toBe(10);
        for (const n of lines) expect(n).toBe(1);
    });

    test('the timeline has three rails with toggles and draws no recall', async ({ page }) => {
        await page.setViewportSize({ width: 1280, height: 900 });
        await navigateTo(page, DEMO);
        await expect(page.locator('[data-rail]')).toHaveCount(3);
        // The demo stores an ITEM_UNDO row: it must not be drawn as a recall.
        await expect(page.locator('[data-event-category=Recall]')).toHaveCount(0);
        const fights = page.locator('[data-rail=fights]');
        await expect(fights.locator('[data-event-category=Kill]').first()).toBeVisible();
        const objectives = page.locator('[data-rail=objectives] [data-event-category]');
        const objectiveCount = await objectives.count();
        expect(objectiveCount).toBeGreaterThan(0);
        // A kind toggle hides only its own kind.
        await fights.getByRole('button', { name: 'Kill', exact: true }).click();
        await expect(fights.locator('[data-event-category=Kill]')).toHaveCount(0);
        await expect(objectives).toHaveCount(objectiveCount);
        // The rail toggle hides and shows the whole rail.
        const railToggle = page.getByRole('button', { name: 'Show or hide all of Objectives' });
        await railToggle.click();
        await expect(objectives).toHaveCount(0);
        await railToggle.click();
        await expect(objectives).toHaveCount(objectiveCount);
        // Selecting a marker moves the shared playhead onto it.
        await objectives.first().click();
        await expect(page.locator('[data-timeline-detail]')).toBeVisible();
        await expect(page.locator('[data-near-playhead]')).toContainText('Near');
        await expect(page.locator('[data-playhead]')).toHaveCount(3);
    });
});
