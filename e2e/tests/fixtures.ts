/**
 * Shared test fixtures — provides an authenticated page for e2e tests.
 *
 * Usage:
 *   import { test, expect } from "./fixtures";
 *   test("my test", async ({ authedPage }) => { ... });
 *   test("team test", async ({ teamPage }) => { ... });
 */
import { test as base, expect, type Page } from "@playwright/test";

const TIMESTAMP = Date.now();
const TEST_PASSWORD = "Test1234!";
let userCounter = 0;

/** Register a new user — registration auto-logs in and redirects to /team/dashboard. */
async function authenticatePage(page: Page): Promise<void> {
  const id = ++userCounter;
  const email = `pages_${TIMESTAMP}_${id}@test.invalid`;
  const username = `pagesuser_${TIMESTAMP}_${id}`;

  await page.goto("/auth/register");
  await page.fill("input[name=username]", username);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");

  // Registration auto-logs in and fires window.location.set_href("/team/dashboard")
  // via a WASM Effect. Wait for the URL to reach /team/dashboard.
  await page.waitForURL("**/team/dashboard", { timeout: 20000 });

  // Critical: The WASM hydration on /team/dashboard may fire ANOTHER redirect
  // (the register page's Effect can queue a redirect that fires after we land).
  // Wait for the page to fully stabilize — no more navigations.
  await page.waitForLoadState("load");
  // Give WASM a moment to hydrate and fire any remaining Effects
  await page.waitForTimeout(500);

  // Now verify we're properly authenticated
  if (page.url().includes("/auth/")) {
    throw new Error(`Authentication failed — ended at ${page.url()}`);
  }
}

/**
 * Create a team for the current authenticated user.
 * Navigates to /team/roster, fills the form if visible, and waits for redirect.
 * Safe to call even if the user already has a team — the form won't be visible.
 *
 * Source: derived from existing pipeline.spec.ts pattern (lines 25-40).
 */
export async function createTeam(page: Page, name: string): Promise<void> {
  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  const teamNameInput = page.locator('input[name="name"]');
  if (await teamNameInput.isVisible({ timeout: 3000 }).catch(() => false)) {
    await teamNameInput.fill(name);
    const regionSelect = page.locator('select[name="region"]');
    if (await regionSelect.isVisible({ timeout: 1000 }).catch(() => false)) {
      await regionSelect.selectOption("NA");
    }
    await page.click('button:has-text("Create Team")');
    await page.waitForURL("**/team/dashboard", { timeout: 15000 });
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }
}

type Fixtures = {
  authedPage: Page;
  teamPage: Page;
};

export const test = base.extend<Fixtures>({
  authedPage: async ({ page }, use) => {
    await authenticatePage(page);
    await use(page);
  },

  teamPage: async ({ page }, use) => {
    await authenticatePage(page);
    await createTeam(page, `AuditTeam_${Date.now()}`);
    await use(page);
  },
});

export { expect };
