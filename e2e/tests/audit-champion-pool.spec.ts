/**
 * Champion pool audit tests.
 *
 * Documents any bugs and verifies working flows for the /champion-pool route.
 * Requires a running dev server: cargo leptos watch
 *
 * Champion pool is per-user (not per-team), but we use teamPage to ensure a
 * consistent test environment with a known user.
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("champion-pool: page loads with empty state", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await page.waitForTimeout(500);

  // Page should render — h1 or main content visible
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // Should NOT show a full-page error banner
  const errorBanner = page.locator('[class*="error"], [class*="Error"]').filter({ hasText: /error|failed/i });
  const errorVisible = await errorBanner.isVisible({ timeout: 1000 }).catch(() => false);
  if (errorVisible) {
    console.log("AUDIT-FINDING: Error banner visible on /champion-pool page load");
  }

  // Look for champion autocomplete or grid
  const autocompleteInput = page.locator("input").first();
  const hasInput = await autocompleteInput.isVisible({ timeout: 2000 }).catch(() => false);
  if (!hasInput) {
    console.log("AUDIT-FINDING: No input/autocomplete visible on /champion-pool — may need Data Dragon data");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool: add a champion to pool", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await page.waitForTimeout(1000);

  // Look for an autocomplete input to search for a champion
  const autocompleteInput = page.locator("input").first();
  const hasInput = await autocompleteInput.isVisible({ timeout: 3000 }).catch(() => false);

  if (!hasInput) {
    console.log("AUDIT-FINDING: No autocomplete input visible on /champion-pool — cannot test add flow");
    expect(filterRealErrors(errors)).toHaveLength(0);
    return;
  }

  // Type a champion name
  await autocompleteInput.fill("Ahri");
  await page.waitForTimeout(500);

  // Look for dropdown suggestions
  const dropdown = page.locator('[class*="dropdown"], [class*="autocomplete"], [role="listbox"], [class*="suggestion"]').first();
  const hasDropdown = await dropdown.isVisible({ timeout: 2000 }).catch(() => false);

  if (hasDropdown) {
    // Click the first suggestion
    const firstSuggestion = dropdown.locator("button, li, [role='option']").first();
    const hasSuggestion = await firstSuggestion.isVisible({ timeout: 1000 }).catch(() => false);
    if (hasSuggestion) {
      await firstSuggestion.click();
      await page.waitForTimeout(1000);
    }
  } else {
    // Try pressing Enter to select
    await autocompleteInput.press("Enter");
    await page.waitForTimeout(1000);
  }

  // Look for "Add to Pool" or similar button
  const addBtn = page.locator('button:has-text("Add"), button:has-text("Add to Pool")').first();
  const hasAddBtn = await addBtn.isVisible({ timeout: 2000 }).catch(() => false);
  if (hasAddBtn) {
    await addBtn.click();
    await page.waitForTimeout(1500);
    // Verify champion appears in pool (look for Ahri text)
    const ahriInPool = page.locator("text=Ahri").first();
    const isInPool = await ahriInPool.isVisible({ timeout: 3000 }).catch(() => false);
    if (!isInPool) {
      console.log("AUDIT-FINDING: Ahri not visible in champion pool after add — add may have failed or UI not updated");
    }
  } else {
    console.log("AUDIT-FINDING: No explicit Add button found on /champion-pool after champion selection");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("champion-pool: remove a champion from pool", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/champion-pool");
  await page.waitForTimeout(1000);

  // First check if there are any existing pool entries with remove buttons
  const removeBtn = page.locator('button:has-text("Remove"), button[aria-label*="remove"], button[title*="remove"]').first();
  const hasRemoveBtn = await removeBtn.isVisible({ timeout: 2000 }).catch(() => false);

  if (!hasRemoveBtn) {
    // Try adding a champion first (using the same pattern as test 2)
    const autocompleteInput = page.locator("input").first();
    const hasInput = await autocompleteInput.isVisible({ timeout: 2000 }).catch(() => false);
    if (!hasInput) {
      console.log("AUDIT-FINDING: Cannot test remove — no input and no existing pool entries found");
      expect(filterRealErrors(errors)).toHaveLength(0);
      return;
    }
    await autocompleteInput.fill("Garen");
    await page.waitForTimeout(500);
    await autocompleteInput.press("Enter");
    await page.waitForTimeout(500);
    const addBtn2 = page.locator('button:has-text("Add")').first();
    if (await addBtn2.isVisible({ timeout: 1500 }).catch(() => false)) {
      await addBtn2.click();
      await page.waitForTimeout(1500);
    }
  }

  // Now look for a remove button
  const removeBtnAfterAdd = page.locator('button:has-text("Remove"), button[aria-label*="emove"]').first();
  const hasRemove = await removeBtnAfterAdd.isVisible({ timeout: 3000 }).catch(() => false);

  if (hasRemove) {
    await removeBtnAfterAdd.click();
    await page.waitForTimeout(1000);
    // Verify it's gone (soft check)
    const stillVisible = await removeBtnAfterAdd.isVisible({ timeout: 1000 }).catch(() => false);
    if (stillVisible) {
      console.log("AUDIT-FINDING: Remove button still visible after click — champion may not have been removed");
    }
  } else {
    console.log("AUDIT-FINDING: No remove button found on /champion-pool — remove flow untestable");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});
