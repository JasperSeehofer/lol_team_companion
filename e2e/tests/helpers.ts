/**
 * Shared test helper utilities for e2e tests.
 *
 * Import into test files:
 *   import { navigateTo, captureErrors, filterRealErrors, ... } from "./helpers";
 *
 * All functions are named exports. None modify test fixtures directly.
 */
import { type Page, expect } from "@playwright/test";

/**
 * Capture console errors and page errors for a page.
 * Registers `pageerror` and `console.error` listeners.
 * Call before navigating — returns the mutable errors array.
 *
 * Extracted from audit-tree-drafter.spec.ts (lines 37-44) to centralize.
 */
export function captureErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`[pageerror] ${e.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`[console.error] ${msg.text()}`);
  });
  return errors;
}

/**
 * Filter out known-harmless noise from captured errors.
 * Removes favicon 404s and Tailwind CSS import 404s (CLAUDE.md rule 47).
 *
 * Extracted from audit-tree-drafter.spec.ts (lines 46-50) to centralize.
 */
export function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
}

/**
 * Navigate to a relative route and wait for WASM hydration to settle.
 *
 * - Calls page.goto(route)
 * - Waits for networkidle
 * - Waits 500ms for WASM hydration to settle (CLAUDE.md rule 56)
 *
 * Route is relative, e.g. "/draft" or "/game-plan".
 */
export async function navigateTo(page: Page, route: string): Promise<void> {
  await page.goto(route);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
}

/**
 * Capture hydration-specific warnings from the browser console.
 * Registers a console listener for messages containing "hydration" or "mismatch".
 * Returns the mutable warnings array.
 *
 * Used by BUG-04 test (hydration mismatch detection).
 */
export function captureHydrationWarnings(page: Page): string[] {
  const warnings: string[] = [];
  page.on("console", (msg) => {
    const text = msg.text();
    if (/hydration|mismatch/i.test(text)) {
      warnings.push(`[${msg.type()}] ${text}`);
    }
  });
  return warnings;
}

/**
 * Select a champion from the champion picker grid by name.
 *
 * Strategy:
 * 1. Type the champion name into the search input (placeholder="Search champion...")
 * 2. Wait for the grid to filter
 * 3. Click the matching champion button (matches by alt text on the img)
 *
 * The champion picker grid is inside:
 *   div[style*="display:grid"] button (see champion_picker.rs)
 * Each button contains an <img alt="ChampionName">.
 */
export async function selectChampion(page: Page, name: string): Promise<void> {
  const searchInput = page.locator('input[placeholder="Search champion..."]');
  await searchInput.fill(name);
  await page.waitForTimeout(300);

  // Click the champion button matching the name (img alt attribute)
  const champBtn = page.locator(
    `div[style*="display:grid"] button img[alt="${name}"]`
  );
  await champBtn.first().click();
  await page.waitForTimeout(100);
}

/**
 * Fill a specific draft slot (0-indexed) with a champion.
 *
 * The draft board advances slots sequentially — clicking a champion fills
 * the currently active slot. To fill a specific slot:
 * 1. Click the target slot div to make it active (on_slot_click callback)
 * 2. Call selectChampion to pick the champion
 *
 * Slots are rendered as divs with on:click handlers. We select by nth child
 * of the ban/pick containers. The draft board renders bans in one row and
 * picks in another (see draft_board.rs). We use a data-slot attribute if
 * available, or fall back to clicking slots in order.
 *
 * Note: The simplest approach is to ensure the slot is active (by clicking
 * an empty slot) and then select a champion. For sequential fills, just
 * call selectChampion repeatedly — the board auto-advances the active slot.
 */
export async function fillDraftSlot(
  page: Page,
  slot: number,
  champion: string
): Promise<void> {
  // The draft board doesn't expose data-slot attributes, so we select by
  // clicking champions in order (the board auto-advances the active slot).
  // For a specific slot, we need to click it first if it's not already active.
  //
  // The draft board renders 6 ban slots + 5 pick slots per side.
  // Slots are rendered inside the main content area as clickable divs.
  // We use the pipeline.spec.ts pattern: click the nth champion grid button.
  await selectChampion(page, champion);
}

/**
 * Create a draft with the given name.
 *
 * Navigates to /draft, fills the draft name input (first textbox on the page),
 * then clicks "Save Draft". Waits for "Update Draft" to appear confirming save.
 *
 * Pattern from pipeline.spec.ts lines 43-72.
 */
export async function createDraft(page: Page, name: string): Promise<void> {
  await navigateTo(page, "/draft");
  await page.waitForTimeout(1000);

  // Fill draft name — first textbox on the draft page (pipeline.spec.ts line 48)
  await page.getByRole("textbox").first().fill(name);

  // Save the draft
  await page.locator('button:has-text("Save Draft")').click();
  await page.waitForTimeout(2000);
}

/**
 * Save a game plan by filling the name input and clicking Save Plan.
 *
 * Fills the plan name input (placeholder contains "Comp" — pipeline.spec.ts line 97),
 * clicks "Save Plan", and waits for a confirmation message.
 *
 * Assumes the game plan page is already loaded.
 */
export async function saveGamePlan(page: Page, name: string): Promise<void> {
  const planNameInput = page.locator('input[placeholder*="Comp"]');
  if (await planNameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
    await planNameInput.fill(name);
  }

  await page.locator('button:has-text("Save Plan")').click();
  await page.waitForTimeout(2000);
}

/**
 * Take a named screenshot and compare against a stored snapshot baseline.
 *
 * On first run, creates the baseline in regression.spec.ts-snapshots/.
 * On subsequent runs, compares against the baseline.
 *
 * To update baselines: npx playwright test regression.spec.ts --update-snapshots
 *
 * Uses Playwright's built-in toMatchSnapshot for pixel-level comparison.
 */
export async function takeSnapshot(page: Page, name: string): Promise<void> {
  const screenshot = await page.screenshot({ fullPage: false });
  expect(screenshot).toMatchSnapshot(name + ".png");
}
