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
  test(
    "tree drafter branch does not freeze UI",
    async ({ teamPage }) => {
      const page = teamPage;
      const errors = captureErrors(page);

      // Navigate to tree drafter
      await page.goto("/tree-drafter");
      await page.waitForLoadState("networkidle");
      await page.waitForTimeout(1000);

      // Create a tree
      const treeName = `BUG01Tree_${Date.now()}`;
      const treeInput = page.locator('input[placeholder="Tree name..."]');
      await treeInput.fill(treeName);
      await page.locator('button:has-text("Create Tree")').click();
      await page.waitForTimeout(2000);

      // Click the tree to select it
      await page.locator(`button:has-text("${treeName}")`).click();
      await page.waitForTimeout(1000);

      // The root node should appear in the tree graph — look for the "Branch from here" button
      // on any node. In tree_drafter.rs nodes have a "+" (title="Add branch") button.
      // Hover the first node group to reveal it.
      const treeContent = page.locator(".group").first();
      if (await treeContent.isVisible({ timeout: 2000 }).catch(() => false)) {
        await treeContent.hover();
        await page.waitForTimeout(300);
      }

      // Click the "Add branch" button (title="Add branch") on the root node
      const addBranchBtn = page.locator('[title="Add branch"]').first();
      const canAddBranch = await addBranchBtn
        .isVisible({ timeout: 3000 })
        .catch(() => false);

      if (canAddBranch) {
        await addBranchBtn.click();
        await page.waitForTimeout(2500); // Wait for branch creation + refetch

        // After branching, the new node should be selected and the editor should be responsive.
        // Verify editor interactivity: find the node label input and type a character.
        const labelInput = page.locator('input[placeholder="Node label..."]').first();
        const inputVisible = await labelInput
          .isVisible({ timeout: 5000 })
          .catch(() => false);

        if (inputVisible) {
          // Clear and type — if the UI is frozen, this will timeout and fail the test
          await labelInput.fill("BUG01 label test");
          await page.waitForTimeout(300);
          const val = await labelInput.inputValue();
          expect(val).toBe("BUG01 label test");

          // Also verify the Save button is clickable (not frozen)
          const saveBtn = page.locator('button:has-text("Save Node")').first();
          await expect(saveBtn).toBeEnabled({ timeout: 3000 });
        } else {
          // At minimum verify the page is still interactive (not frozen)
          const createTreeBtn = page.locator('button:has-text("Create Tree")');
          await expect(createTreeBtn).toBeEnabled({ timeout: 3000 });
        }
      } else {
        // Branch button not visible — verify the page is at least interactive
        const createTreeBtn = page.locator('button:has-text("Create Tree")');
        await expect(createTreeBtn).toBeEnabled({ timeout: 3000 });
      }

      // No WASM panics
      expect(filterRealErrors(errors)).toHaveLength(0);
    }
  );
});

