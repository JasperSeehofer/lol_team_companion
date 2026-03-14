/**
 * Game plan audit tests.
 *
 * Documents any bugs and verifies working flows for the /game-plan route.
 * Requires a running dev server: cargo leptos watch
 *
 * Uses teamPage fixture since game plans are scoped to a team.
 */
import { test, expect } from "./fixtures";

/**
 * Capture console errors for a page, filtering known-harmless noise.
 */
function captureErrors(page: import("@playwright/test").Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`[pageerror] ${e.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`[console.error] ${msg.text()}`);
  });
  return errors;
}

function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
}

test("game-plan: create a new game plan", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const planName = `AuditPlan_${Date.now()}`;

  await page.goto("/game-plan");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  // Fill the plan name input (prop:value-controlled input near label)
  const planNameInput = page.locator("input").first();
  await planNameInput.fill(planName);

  // Click "Save Plan" button
  await page.locator('button:has-text("Save Plan")').click();
  await page.waitForTimeout(2000);

  // Verify save succeeded — button switches to "Update Plan"
  await expect(page.locator('button:has-text("Update Plan")')).toBeVisible({
    timeout: 5000,
  });

  // Assert no real console errors
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("game-plan: plan appears in saved plans list", async ({ teamPage }) => {
  const page = teamPage;
  const planName = `AuditPlanList_${Date.now()}`;

  await page.goto("/game-plan");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  // Create and save a plan
  const planNameInput = page.locator("input").first();
  await planNameInput.fill(planName);
  await page.locator('button:has-text("Save Plan")').click();
  await page.waitForTimeout(2000);

  // Verify save succeeded (prerequisite)
  await expect(page.locator('button:has-text("Update Plan")')).toBeVisible({
    timeout: 5000,
  });

  // The plan should appear in the left sidebar plan list
  await expect(page.locator(`text=${planName}`)).toBeVisible({
    timeout: 5000,
  });
});

test("game-plan: draft_id prefill works without crash", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  // Navigate with a nonexistent draft_id — should not crash
  await page.goto("/game-plan?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  // Page should render (h1 visible)
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });

  // No WASM panics
  const panics = filterRealErrors(errors).filter((e) =>
    e.toLowerCase().includes("panic")
  );
  expect(panics).toHaveLength(0);

  // Navigate to clean game-plan — also should not crash
  await page.goto("/game-plan");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });
});

test("game-plan: checklist interaction (if available)", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const planName = `AuditChecklist_${Date.now()}`;

  await page.goto("/game-plan");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  // Create and save a plan first
  const planNameInput = page.locator("input").first();
  await planNameInput.fill(planName);
  await page.locator('button:has-text("Save Plan")').click();
  await page.waitForTimeout(2000);

  // Verify plan saved
  await expect(page.locator('button:has-text("Update Plan")')).toBeVisible({
    timeout: 5000,
  });

  // Look for checklist section or "Add" button for objectives
  const checklistSection = page.locator('[class*="checklist"], button:has-text("Add Checklist"), button:has-text("Add Item")').first();
  const hasChecklist = await checklistSection.isVisible({ timeout: 2000 }).catch(() => false);

  if (hasChecklist) {
    await checklistSection.click();
    await page.waitForTimeout(500);
    // Look for checkbox items
    const checkboxes = page.locator('input[type="checkbox"]');
    const checkboxCount = await checkboxes.count();
    if (checkboxCount > 0) {
      await checkboxes.first().click();
      await page.waitForTimeout(300);
    }
  } else {
    // No checklist UI visible — this is an audit finding, not necessarily a bug
    console.log("AUDIT-FINDING: No checklist interaction UI visible on /game-plan after plan creation");
  }

  // No real errors in any case
  expect(filterRealErrors(errors)).toHaveLength(0);
});
