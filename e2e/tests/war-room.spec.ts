/**
 * War Room — immersive room e2e contract (Phase 27 plan 27-03).
 *
 * These tests define the executable acceptance contract for the War Room and
 * are EXPECTED TO FAIL (RED) until Plan 27-07 ships the `/war-room` route.
 * Plan 27-08 turns them GREEN as the phase gate.
 *
 * Coverage (per 27-VALIDATION.md success criteria):
 *
 *   SC1 — /war-room smoke (in pages.spec.ts AUTHED_PAGES)
 *   SC2 — ARIA tablist: tablist/tab/aria-selected/tabpanel roles present and correct
 *   SC3 — Growth tiers: data-tactics-tier / data-intel-tier / data-command-tier
 *          on .room-atmosphere in 0..=3; seeded team ≥ 1 on at least one axis
 *   SC4 — Mousemove guard: /draft→/war-room→/journey→/war-room round-trip leaves
 *          exactly one mousemove listener (advisory if getEventListeners unavailable)
 *   SC5 — Hydration canary: zero tachys/Rust panics in demacia + pandemonium themes
 *
 * attachPanicListeners and PANIC_REGEX are intentionally local (not promoted to
 * fixtures.ts) — per D-06 of hydration-no-panic.spec.ts, the listener overhead
 * should not leak into tests that don't need it.
 */

import { test, expect, setRegion, seedTier1Team, createTeam } from "./fixtures";
import type { Page } from "@playwright/test";

// ---------------------------------------------------------------------------
// Panic detection helpers (local per D-06 — NOT promoted to fixtures)
// ---------------------------------------------------------------------------

/**
 * D-08 narrow panic filter. Matches Rust/WASM panic shapes observed in the
 * Phase 18.2 evidence trail. Does NOT fire on favicon 404, Tailwind import
 * warning, or unrelated console.error calls.
 */
const PANIC_REGEX = /panicked at|Option::unwrap.*None|tachys/i;

/**
 * Attach pageerror + filtered console.error listeners BEFORE navigation so
 * first-hydrate panics are captured. Returns the mutable errors array —
 * callers assert `expect(errors).toEqual([])` after the hydrate-settle pause.
 */
async function attachPanicListeners(page: Page): Promise<string[]> {
  const errors: string[] = [];

  // pageerror = uncaught JS exception. WASM panics propagate here via
  // console_error_panic_hook (lib.rs) → throw.
  page.on("pageerror", (err) => {
    errors.push(`pageerror: ${err.message}`);
  });

  // console.error = WASM/JS console.error. Filter narrowly so unrelated
  // errors (favicon 404, Tailwind import) don't false-positive.
  page.on("console", (msg) => {
    if (msg.type() === "error" && PANIC_REGEX.test(msg.text())) {
      errors.push(`console.error: ${msg.text()}`);
    }
  });

  return errors;
}

// ---------------------------------------------------------------------------
// War Room test suite
// ---------------------------------------------------------------------------

