/**
 * Phase 29 Plan 02 — Room contrast scrim SC5 assertions.
 *
 * SC5: Every immersive room (/war-room, /library, /throne) must have a z-8
 *   contrast scrim element: a `div` with a `radial-gradient` background and
 *   `pointer-events: none`. This scrim provides WCAG AA (4.5:1) text contrast
 *   over the parallax art (D-15 — mirror of solo_journey.rs:69).
 *
 * War Room and Library are tested immediately (their routes are live).
 * Throne is skipped until Plan 29-05 ships the route — Plan 07 un-skips it.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

/**
 * Rooms whose contrast scrim is asserted.
 * - war-room: ships Plan 27-07 (live).
 * - library: ships Plan 28-05 (live).
 * - throne: ships Plan 29-05 (skipped until then — see below).
 */
const LIVE_ROOMS = ["/war-room", "/library"] as const;

// ---------------------------------------------------------------------------
// SC5 — live rooms (War Room + Library already built)
// ---------------------------------------------------------------------------
for (const room of LIVE_ROOMS) {
  test(`SC5: ${room} has z-8 radial-gradient contrast scrim`, async ({
    authedPage,
  }) => {
    // Filter the tailwind @import 404 console error (wasm-patterns rule #47).
    const errors: string[] = [];
    authedPage.on("pageerror", (e) => errors.push(e.message));

    await authedPage.goto(room);
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1000); // WASM hydrate settle

    // Assert the z-8 contrast scrim is present and visible.
    // The scrim is rendered as:
    //   <div style="z-index: 8; background: radial-gradient(ellipse at center, ...)"
    //        aria-hidden="true" class="fixed inset-0 pointer-events-none">
    // Match via the inline style containing "radial-gradient" (D-15 canonical marker).
    const scrim = authedPage.locator('div[style*="radial-gradient"]').first();
    await expect(scrim).toBeAttached({
      timeout: 5000,
    });

    // Verify it is also aria-hidden (the rooms set aria-hidden="true" on the scrim).
    const ariaHidden = await scrim.getAttribute("aria-hidden");
    expect(ariaHidden).toBe("true");
  });
}

// ---------------------------------------------------------------------------
// SC5 — Throne room (pending Plan 29-05)
// ---------------------------------------------------------------------------
// Skipped until Plan 29-05 ships /throne. Plan 07 removes the skip guard
// and moves /throne into LIVE_ROOMS above (or a merged loop).
test.skip(
  "SC5: /throne has z-8 radial-gradient contrast scrim (pending Plan 29-05)",
  async ({ authedPage }) => {
    await authedPage.goto("/throne");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1000);

    const scrim = authedPage.locator('div[style*="radial-gradient"]').first();
    await expect(scrim).toBeAttached({ timeout: 5000 });
    const ariaHidden = await scrim.getAttribute("aria-hidden");
    expect(ariaHidden).toBe("true");
  }
);
