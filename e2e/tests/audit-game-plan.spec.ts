/**
 * Game plan audit tests.
 *
 * Documents any bugs and verifies working flows for the /game-plan route.
 * Requires a running dev server: cargo leptos watch
 *
 * Uses teamPage fixture since game plans are scoped to a team.
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo, saveGamePlan } from "./helpers";

test("game-plan: create a new game plan", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const planName = `AuditPlan_${Date.now()}`;

  await navigateTo(page, "/game-plan");
  // Saving shows "Saved" and puts the plan id in the URL.
  await saveGamePlan(page, planName);
  await expect(page).toHaveURL(/plan_id=/);

  // Assert no real console errors
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("game-plan: plan appears in saved plans list", async ({ teamPage }) => {
  const page = teamPage;
  const planName = `AuditPlanList_${Date.now()}`;

  await navigateTo(page, "/game-plan");
  await saveGamePlan(page, planName);

  // The plan appears in the plan list beside the editor.
  await expect(page.getByTestId("plan-list").getByText(planName)).toBeVisible({
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
  await navigateTo(page, "/game-plan");
  await expect(page.locator("h1")).toBeVisible({ timeout: 5000 });
});

test("game-plan: checklist add, tick and progress", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/game-plan");
  await saveGamePlan(page, `AuditChecklist_${Date.now()}`);

  const progress = page.getByTestId("checklist-progress");
  for (const item of ["Check summoner spells", "Agree the level 1"]) {
    await page.locator("#checklist-new").fill(item);
    await page.locator("#checklist-new").press("Enter");
  }
  await expect(progress).toHaveText("0 of 2 done");

  // Space on a checkbox ticks it (a native checkbox inside its label).
  const first = page.getByRole("checkbox", { name: "Check summoner spells" });
  await first.focus();
  await page.keyboard.press("Space");
  await expect(first).toBeChecked();
  await expect(progress).toHaveText("1 of 2 done");

  expect(filterRealErrors(errors)).toHaveLength(0);
});
