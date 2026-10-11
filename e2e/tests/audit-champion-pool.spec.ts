/**
 * Champion pool audit tests: empty state, add, remove (with Undo) and the
 * keyboard tier move. The pool is per user; teamPage gives a fresh user.
 * Requires a running dev server: cargo leptos watch
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

const TOP = "#pool-roles-panel-Top";

/** Add a champion to the Top ledger through the search field. */
async function addTop(page: Page, name: string) {
  const field = page.locator("#pool-add-top");
  await field.fill(name);
  await expect(page.locator(`#pool-add-top-list [role="option"]`).first()).toBeVisible();
  await field.press("Enter");
  await expect(page.locator(`${TOP} [data-tier="comfort"] li[data-champion="${name}"]`)).toBeVisible({
    timeout: 5000,
  });
}

test("champion-pool: page loads with empty state", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await expect(page.getByRole("heading", { level: 1, name: "Champion pool" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Top" })).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(TOP).getByText("No top champions yet.")).toBeVisible();
  await expect(page.locator("#pool-add-top")).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool: add a champion to pool", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await addTop(page, "Ahri");

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool: remove a champion, then undo", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await addTop(page, "Garen");

  // Open the champion's detail and remove it.
  await page.locator("#pool-top-Garen").click();
  await page.getByRole("button", { name: "Remove from pool" }).click();
  await expect(page.locator(`${TOP} li[data-champion="Garen"]`)).toHaveCount(0, { timeout: 5000 });

  // Undo puts it back.
  await page.getByRole("button", { name: "Undo" }).click();
  await expect(page.locator(`${TOP} li[data-champion="Garen"]`)).toBeVisible({ timeout: 5000 });

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool: move a champion between tiers with the keyboard", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await addTop(page, "Ahri");

  // Space picks it up, Down chooses the next tier, Space drops it there.
  await page.locator("#pool-top-Ahri").focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Space");

  await expect(page.locator(`${TOP} [data-tier="match_ready"] li[data-champion="Ahri"]`)).toBeVisible({
    timeout: 5000,
  });
  await expect(page.locator(`${TOP} [data-tier="comfort"] li[data-champion="Ahri"]`)).toHaveCount(0);
  await expect(page.getByTestId("pool-announce")).toHaveText("Ahri moved to Match ready.");

  // The move is stored: it survives a reload.
  await navigateTo(page, "/champion-pool");
  await expect(page.locator(`${TOP} [data-tier="match_ready"] li[data-champion="Ahri"]`)).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});
