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

/** Register a new user — registration auto-logs in and redirects to /solo.
 *
 * Phase 17 plan 17-06 introduced the closed-beta gate (D-16): `/auth/register`
 * without `?invite=...` redirects to `/closed-beta` via a WASM Effect. The
 * fixture must therefore always pass an invite query param. Phase 19.1 will
 * wire real server-side validation against the `invite_code` table; for now
 * any non-empty string is echoed into the hidden input and the registration
 * server fn accepts it.
 *
 * Phase 17 plan 17-06 also introduced D-03: new users default to solo mode and
 * `register_action()` now returns `/solo` (not `/team/dashboard`). The fixture
 * waits for `/solo`; tests that need a team explicitly use the `teamPage`
 * fixture which navigates from `/solo` → `/team/roster` → creates a team.
 */
async function authenticatePage(page: Page): Promise<void> {
  const id = ++userCounter;
  const email = `pages_${TIMESTAMP}_${id}@test.invalid`;
  const username = `pagesuser_${TIMESTAMP}_${id}`;

  await page.goto("/auth/register?invite=E2E-TEST");
  await page.fill("input[name=username]", username);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");

  // Registration auto-logs in and fires window.location.set_href("/solo")
  // via a WASM Effect (D-03). Wait for the URL to reach /solo.
  await page.waitForURL("**/solo", { timeout: 20000 });

  // Critical: The WASM hydration on /solo may fire ANOTHER redirect
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
