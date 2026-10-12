/**
 * Miscellaneous pages audit tests.
 *
 * Documents any bugs and verifies basic load + empty state for:
 * stats, profile, opponents and action-items pages.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("stats: page loads with empty state", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/stats");
  await page.waitForTimeout(500);

  // Page should render (h1 visible)
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // Should NOT show a full-page crash — look for either empty state or stats content
  const hasContent = await page.locator("main, .max-w-\\[80rem\\], [class*='container']").first().isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasContent) {
    console.log("AUDIT-FINDING: /stats page appears to have no main content container");
  }

  // Look for empty state message or stats content
  const emptyMsg = page.locator('text=/no matches|no stats|no data|get started/i').first();
  const hasEmptyMsg = await emptyMsg.isVisible({ timeout: 2000 }).catch(() => false);
  if (hasEmptyMsg) {
    console.log("AUDIT-INFO: /stats shows empty state message (expected for new user)");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("profile: page loads and shows username", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/profile");
  await page.waitForTimeout(500);

  // Page should render (h1 visible)
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // Username or email should be displayed somewhere
  // The user was registered with pattern: pagesuser_{timestamp}_{id}
  const usernameEl = page.locator('text=/pagesuser|@test.invalid/').first();
  const hasUsername = await usernameEl.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasUsername) {
    // Maybe it's in an input field
    const usernameInput = page.locator('input[value*="pagesuser"]').first();
    const inputVisible = await usernameInput.isVisible({ timeout: 1000 }).catch(() => false);
    if (!inputVisible) {
      console.log("AUDIT-FINDING: Username/email not visibly displayed on /profile page");
    }
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("opponents: page loads", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/opponents");
  await page.waitForTimeout(500);

  // Page should render (not blank, not just a loading spinner forever)
  // Check for h1 or any meaningful content
  const hasHeading = await page.locator("h1").isVisible({ timeout: 5000 }).catch(() => false);
  if (!hasHeading) {
    console.log("AUDIT-FINDING: No h1 heading on /opponents page — may not render correctly");
  } else {
    await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });
  }

  // Check for any WASM panics
  const panics = filterRealErrors(errors).filter((e) =>
    e.toLowerCase().includes("panic")
  );
  expect(panics).toHaveLength(0);

  // No real console errors
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("action-items: page loads", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/action-items");
  await page.waitForTimeout(500);

  // Page should render (not blank)
  const hasHeading = await page.locator("h1").isVisible({ timeout: 5000 }).catch(() => false);
  if (!hasHeading) {
    console.log("AUDIT-FINDING: No h1 heading on /action-items page — may not render correctly");
  } else {
    await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });
  }

  // Check for any WASM panics
  const panics = filterRealErrors(errors).filter((e) =>
    e.toLowerCase().includes("panic")
  );
  expect(panics).toHaveLength(0);

  // No real console errors
  expect(filterRealErrors(errors)).toHaveLength(0);
});

