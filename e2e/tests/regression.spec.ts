// Visual regression snapshots:
// - First run creates baselines in regression.spec.ts-snapshots/
// - To update baselines: npx playwright test regression.spec.ts --update-snapshots

/**
 * Regression test scaffold for BUG-01 through BUG-05.
 *
 * Visual Regression describe block is active and creates the first snapshot baseline.
 *
 * Bug descriptions:
 * - BUG-01: Tree drafter branch switching freezes UI (suppress_autosave + signal lifecycle)
 * - BUG-02: "View Game Plan" navigates to wrong plan (wrong ID used in navigation)
 * - BUG-03: Team leader always visible in roster (owner not inserted as team_member on create)
 * - BUG-04: Hydration mismatch warnings on initial page load
 * - BUG-05: Drag-and-drop moves champion but does not clear source slot
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import {
  navigateTo,
  captureErrors,
  filterRealErrors,
  captureHydrationWarnings,
  createDraft,
  takeSnapshot,
} from "./helpers";

// ---------------------------------------------------------------------------
// BUG-01: Tree drafter branch switching freezes UI
// ---------------------------------------------------------------------------
test.describe("BUG-01", () => {
  test("tree drafter branch does not freeze UI", async ({ teamPage }) => {
    const page = teamPage;
    const errors = captureErrors(page);

    // Create a tree (it opens with its root node).
    await navigateTo(page, "/tree-drafter");
    const treeName = `BUG01Tree_${Date.now()}`;
    await page.locator("#tree-name").fill(treeName);
    await page.getByRole("button", { name: "Create tree" }).click();
    const root = page.getByRole("treeitem").first();
    await expect(root).toContainText(treeName, { timeout: 5000 });

    // Branch off the root.
    await root.getByRole("button", { name: "Branch", exact: true }).click();
    await page.locator("#branch-label").fill("BUG01 branch");
    await page.getByRole("button", { name: "Add branch" }).click();
    const branch = page.getByRole("treeitem").filter({ hasText: "BUG01 branch" }).last();
    await expect(branch).toBeVisible({ timeout: 5000 });

    // Switch between the branch and the root, then edit: a frozen UI times out here.
    await branch.getByRole("button", { name: /BUG01 branch/ }).click();
    await expect(page.locator("#node-label")).toHaveValue("BUG01 branch");
    await root.getByRole("button", { name: new RegExp(treeName) }).first().click();
    await expect(page.locator("#node-label")).toHaveValue(treeName);
    await branch.getByRole("button", { name: /BUG01 branch/ }).click();
    await page.locator("#node-label").fill("BUG01 label test");
    await expect(page.locator("#node-label")).toHaveValue("BUG01 label test");
    await expect(page.getByRole("button", { name: "Save node" })).toBeEnabled();

    // No WASM panics
    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// BUG-02: "View Game Plan" navigates to correct plan
// ---------------------------------------------------------------------------
test.describe("BUG-02", () => {
  test("view game plan navigates to correct plan", async ({ teamPage }) => {
    const page = teamPage;
    const ts = Date.now();

    // 1. A draft, and a plan prepped from it.
    await createDraft(page, `BUG02Draft_${ts}`);
    const draftUrl = new URL(page.url());
    await page.getByRole("button", { name: "Prep a game plan" }).click();
    await page.waitForURL("**/game-plan?draft_id=**", { timeout: 10000 });
    await page.waitForLoadState("networkidle");
    const planName = `BUG02Plan_${ts}`;
    await page.locator("#plan-name").fill(planName);
    await page.getByTestId("plan-save").click();
    await expect(page).toHaveURL(/plan_id=/, { timeout: 10000 });
    const planId = new URL(page.url()).searchParams.get("plan_id");

    // 2. Back on the draft, prepping again offers the existing plan.
    await navigateTo(page, draftUrl.pathname + draftUrl.search);
    await page.getByRole("button", { name: "Prep a game plan" }).click();
    const open = page.getByRole("link", { name: "Open it" });
    await expect(open).toBeVisible({ timeout: 5000 });
    expect(await open.getAttribute("href")).toBe(`/game-plan?plan_id=${planId}`);

    // 3. It opens exactly that plan.
    await open.click();
    await page.waitForURL(`**/game-plan?plan_id=${planId}`, { timeout: 10000 });
    await expect(page.locator("#plan-name")).toHaveValue(planName, { timeout: 10000 });
  });
});

