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

/** Register + login, return the authenticated page. */
async function authenticatePage(page: Page): Promise<void> {
  await page.goto("/auth/register");
  await page.fill("input[name=username]", TEST_USERNAME);
  await page.fill("input[name=email]", TEST_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");
  await page.waitForLoadState("networkidle");

  // If registration redirected us away from register, we're logged in.
  // Otherwise, the user may already exist — try logging in.
  if (page.url().includes("/auth/register")) {
    await page.goto("/auth/login");
    await page.fill("input[name=email]", TEST_EMAIL);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await page.waitForLoadState("networkidle");
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
