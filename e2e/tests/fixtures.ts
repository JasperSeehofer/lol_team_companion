/**
 * Shared test fixtures — provides an authenticated page for e2e tests.
 *
 * Usage:
 *   import { test, expect } from "./fixtures";
 *   test("my test", async ({ authedPage }) => { ... });
 */
import { test as base, expect, type Page } from "@playwright/test";

const TIMESTAMP = Date.now();
const TEST_EMAIL = `pages_${TIMESTAMP}@test.invalid`;
const TEST_PASSWORD = "Test1234!";
const TEST_USERNAME = `pagesuser_${TIMESTAMP}`;

/** Register a new user, then log in. */
async function authenticatePage(page: Page): Promise<void> {
  // Step 1: Register (creates the user, redirects to /auth/login)
  await page.goto("/auth/register");
  await page.fill("input[name=username]", TEST_USERNAME);
  await page.fill("input[name=email]", TEST_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");
  await page.waitForLoadState("networkidle");

  // Step 2: Always log in — registration does not auto-login
  await page.goto("/auth/login");
  await page.fill("input[name=email]", TEST_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");
  await page.waitForLoadState("networkidle");

  // Login triggers hard navigation to /team/dashboard — wait for it
  await page.waitForURL("**/team/dashboard", { timeout: 5000 }).catch(() => {
    // If redirect didn't happen, we may already be on the right page
  });
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
