/**
 * Miscellaneous pages audit tests.
 *
 * Documents any bugs and verifies basic load + empty state for:
 * stats, profile, opponents, action-items, team-builder pages.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("stats: page loads with empty state", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  // /stats now redirects → /library?tab=stats (D-11, Plan 28-06).
  await navigateTo(page, "/stats");
  await page.waitForTimeout(1000);

  // The destination (/library?tab=stats) is a room surface — no standalone h1.
  // Assert: tab nav is present (RoomSurface renders role="tablist") and
  // no JS errors (the redirect itself is error-free).
  await expect(page.locator('nav[role="tablist"]').first()).toBeVisible({ timeout: 5000 });

  // Look for empty state message or stats content
  const emptyMsg = page.locator('text=/no matches|no stats|no data|get started/i').first();
  const hasEmptyMsg = await emptyMsg.isVisible({ timeout: 2000 }).catch(() => false);
  if (hasEmptyMsg) {
    console.log("AUDIT-INFO: /library?tab=stats shows empty state message (expected for new user)");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("profile: page loads and shows identity content", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);

  // /profile now redirects → /throne?tab=identity (D-11, Plan 29-05).
  await navigateTo(page, "/profile");
  await page.waitForTimeout(1000);

  // The destination (/throne?tab=identity) is a room surface — no standalone h1.
  // Assert: tab nav is present (RoomSurface) and no JS errors.
  await expect(page.locator('nav[role="tablist"]').first()).toBeVisible({ timeout: 5000 });

  // Check for the Identity tab button or some identity-related content
  const identityTab = page.locator('button:has-text("Identity")').first();
  const hasIdentityTab = await identityTab.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasIdentityTab) {
    console.log("AUDIT-FINDING: Identity tab not visible on /throne");
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

  // /action-items now redirects → /throne?tab=development (D-11, Plan 29-05).
  await navigateTo(page, "/action-items");
  await page.waitForTimeout(1000);

  // The destination (/throne?tab=development) is a room surface — no standalone h1.
  // Assert: tab nav is present (RoomSurface) and redirect arrived at /throne.
  await expect(page.locator('nav[role="tablist"]').first()).toBeVisible({ timeout: 5000 });

  // Phase 29.1 FIXED the tachys hydration panic that previously affected team
  // users here (ActionItemsPanel nested in keep-mounted DevelopmentSurface).
  // The fix converted ActionItemsPanel/Page Suspense reads to Suspend::new.
  // This assertion now actively gates the fix: no panic/unreachable console error.
  // See: .planning/phases/29.1-actionitemspanel-hydration-reconciliation-gap-closure-for-ph/
  const panics = filterRealErrors(errors).filter(
    (e) =>
      e.toLowerCase().includes("panic") ||
      e.toLowerCase().includes("unreachable")
  );
  expect(panics).toHaveLength(0);
});

test("team-builder: page loads and basic interaction", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/team-builder");
  await page.waitForTimeout(500);

  // Page should render
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // Look for any interactive elements — dropdowns, champion selectors, role buttons
  const selects = page.locator("select");
  const selectCount = await selects.count();
  if (selectCount > 0) {
    // Interact with the first select (likely role or champion selector)
    const opts = await selects.first().locator("option").count();
    if (opts > 1) {
      const secondOpt = await selects.first().locator("option").nth(1).getAttribute("value");
      if (secondOpt) {
        await selects.first().selectOption(secondOpt);
        await page.waitForTimeout(300);
      }
    }
  }

  // Look for buttons (role slots, champion selectors)
  const buttons = page.locator("button:not([disabled])");
  const btnCount = await buttons.count();
  if (btnCount > 0) {
    // Click a visible button that's not a nav link
    const firstNonNavBtn = buttons.filter({ hasNot: page.locator('a') }).first();
    const isVisible = await firstNonNavBtn.isVisible({ timeout: 1000 }).catch(() => false);
    if (isVisible) {
      await firstNonNavBtn.click();
      await page.waitForTimeout(300);
    }
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});
