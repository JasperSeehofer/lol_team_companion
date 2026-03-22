/**
 * Pipeline e2e test: draft → game plan → post-game linking.
 *
 * Verification for GSD Phase 1 (Pipeline CTAs):
 * 1. Create draft with champion picks → save
 * 2. Click "Prep for This Draft" → game plan prefills champions
 * 3. Save game plan → click "Source Draft" → draft loads correctly
 * 4. Game plan + post-game pages handle draft_id query param gracefully
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

test("full pipeline: draft with picks → game plan prefill → save → source draft roundtrip", async ({
  authedPage,
}) => {
  const page = authedPage;
  const timestamp = Date.now();
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  // --- 1. Create team ---
  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  const teamNameInput = page.locator('input[name="name"]');
  if (await teamNameInput.isVisible({ timeout: 3000 }).catch(() => false)) {
    await teamNameInput.fill(`PipeTeam_${timestamp}`);
    const regionSelect = page.locator('select[name="region"]');
    if (await regionSelect.isVisible({ timeout: 1000 }).catch(() => false)) {
      await regionSelect.selectOption("NA");
    }
    await page.click('button:has-text("Create Team")');
    await page.waitForURL("**/team/dashboard", { timeout: 15000 });
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(2000);
  }

  // --- 2. Create draft with champion picks ---
  await page.goto("/draft");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1500);

  // Fill draft name
  await page.getByRole("textbox").first().fill(`PipeDraft_${timestamp}`);

  // Pick 10 champions (6 bans + 4 picks) from the champion grid
  // Champion picker uses buttons inside a grid with display:grid style
  const champButtons = page.locator(
    'div[style*="display:grid"] button:not([disabled])'
  );
  let picked = 0;
  const champCount = await champButtons.count();
  for (let i = 0; i < champCount && picked < 10; i++) {
    const btn = champButtons.nth(i);
    if (await btn.isDisabled().catch(() => true)) continue;
    await btn.click();
    await page.waitForTimeout(100);
    picked++;
  }

  // Save the draft
  await page.locator('button:has-text("Save Draft")').click();
  await page.waitForTimeout(2000);

  // Verify draft saved — "Update Draft" button appears when loaded
  await expect(
    page.locator('button:has-text("Update Draft")')
  ).toBeVisible({ timeout: 3000 });

  // --- 3. Click "Prep for This Draft" → game plan page ---
  const prepBtn = page.locator('button:has-text("Prep for This Draft")');
  await expect(prepBtn.first()).toBeVisible({ timeout: 5000 });
  await prepBtn.first().click();

  // Wait for hard navigation to game plan page
  await page.waitForTimeout(3000);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(2000);

  // Verify game plan page loaded with draft_id
  expect(page.url()).toContain("/game-plan");
  expect(page.url()).toContain("draft_id=");

  // Verify "Source Draft" badge and "Champions pre-filled" indicator
  await expect(
    page.locator('text=Source Draft')
  ).toBeVisible({ timeout: 3000 });
  await expect(
    page.locator('text=Champions pre-filled from draft')
  ).toBeVisible({ timeout: 3000 });

  // --- 4. Save the game plan ---
  const planNameInput = page.locator('input[placeholder*="Comp"]');
  if (await planNameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
    await planNameInput.fill(`PipePlan_${timestamp}`);
  }

  await page.locator('button:has-text("Save Plan")').click();
  await page.waitForTimeout(2000);

  // Verify plan saved
  const bodyText = await page.textContent("body") || "";
  expect(bodyText).toMatch(/Plan created|saved|Updated/i);

  // --- 5. Click Source Draft link → verify draft loads ---
  const sourceDraftLink = page.locator('a[href*="/draft?draft_id="]');
  await expect(sourceDraftLink.first()).toBeVisible({ timeout: 3000 });
  await sourceDraftLink.first().click();
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(2000);

  // Verify draft page loaded with correct draft
  expect(page.url()).toContain("/draft?draft_id=");

  // Draft name should be loaded
  const draftNameVal = await page.getByRole("textbox").first().inputValue();
  expect(draftNameVal).toContain(`PipeDraft_${timestamp}`);

  // "Update Draft" button means draft is loaded from DB
  await expect(
    page.locator('button:has-text("Update Draft")')
  ).toBeVisible({ timeout: 3000 });

  // "Review This Game" button should be visible (draft has a linked game plan)
  await expect(
    page.locator('button:has-text("Review This Game")')
  ).toBeVisible({ timeout: 3000 });

  // Nav should be present
  await expect(page.locator("nav")).toBeVisible();

  // No real JS errors
  const realErrors = errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
  expect(realErrors).toHaveLength(0);
});

test("game plan page loads cleanly with draft_id query param", async ({
  authedPage,
}) => {
  const page = authedPage;
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  // Navigate to game plan with a non-existent draft_id — should not crash
  await page.goto("/game-plan?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  expect(page.url()).toContain("/game-plan");
  const bodyText = await page.textContent("body");
  expect(bodyText).toMatch(/game|plan|strategy/i);
  await expect(page.locator("nav")).toBeVisible();

  const realErrors = errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
  expect(realErrors).toHaveLength(0);
});

test("post-game page loads cleanly with draft_id query param", async ({
  authedPage,
}) => {
  const page = authedPage;
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  await page.goto("/post-game?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  expect(page.url()).toContain("/post-game");
  const bodyText = await page.textContent("body");
  expect(bodyText).toMatch(/post|game|review/i);
  await expect(page.locator("nav")).toBeVisible();

  const realErrors = errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
  expect(realErrors).toHaveLength(0);
});