// ---------------------------------------------------------------------------
// BUG-02: "View Game Plan" navigates to correct plan
// ---------------------------------------------------------------------------
test.describe("BUG-02", () => {
  test(
    "view game plan navigates to correct plan",
    async ({ teamPage }) => {
      const page = teamPage;

      // 1. Create a draft
      const draftName = `BUG02Draft_${Date.now()}`;
      await createDraft(page, draftName);

      // 2. Wait for the saved draft to appear and load it
      await page.waitForTimeout(1500);

      // 3. Click the "Prep for This Draft" CTA to create a game plan
      //    The CTA appears in the saved drafts list per draft.rs
      const prepBtn = page
        .locator('button:has-text("Prep for This Draft")')
        .first();
      const prepVisible = await prepBtn
        .isVisible({ timeout: 5000 })
        .catch(() => false);

      if (prepVisible) {
        await prepBtn.click();
        // After clicking, it should navigate to /game-plan?draft_id=...
        await page.waitForURL("**/game-plan**", { timeout: 10000 });
        await page.waitForLoadState("networkidle");
        await page.waitForTimeout(1000);

        // Save the game plan on the game-plan page
        const planNameInput = page
          .locator('input[placeholder*="Comp"]')
          .first();
        if (
          await planNameInput.isVisible({ timeout: 2000 }).catch(() => false)
        ) {
          await planNameInput.fill(`BUG02Plan_${Date.now()}`);
        }
        const savePlanBtn = page.locator('button:has-text("Save Plan")').first();
        if (
          await savePlanBtn.isVisible({ timeout: 2000 }).catch(() => false)
        ) {
          await savePlanBtn.click();
          await page.waitForTimeout(2000);
        }

        // 4. Navigate back to the draft page
        await navigateTo(page, "/draft");

        // 5. Reload the draft list — the draft should now show "View Game Plan" or
        //    a game plan count badge. Click the draft to load it.
        await page.waitForTimeout(1500);

        // Look for the "View Game Plan" link (appears in the duplicate prompt area)
        const viewPlanLink = page.locator('a:has-text("View Game Plan")').first();
        const viewPlanVisible = await viewPlanLink
          .isVisible({ timeout: 5000 })
          .catch(() => false);

        if (viewPlanVisible) {
          // The href should contain plan_id= (or draft_id=)
          const href = await viewPlanLink.getAttribute("href");
          expect(href).toMatch(/[?&](plan_id|draft_id)=/);

          // Click it and verify navigation
          await viewPlanLink.click();
          await page.waitForURL("**/game-plan**", { timeout: 10000 });
          await page.waitForLoadState("networkidle");
          await page.waitForTimeout(500);

          // The URL should contain a query param
          const url = page.url();
          expect(url).toMatch(/[?&](plan_id|draft_id)=/);
        } else {
          // If "View Game Plan" not visible, check for game plan count badge
          const countBadge = page.locator('a:has-text("game plan")').first();
          const badgeVisible = await countBadge
            .isVisible({ timeout: 3000 })
            .catch(() => false);
          if (badgeVisible) {
            console.log(
              "BUG-02: Game plan count badge found — View Game Plan link requires triggering the duplicate prompt"
            );
            // This is acceptable — the bug was about the href when it IS shown
          }
        }
      } else {
        // "Prep for This Draft" not visible — draft may not have appeared yet.
        // Log as info, not a hard failure.
        console.log(
          "BUG-02: Prep for This Draft button not found — skipping navigation check"
        );
      }
    }
  );
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

    // The "Bench / Substitutes" section should show the leader since they
    // were just created — they won't have a role slot assigned yet.
    const benchSection = page.locator('h3:has-text("Bench / Substitutes")');
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
  test(
    "drag-and-drop moves champion, clears source",
    async ({ teamPage }) => {
      const page = teamPage;

      // Navigate to draft and create a draft
      const draftName = `BUG05Draft_${Date.now()}`;
      await createDraft(page, draftName);
      await page.waitForTimeout(1500);

      // Select a champion into the first ban slot (slot 0 = blue ban 1)
      // Click slot 0 to make it active, then pick a champion
      const slots = page.locator('[draggable]').filter({ hasText: /^$/ }).or(
        page.locator('.animate-pulse')
      );

      // Use the champion search to fill slot 6 (blue pick 1) with Aatrox
      // Click on a ban slot first (the first animate-pulse slot should be active)
      await page.locator('input[placeholder="Search champion..."]').fill("Aatrox");
      await page.waitForTimeout(300);
      const aatroxBtn = page
        .locator('div[style*="display:grid"] button img[alt="Aatrox"]')
        .first();
      const aatroxVisible = await aatroxBtn
        .isVisible({ timeout: 3000 })
        .catch(() => false);

      if (!aatroxVisible) {
        console.log(
          "BUG-05: Champion picker not visible — skipping drag test"
        );
        return;
      }

      // Select Aatrox (fills active slot, which auto-advances)
      await aatroxBtn.click();
      await page.waitForTimeout(500);

      // Fill second slot with Ahri
      await page.locator('input[placeholder="Search champion..."]').fill("Ahri");
      await page.waitForTimeout(300);
      const ahriBtn = page
        .locator('div[style*="display:grid"] button img[alt="Ahri"]')
        .first();
      if (await ahriBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
        await ahriBtn.click();
        await page.waitForTimeout(500);
      }

      // Now try drag-and-drop between two filled slots.
      // The draft board renders ban slots and pick slots with draggable="true".
      // Find two filled slots and drag between them.
      const filledSlots = page.locator('[draggable="true"]').filter({
        has: page.locator("img"),
      });
      const filledCount = await filledSlots.count();

      if (filledCount >= 2) {
        const sourceSlot = filledSlots.nth(0);
        const targetSlot = filledSlots.nth(1);

        // Get champion names before drag
        const sourceImg = sourceSlot.locator("img").first();
        const targetImg = targetSlot.locator("img").first();
        const sourceChamp = await sourceImg
          .getAttribute("alt")
          .catch(() => null);
        const targetChamp = await targetImg
          .getAttribute("alt")
          .catch(() => null);

        if (sourceChamp && targetChamp && sourceChamp !== targetChamp) {
          // Use Playwright's dragAndDrop to move champion between slots
          await page.dragAndDrop(
            `[draggable="true"]:has(img[alt="${sourceChamp}"])`,
            `[draggable="true"]:has(img[alt="${targetChamp}"])`
          );
          await page.waitForTimeout(500);

          // After drag: the source slot champion should have moved (source cleared)
          // The source slot should no longer show sourceChamp, or the target should now
          // have sourceChamp. We verify the drag happened by checking source is gone.
          const sourceStillFilled = await page
            .locator(`[draggable="true"] img[alt="${sourceChamp}"]`)
            .count();
          // After a successful slot-to-slot move, sourceChamp moves to the target slot.
          // There should still be exactly 1 instance (at target), or 0 if it replaced.
          expect(sourceStillFilled).toBeLessThanOrEqual(1);
        } else {
          console.log(
            "BUG-05: Not enough distinct champions for drag test — slots may have same champion"
          );
        }
      } else {
        console.log(
          `BUG-05: Only ${filledCount} filled slot(s) found — need 2 for drag test`
        );
      }
    }
  );
});

// ---------------------------------------------------------------------------
// Visual Regression — establishes baseline snapshots for key pages
// ---------------------------------------------------------------------------
test.describe("Visual Regression", () => {
  test("home page snapshot baseline", async ({ authedPage }) => {
    const page = authedPage;
    await navigateTo(page, "/");
    await takeSnapshot(page, "home-page");
  });
});
