/**
 * Pipeline e2e test, the core loop on the rebuilt strategy pages:
 * 1. Draft through the board with the keyboard, save, reopen.
 * 2. "Prep a game plan" opens a plan linked to the draft; save it.
 * 3. "Start the review" opens a review linked to the plan; its action
 *    item lines become action items.
 * 4. The action item is completed on the action items page.
 * Plus: game plan and post-game handle a bad draft_id gracefully.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, DRAFT_SEARCH, draftSlot, filterRealErrors, navigateTo, selectChampion } from "./helpers";

const CHAMPS = ["Aatrox", "Ahri", "Akali", "Alistar", "Amumu", "Anivia", "Annie", "Ashe", "Azir", "Bard"];

test("full pipeline: keyboard draft -> linked plan -> linked review -> action item done", async ({
  teamPage,
}) => {
  const page = teamPage;
  const ts = Date.now();
  const errors = captureErrors(page);
  const draftName = `PipeDraft_${ts}`;
  const planName = `PipePlan_${ts}`;
  const actionText = `Ward the river at 3:00 ${ts}`;

  // --- 1. Draft with the keyboard ---
  await navigateTo(page, "/draft");
  await page.locator("#draft-name").fill(draftName);
  await page.locator(draftSlot(0)).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(DRAFT_SEARCH)).toBeFocused();
  for (const name of CHAMPS) {
    await selectChampion(page, name);
  }
  for (let i = 0; i < CHAMPS.length; i++) {
    await expect(page.locator(draftSlot(i))).toHaveAttribute("data-filled", "true");
  }
  await expect(page.locator("[data-draft-status]")).toContainText("Pick 5 of 10");

  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });
  await expect(page).toHaveURL(/draft_id=/);
  const draftUrl = page.url();
  const draftId = new URL(draftUrl).searchParams.get("draft_id");

  // Reopen: a fresh load shows the same name and filled steps.
  await navigateTo(page, new URL(draftUrl).pathname + new URL(draftUrl).search);
  await expect(page.locator("#draft-name")).toHaveValue(draftName, { timeout: 10000 });
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved");
  for (let i = 0; i < CHAMPS.length; i++) {
    await expect(page.locator(draftSlot(i))).toHaveAttribute("data-filled", "true");
  }

  // --- 2. A game plan linked to the draft ---
  await page.getByRole("button", { name: "Prep a game plan" }).click();
  await page.waitForURL(/\/game-plan\?draft_id=/, { timeout: 10000 });
  await page.waitForLoadState("networkidle");
  await expect(page.locator("#plan-draft")).toHaveValue(draftId ?? "", { timeout: 10000 });
  await expect(page.getByRole("button", { name: "Open the draft" })).toBeVisible();
  await page.locator("#plan-name").fill(planName);
  await page.getByTestId("plan-save").click();
  await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 10000 });
  await expect(page.getByTestId("plan-list").getByText(planName)).toBeVisible();

  // --- 3. A review linked to the plan ---
  await page.getByRole("button", { name: "Start the review" }).click();
  await page.waitForURL(/\/post-game\?review_id=/, { timeout: 10000 });
  await page.waitForLoadState("networkidle");
  await expect(page.getByText(`Plan: ${planName}`)).toBeVisible({ timeout: 10000 });
  await expect(page.locator("#review-plan option:checked")).toHaveText(planName);
  await page.locator("#review-actions").fill(actionText);
  await page.getByTestId("review-save").click();
  await expect(page.getByText("1 action item created")).toBeVisible({ timeout: 10000 });

  // --- 4. Complete the action item ---
  await page.getByRole("link", { name: "Open action items" }).click();
  await page.waitForURL(/\/action-items/);
  await page.waitForLoadState("networkidle");
  const item = page.locator("[data-action-item]", { hasText: actionText });
  await expect(item).toHaveAttribute("data-status", "open", { timeout: 10000 });
  await expect(item.getByRole("link", { name: "From a review" })).toBeVisible();
  await item.getByRole("checkbox", { name: actionText }).check();
  await expect(page.locator("[data-action-item]", { hasText: actionText })).toHaveAttribute(
    "data-status",
    "done",
    { timeout: 10000 }
  );

  await expect(page.locator('nav[aria-label="Primary"]')).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("game plan page loads cleanly with draft_id query param", async ({
  authedPage,
}) => {
  const page = authedPage;
  const errors = captureErrors(page);

  // Navigate to game plan with a non-existent draft_id — should not crash
  await page.goto("/game-plan?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  expect(page.url()).toContain("/game-plan");
  const bodyText = await page.textContent("body");
  expect(bodyText).toMatch(/game|plan|strategy/i);
  await expect(page.locator('nav[aria-label="Primary"]')).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("post-game page loads cleanly with draft_id query param", async ({
  authedPage,
}) => {
  const page = authedPage;
  const errors = captureErrors(page);

  await page.goto("/post-game?draft_id=draft:nonexistent");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(1000);

  expect(page.url()).toContain("/post-game");
  const bodyText = await page.textContent("body");
  expect(bodyText).toMatch(/post|game|review/i);
  await expect(page.locator('nav[aria-label="Primary"]')).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});
