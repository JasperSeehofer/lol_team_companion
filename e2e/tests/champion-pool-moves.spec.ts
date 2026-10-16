/**
 * M141 group C (bug_report vdgtxa2n): moving a champion between tiers, by
 * keyboard or by drag, never reloads the list or resets the scroll; the
 * header says "Saving..." then "Saved", and a failed save says "Could not
 * save" with Retry and puts the champion back.
 * Requires a running dev server (CI runs it).
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

const TOP = "#pool-roles-panel-Top";

async function addTop(page: Page, name: string) {
  const field = page.locator("#pool-add-top");
  await field.fill(name);
  await expect(page.locator(`#pool-add-top-list [role="option"]`).first()).toBeVisible();
  await field.press("Enter");
  await expect(page.locator(`${TOP} [data-tier="comfort"] li[data-champion="${name}"]`)).toBeVisible({
    timeout: 5000,
  });
}

/** A short viewport so the ledger is below the fold, then scroll to it. */
async function scrollToLedger(page: Page) {
  await page.setViewportSize({ width: 1280, height: 420 });
  await page.locator(`${TOP} [data-tier="practicing"]`).scrollIntoViewIfNeeded();
  await page.evaluate(() => window.scrollBy(0, 40));
  return page.evaluate(() => window.scrollY);
}

test("champion-pool moves: keyboard move keeps the scroll and shows the save state", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await navigateTo(page, "/champion-pool");
  for (const name of ["Ahri", "Garen", "Lux"]) await addTop(page, name);

  const before = await scrollToLedger(page);
  expect(before).toBeGreaterThan(0);
  // The skeleton must never replace the tiers during a move.
  await page.evaluate(() => {
    (window as unknown as { __skeleton: boolean }).__skeleton = false;
    new MutationObserver(() => {
      if (document.querySelector('#pool-roles-panel-Top .animate-pulse')) {
        (window as unknown as { __skeleton: boolean }).__skeleton = true;
      }
    }).observe(document.body, { subtree: true, childList: true });
  });

  await page.evaluate(() => document.getElementById("pool-top-Ahri")?.focus({ preventScroll: true }));
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Space");

  await expect(page.locator(`${TOP} [data-tier="match_ready"] li[data-champion="Ahri"]`)).toBeVisible();
  await expect(page.locator('[data-save-state="saved"]')).toHaveText(/Saved/, { timeout: 5000 });
  const after = await page.evaluate(() => window.scrollY);
  expect(Math.abs(after - before)).toBeLessThanOrEqual(2);
  expect(await page.evaluate(() => (window as unknown as { __skeleton: boolean }).__skeleton)).toBe(false);
  await expect(page.locator("#pool-top-Ahri")).toBeFocused();

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool moves: drag and drop keeps the scroll", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await navigateTo(page, "/champion-pool");
  for (const name of ["Ahri", "Garen", "Lux"]) await addTop(page, name);

  await scrollToLedger(page);
  // The drag itself scrolls the source and then the target into view (what
  // `dragTo` does too); the page must keep the scroll from the drop on.
  await page.locator(`${TOP} li[data-champion="Garen"]`).hover();
  await page.mouse.down();
  await page.locator(`${TOP} [data-tier="scrim_ready"]`).hover();
  const before = await page.evaluate(() => window.scrollY);
  await page.mouse.up();

  await expect(page.locator(`${TOP} [data-tier="scrim_ready"] li[data-champion="Garen"]`)).toBeVisible();
  await expect(page.locator('[data-save-state="saved"]')).toBeVisible({ timeout: 5000 });
  const after = await page.evaluate(() => window.scrollY);
  expect(Math.abs(after - before)).toBeLessThanOrEqual(2);

  // Stored: a reload shows it in the new tier.
  await navigateTo(page, "/champion-pool");
  await expect(page.locator(`${TOP} [data-tier="scrim_ready"] li[data-champion="Garen"]`)).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool moves: a failed save reverts and offers Retry", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/champion-pool");
  await addTop(page, "Ahri");

  await page.route("**/api/set_champion_tier*", (route) => route.abort());
  await page.locator("#pool-top-Ahri").focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Space");

  const status = page.locator('[data-save-state="failed"]');
  await expect(status).toContainText("Could not save", { timeout: 5000 });
  await expect(page.locator(`${TOP} [data-tier="comfort"] li[data-champion="Ahri"]`)).toBeVisible();

  await page.unroute("**/api/set_champion_tier*");
  await status.getByRole("button", { name: "Retry" }).click();
  await expect(page.locator(`${TOP} [data-tier="match_ready"] li[data-champion="Ahri"]`)).toBeVisible();
  await expect(page.locator('[data-save-state="saved"]')).toBeVisible({ timeout: 5000 });
});
