/**
 * Shared test fixtures — provides an authenticated page for e2e tests.
 *
 * Usage:
 *   import { test, expect } from "./fixtures";
 *   test("my test", async ({ authedPage }) => { ... });
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

type Fixtures = {
  authedPage: Page;
};

export const test = base.extend<Fixtures>({
  authedPage: async ({ page }, use) => {
    await authenticatePage(page);
    await use(page);
  },
});

export { expect };