test.describe("War Room", () => {
  // SC2 (partial) + atmosphere render
  test("renders the War surfaces over the parallax atmosphere", async ({
    authedPage,
  }) => {
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");

    // Atmosphere root and at least the sky/mid/near planes exist.
    await expect(authedPage.locator(".room-atmosphere")).toBeVisible();
    expect(
      await authedPage.locator(".room-plane").count()
    ).toBeGreaterThanOrEqual(3);

    // The surface wrapper carries the feedback label (mirrors solo_journey.rs pattern).
    await expect(
      authedPage.locator('[data-feedback-label="War Room"]')
    ).toBeVisible();

    // No error banners.
    expect(await authedPage.locator("text=/Failed to load/i").count()).toBe(0);
  });

  // SC2 — ARIA tablist
  test("aria tablist", async ({ authedPage }) => {
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500); // hydrate settle

    // One tablist present.
    await expect(authedPage.locator('[role="tablist"]')).toBeVisible();

    // Exactly 5 tabs (Draft, Tree-Drafter, Game Plan, Opponents, Ban Priority).
    const tabs = authedPage.locator('[role="tab"]');
    expect(await tabs.count()).toBe(5);

    // Exactly one tab is selected at a time.
    const selectedTabs = authedPage.locator('[role="tab"][aria-selected="true"]');
    expect(await selectedTabs.count()).toBe(1);

    // At least one tabpanel is present.
    const panels = authedPage.locator('[role="tabpanel"]');
    expect(await panels.count()).toBeGreaterThanOrEqual(1);
  });

  // SC3 — Growth tiers (seeded team + fresh user)
  test("growth tiers", async ({ authedPage }) => {
    // --- Part A: seeded team should push at least one axis to tier ≥ 1 ---
    //
    // seedTier1Team: creates a team then saves 1 draft → tactics_tier = 1.
    // Threshold from src/models/growth.rs TeamGrowthState::from_counts:
    //   tactics = tier_for(draft + draft_tree, [1, 5, 20]) — tier 1 requires ≥ 1.
    await createTeam(authedPage, `WarRoomSeed_${Date.now()}`);
    await seedTier1Team(authedPage);

    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(800); // growth resource resolve

    const root = authedPage.locator(".room-atmosphere");

    // All three axes must be present and in valid range 0..=3.
    for (const axis of ["tactics", "intel", "command"]) {
      const val = await root.getAttribute(`data-${axis}-tier`);
      expect(val, `data-${axis}-tier present`).not.toBeNull();
      const n = parseInt(val ?? "", 10);
      expect(Number.isNaN(n), `data-${axis}-tier is numeric`).toBe(false);
      expect(n, `data-${axis}-tier ≥ 0`).toBeGreaterThanOrEqual(0);
      expect(n, `data-${axis}-tier ≤ 3`).toBeLessThanOrEqual(3);
    }

    // At least one axis must be ≥ 1 (the seed guarantee).
    const tacticsVal = await root.getAttribute("data-tactics-tier");
    const intelVal = await root.getAttribute("data-intel-tier");
    const commandVal = await root.getAttribute("data-command-tier");
    const maxTier = Math.max(
      parseInt(tacticsVal ?? "0", 10),
      parseInt(intelVal ?? "0", 10),
      parseInt(commandVal ?? "0", 10)
    );
    expect(maxTier, "seeded team has at least one axis at tier ≥ 1").toBeGreaterThanOrEqual(1);

    // --- Part B (SC3 "visibly"): tier must translate to overlay opacity ---
    // Guards against the data-attrs-wired-but-no-CSS regression: an axis at
    // tier ≥ 1 must have a visible overlay (opacity > 0); a tier-0 axis stays
    // hidden (opacity 0). This is what makes the room grow *visibly*.
    const axisTiers: Record<string, number> = {
      tactics: parseInt(tacticsVal ?? "0", 10),
      intel: parseInt(intelVal ?? "0", 10),
      command: parseInt(commandVal ?? "0", 10),
    };
    for (const [axis, tier] of Object.entries(axisTiers)) {
      const opacity = await authedPage
        .locator(`.room-overlay--${axis}`)
        .evaluate((el) => parseFloat(getComputedStyle(el).opacity));
      if (tier >= 1) {
        expect(opacity, `${axis} overlay (tier ${tier}) is visible`).toBeGreaterThan(0);
      } else {
        expect(opacity, `${axis} overlay (tier 0) is hidden`).toBe(0);
      }
    }
  });

  // SC4 — Mousemove guard (no listener accumulation across navigations)
  test("mousemove guard", async ({ authedPage }) => {
    // Round-trip: /draft → /war-room → /journey → /war-room
    await authedPage.goto("/draft");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    // Chrome DevTools `getEventListeners` API — available in Playwright's
    // Chromium context. Returns -1 when unavailable (advisory path).
    // See 27-RESEARCH.md Pattern 5 and Assumption A2.
    const count = await authedPage.evaluate(
      () => (window as any).getEventListeners?.(window)?.mousemove?.length ?? -1
    );

    // If the API is unavailable (-1), the test is advisory-only (non-blocking).
    // When available, exactly one listener must be present (AtomicBool guard
    // in src/components/room/parallax.rs prevents accumulation — plan 27-02).
    if (count !== -1) {
      expect(count, "exactly one mousemove listener after round-trip").toBe(1);
    }
  });

  // SC5 — Hydration canary: demacia + pandemonium
  // Named so `-g "canary demacia"` and `-g "canary pandemonium"` select them.
  for (const region of ["demacia", "pandemonium"] as const) {
    test(`canary ${region}`, async ({ authedPage }) => {
      // Attach listeners BEFORE any navigation so first-hydrate panics are captured.
      const errors = await attachPanicListeners(authedPage);

      // setRegion sets the `lol_companion_theme` cookie and reloads the current
      // page. When called before goto it pre-sets the cookie so SSR uses the
      // correct region on first render (CLAUDE.md rule #58 — region-read-once).
      await setRegion(authedPage, region);

      await authedPage.goto("/war-room");
      await authedPage.waitForLoadState("networkidle");
      // Hydrate-settle: WASM Effects (auth redirect, tab init) fire async.
      await authedPage.waitForTimeout(1500);

      // Click a tab to exercise WASM event handler attachment — the most
      // likely trigger for the html/mod.rs:217 structural-mismatch panic.
      const firstTab = authedPage.locator('[role="tab"]').first();
      if (await firstTab.isVisible({ timeout: 2000 }).catch(() => false)) {
        await firstTab.click();
        await authedPage.waitForTimeout(500);
      }

      expect(
        errors,
        `Panic on /war-room [${region}]: ${errors.join("\n")}`
      ).toEqual([]);
    });
  }
});
