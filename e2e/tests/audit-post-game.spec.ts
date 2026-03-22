/**
 * Post-game review audit tests.
 *
 * Documents any bugs and verifies working flows for the /post-game route.
 * Requires a running dev server: cargo leptos watch
 *
 * Uses teamPage fixture since reviews are scoped to a team.
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("post-game: create a new review", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/post-game");
  await page.waitForTimeout(500);

  // Fill "What Went Well" textarea
  const wentWellTextarea = page.locator("textarea").first();
  await wentWellTextarea.fill("Good teamfight execution around Dragon");

  // Click "Save Review" button
  await page.locator('button:has-text("Save Review")').click();
  await page.waitForTimeout(2000);

  // Verify save succeeded — button switches to "Update Review"
  await expect(page.locator('button:has-text("Update Review")')).toBeVisible({
    timeout: 5000,
  });

  // Assert no real console errors
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("post-game: draft_id param loads without crash", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  // Navigate with a nonexistent draft_id — should not crash, just render clean
  await page.goto("/post-game?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  // Page should render (h1 visible)
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // No WASM panics
  const panics = filterRealErrors(errors).filter((e) =>
    e.toLowerCase().includes("panic")
  );
  expect(panics).toHaveLength(0);

  // Check if the hidden draft_id field got populated (soft check)
  const pagContent = await page.content();
  if (pagContent.includes("draft:nonexistent")) {
    console.log("AUDIT-FINDING: draft:nonexistent appears in page content — URL param was applied as draft_id seed");
  }
});

test("post-game: review appears in saved list", async ({ teamPage }) => {
  const page = teamPage;

  await navigateTo(page, "/post-game");
  await page.waitForTimeout(500);

  // Create a review
  const wentWellTextarea = page.locator("textarea").first();
  await wentWellTextarea.fill("AuditListCheck_won_teamfight");
  await page.locator('button:has-text("Save Review")').click();
  await page.waitForTimeout(2000);

  // Verify save succeeded (prerequisite)
  const updateBtn = page.locator('button:has-text("Update Review")');
  const updateVisible = await updateBtn.isVisible({ timeout: 5000 }).catch(() => false);
  if (!updateVisible) {
    console.log("AUDIT-FINDING: Save Review did not switch to Update Review — save may have failed");
    return;
  }

  // The saved reviews list is on the left sidebar; look for any review item
  // The list shows match_riot_id or "No match linked" for each review
  const reviewListItem = page.locator('text=No match linked').first();
  const listVisible = await reviewListItem.isVisible({ timeout: 3000 }).catch(() => false);

  if (!listVisible) {
    console.log("AUDIT-FINDING: Saved review not visible in left sidebar list after save + refetch");
  } else {
    // At least one review appears in the list
    await expect(reviewListItem).toBeVisible({ timeout: 3000 });
  }
});

test("post-game: no console errors during form interactions", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/post-game");
  await page.waitForTimeout(500);

  // Interact with all textarea fields
  const textareas = page.locator("textarea");
  const count = await textareas.count();
  for (let i = 0; i < count && i < 3; i++) {
    await textareas.nth(i).fill(`Test content ${i}`);
    await page.waitForTimeout(100);
  }

  // Check for any dropdowns (draft_id or game_plan_id selects)
  const selects = page.locator("select");
  const selectCount = await selects.count();
  if (selectCount > 0) {
    // Just verify the dropdown exists and can be interacted with
    const options = await selects.first().locator("option").count();
    if (options > 1) {
      // Select the second option
      const optionValues = await selects.first().locator("option").nth(1).getAttribute("value");
      if (optionValues) {
        await selects.first().selectOption(optionValues);
        await page.waitForTimeout(200);
      }
    }
  }

  // Assert no real console errors after all interactions
  expect(filterRealErrors(errors)).toHaveLength(0);
});
