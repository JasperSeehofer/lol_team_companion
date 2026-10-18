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
  return errors.filter((e) => !isEnvironmentNoise(e));
}

/**
 * Console errors that say nothing about the app: favicon/asset 404s.
 * (M152: the old ERR_TUNNEL_CONNECTION_FAILED allowance for third-party
 * hosts such as CommunityDragon is gone. Role and objective icons come from
 * the app's own /img/ui/ route, so a blocked third-party request is now a
 * regression, not noise.)
 */
export function isEnvironmentNoise(e: string): boolean {
  return e.includes("favicon") || e.includes("404 (Not Found)");
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

/** DOM id of draft board slot `i` (0-based, tournament order). */
export const draftSlot = (i: number) => `#draft-board-slot-${i}`;

/** The draft page's champion search (the picker beside the board). */
export const DRAFT_SEARCH = "#draft-picker-search";

/**
 * Put a champion into the board's current step: type the name into the
 * picker's search and press Enter (Enter takes the first match). The board
 * then moves on to the next empty step.
 */
export async function selectChampion(page: Page, name: string): Promise<void> {
  const search = page.locator(DRAFT_SEARCH);
  await search.fill(name);
  await expect(page.locator(`#draft-picker-list li[data-champion="${name}"]`).first()).toBeVisible();
  await search.press("Enter");
}

/**
 * Fill draft slot `slot` (0-based) with a champion: focus the slot, press
 * Enter (focus moves to the picker), type and press Enter.
 */
export async function fillDraftSlot(
  page: Page,
  slot: number,
  champion: string
): Promise<void> {
  await page.locator(draftSlot(slot)).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(DRAFT_SEARCH)).toBeFocused();
  await selectChampion(page, champion);
  await expect(page.locator(draftSlot(slot))).toHaveAttribute("data-filled", "true");
}

/**
 * Create and save a draft named `name` on /draft; waits for "Saved" and the
 * draft id in the URL.
 */
export async function createDraft(page: Page, name: string): Promise<void> {
  await navigateTo(page, "/draft");
  await page.locator("#draft-name").fill(name);
  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });
  await expect(page).toHaveURL(/draft_id=/);
}

/**
 * Save the open game plan under `name` (the plan page must be loaded).
 */
export async function saveGamePlan(page: Page, name: string): Promise<void> {
  await page.locator("#plan-name").fill(name);
  await page.getByTestId("plan-save").click();
  await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 10000 });
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