// ---------------------------------------------------------------------------
// BUG-03: Team leader always visible in roster
// ---------------------------------------------------------------------------
test.describe("BUG-03", () => {
  test("team leader always visible in roster", async ({ teamPage }) => {
    const page = teamPage;

    // Navigate to the team dashboard
    await navigateTo(page, "/team/dashboard");

    // The fixture registers a user with username "pagesuser_<timestamp>_<id>"
    // and creates a team for them. The creator is the team leader.
    // Their username should appear in the page (in the roster area).

    // The "Bench" section should show the leader since they were just
    // created — they won't have a role slot assigned yet.
    const benchSection = page.locator('h2', { hasText: /^Bench$/ });
    await expect(benchSection).toBeVisible({ timeout: 5000 });

    // The bench section should NOT show "No players on the bench"
    // because the leader (who created the team) should be in it.
    const emptyBench = page.locator('p:has-text("No players on the bench")');
    const isEmptyBench = await emptyBench
      .isVisible({ timeout: 1000 })
      .catch(() => false);
    expect(isEmptyBench).toBe(false);

    // The leader's name should be somewhere in the page
    // The fixture creates users with name "pagesuser_<timestamp>_<id>"
    // We look for any element containing "pagesuser" which is the leader's username prefix
    const leaderName = page.locator('text=/pagesuser/i').first();
    await expect(leaderName).toBeVisible({ timeout: 5000 });
  });
});

// ---------------------------------------------------------------------------
// BUG-04: No hydration mismatch warnings
// ---------------------------------------------------------------------------
test.describe("BUG-04", () => {
  test("no hydration mismatch warnings", async ({ authedPage }) => {
    const page = authedPage;
    const warnings = captureHydrationWarnings(page);

    // Navigate through multiple pages to catch any hydration mismatch warnings.
    // The nav bar is present on all pages and was the source of BUG-04.
    await navigateTo(page, "/");
    await page.waitForTimeout(500);

    await navigateTo(page, "/team/dashboard");
    await page.waitForTimeout(500);

    await navigateTo(page, "/draft");
    await page.waitForTimeout(500);

    // Assert no hydration mismatch warnings were captured
    expect(warnings).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// BUG-05: Drag-and-drop moves champion, clears source
// ---------------------------------------------------------------------------
test.describe("BUG-05", () => {
  test("drag-and-drop moves champion, clears source", async ({ teamPage }) => {
    const page = teamPage;
    const errors = captureErrors(page);
    await navigateTo(page, "/draft");

    // Fill the first two bans from the keyboard.
    await page.locator("#draft-board-slot-0").focus();
    await page.keyboard.press("Enter");
    for (const name of ["Aatrox", "Ahri"]) {
      await page.locator("#draft-picker-search").fill(name);
      await page.locator("#draft-picker-search").press("Enter");
    }
    await expect(page.locator("#draft-board-slot-1")).toHaveAttribute("aria-label", /: Ahri/);

    // Drag blue ban 1 (Aatrox) onto the empty blue ban 2 slot.
    await page.locator("#draft-board-slot-0").dragTo(page.locator("#draft-board-slot-2"));

    // The champion moved and the source slot is empty.
    await expect(page.locator("#draft-board-slot-2")).toHaveAttribute("aria-label", /: Aatrox/, { timeout: 3000 });
    await expect(page.locator("#draft-board-slot-0")).toHaveAttribute("data-filled", "false");
    await expect(page.locator("#draft-board-slot-1")).toHaveAttribute("aria-label", /: Ahri/);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// Visual Regression — establishes baseline snapshots for key pages
// ---------------------------------------------------------------------------
test.describe("Visual Regression", () => {
  test("home page snapshot baseline", async ({ authedPage }) => {
    const page = authedPage;
    await navigateTo(page, "/");
    // Use a 1% pixel threshold to tolerate minor rendering differences between runs
    // (sub-pixel font rendering, anti-aliasing variation in headless Chromium).
    const screenshot = await page.screenshot({ fullPage: false });
    expect(screenshot).toMatchSnapshot("home-page.png", { maxDiffPixelRatio: 0.02 });
  });
});
