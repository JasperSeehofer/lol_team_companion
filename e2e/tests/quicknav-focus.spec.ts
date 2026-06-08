/**
 * Phase 29 Plan 02 — QuickNav focus SC2 (restore) + SC3 (trap) assertions.
 *
 * SC2 — focus-restore: After opening QuickNav via Meta+k and closing via
 *   Escape, focus returns to the element that triggered the open (the QuickNav
 *   launcher button). WCAG 2.1 SC 2.1.2 requirement.
 *
 * SC3 — focus-trap: While QuickNav overlay is open, pressing Tab cycles focus
 *   within [role="dialog"] and never escapes to the background page. After
 *   cycling ~20 times, document.activeElement must remain inside the dialog.
 *
 * Both tests are marked test.skip until Plan 29-06 lands the focus-trap +
 * focus-restore implementation in quick_nav.rs. Plan 07 un-skips them.
 * The assertion shapes are correct so Plan 07 only needs to remove the skip.
 *
 * Note: QuickNav currently opens but has no focus trap (the SC2.1.2 BLOCKER
 * documented in STATE.md). These tests encode the CORRECT final behavior so
 * the suite fails loudly once the route exists but the trap is absent.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

// Skip until Plan 29-06 ships the focus trap + restore implementation.
// Plan 07 removes this guard.
test.describe.skip("QuickNav focus (SC2/SC3) — pending Plan 29-06", () => {
  /**
   * SC2 — restore: focus returns to the QuickNav trigger after Escape.
   *
   * Steps:
   *   1. Navigate to any authenticated route (use /solo as the stable base).
   *   2. Focus the QuickNav trigger button (aria-label contains "quick navigation").
   *   3. Open via Meta+k.
   *   4. Assert [role="dialog"] is visible (overlay opened).
   *   5. Press Escape.
   *   6. Assert document.activeElement.ariaLabel matches /quick navigation/i.
   */
  test("restore: Escape returns focus to QuickNav trigger button", async ({
    authedPage,
  }) => {
    // Settle after auth before sending keyboard events (wasm-patterns rule #56).
    await authedPage.goto("/solo");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1000);

    // Focus the trigger button.
    const trigger = authedPage.locator(
      'button[aria-label*="quick navigation" i], button[aria-label*="Quick Navigation" i]'
    ).first();
    await expect(trigger).toBeVisible({ timeout: 5000 });
    await trigger.focus();

    // Open QuickNav via Meta+k.
    await authedPage.keyboard.press("Meta+k");
    // Wait for the overlay to appear.
    await expect(
      authedPage.locator('[role="dialog"]').first()
    ).toBeVisible({ timeout: 3000 });

    // Close via Escape.
    await authedPage.keyboard.press("Escape");
    await authedPage.waitForTimeout(200); // settle focus restoration

    // Assert focus returned to the trigger.
    const focusedAriaLabel = await authedPage.evaluate(
      () => (document.activeElement as HTMLElement | null)?.ariaLabel ?? ""
    );
    expect(
      focusedAriaLabel,
      "Focus should return to QuickNav trigger after Escape"
    ).toMatch(/quick navigation/i);
  });

  /**
   * SC3 — trap: Tab key cycles within the open QuickNav overlay; focus never
   * escapes to the background.
   *
   * Steps:
   *   1. Navigate to /solo and open QuickNav via Meta+k.
   *   2. Press Tab 20 times, asserting after each press that
   *      document.activeElement is a descendant of [role="dialog"].
   *   3. Close cleanly via Escape.
   */
  test("trap: Tab focus stays inside QuickNav dialog while open", async ({
    authedPage,
  }) => {
    // Settle after auth.
    await authedPage.goto("/solo");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1000);

    // Open QuickNav.
    await authedPage.keyboard.press("Meta+k");
    const dialog = authedPage.locator('[role="dialog"]').first();
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Tab through items and verify focus stays inside the dialog.
    for (let i = 0; i < 20; i++) {
      await authedPage.keyboard.press("Tab");
      await authedPage.waitForTimeout(50); // brief settle per keystroke

      const isInsideDialog = await authedPage.evaluate(() => {
        const dialog = document.querySelector('[role="dialog"]');
        const active = document.activeElement;
        if (!dialog || !active) return false;
        // document.activeElement is inside the dialog if the dialog contains it
        // or IS it (e.g. the dialog container itself received focus).
        return dialog === active || dialog.contains(active);
      });

      expect(
        isInsideDialog,
        `After Tab press ${i + 1}: focus escaped [role="dialog"]`
      ).toBe(true);
    }

    // Clean up — close the dialog.
    await authedPage.keyboard.press("Escape");
  });
});
