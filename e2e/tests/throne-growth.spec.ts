/**
 * Phase 29 Plan 02 — Throne room growth SC1 / SC1b assertions.
 * Phase 29 Plan 07 — skip guards removed; seeding helpers wired.
 *
 * SC1: Throne overlays render at computed opacity > 0 for a seeded high-tier
 *      scenario. This test deliberately asserts getComputedStyle().opacity on
 *      the overlay element itself — NOT merely the data-tier attribute — to
 *      guard against the Phase 27 "room growth visibility gap" (MEMORY.md
 *      project_room_growth_visibility_gap.md): the CSS tier→opacity selectors
 *      can silently be absent while the data-attr assertion still passes.
 *
 * SC1b: A teamless-but-ranked user yields data-progression-tier > 0 while
 *       data-roster-tier == "0". This validates the D-05/D-07 blended growth
 *       divergence: progression advances independently of team membership.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect, createTeam, seedRankedSnapshotForTest } from "./fixtures";

test.describe("Throne room growth (SC1/SC1b)", () => {
  /**
   * SC1 — seed a team (roster axis ≥ 1) then assert:
   *   1. data-roster-tier attribute matches /[1-3]/ (data-attr assertion)
   *   2. getComputedStyle(overlay).opacity > 0 (computed-opacity assertion —
   *      the Phase 27 lesson: CSS selectors absent → opacity stays 0 even with
   *      correct data-attr)
   */
  test("SC1: throne overlay computed opacity > 0 after seeding roster tier", async ({
    authedPage,
  }) => {
    // Seed: create a team so roster_tier ≥ 1 (at least one member = the creator).
    await createTeam(authedPage, `ThroneGrowthSeed_${Date.now()}`);

    // Navigate to Throne (wasm-patterns rule #56: settle delay after team creation).
    await authedPage.waitForTimeout(500);
    await authedPage.goto("/throne");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1500); // WASM hydrate settle

    // Data-attr assertion (necessary but not sufficient).
    const root = authedPage.locator("[data-roster-tier]").first();
    await expect(root).toBeVisible({ timeout: 5000 });
    await expect(root).toHaveAttribute("data-roster-tier", /[1-3]/);

    // Computed-opacity assertion (the Phase 27 seed-and-assert-opacity lesson).
    // Verifies the input.css tier→opacity selectors are wired, not just the attr.
    const overlay = authedPage.locator(".room-overlay--roster").first();
    await expect(overlay).toBeAttached({ timeout: 5000 });
    const opacity = await overlay.evaluate(
      (el) => getComputedStyle(el).opacity
    );
    expect(
      parseFloat(opacity),
      `Expected .room-overlay--roster opacity > 0 (data-roster-tier ≥ 1 was set but CSS selector may be missing)`
    ).toBeGreaterThan(0);
  });

  /**
   * SC1b — blended axes: teamless user with ranked games gets
   * data-progression-tier > 0 but data-roster-tier == "0".
   *
   * Seeding: inserts a RANKED_SOLO_5x5 snapshot via the debug-only
   * seed_ranked_snapshot_for_test server fn (throne.rs). The user
   * intentionally has NO team — the progression axis must advance
   * independently (D-05/D-07 blended growth contract).
   */
  test("SC1b: progression-tier > 0 for teamless ranked user; roster-tier == 0",
    async ({ authedPage }) => {
      // Seed a ranked snapshot for this user WITHOUT creating a team.
      // The helper calls the debug-only seed_ranked_snapshot_for_test server fn
      // which inserts Gold II, 25W/10L (35 games ≥ 20 → progression_tier = 2).
      await seedRankedSnapshotForTest(authedPage);
      await authedPage.waitForTimeout(300);

      await authedPage.goto("/throne");
      await authedPage.waitForLoadState("networkidle");
      await authedPage.waitForTimeout(1500);

      // Progression axis: user-scoped (advances with no team — D-05/D-07).
      const root = authedPage.locator("[data-progression-tier]").first();
      await expect(root).toBeVisible({ timeout: 5000 });
      await expect(root).toHaveAttribute("data-progression-tier", /[1-3]/);

      // Roster axis: team-scoped (stays 0 for a teamless user).
      await expect(root).toHaveAttribute("data-roster-tier", "0");

      // Computed opacity checks.
      const progressionOverlay = authedPage
        .locator(".room-overlay--progression")
        .first();
      await expect(progressionOverlay).toBeAttached({ timeout: 5000 });
      const progressionOpacity = await progressionOverlay.evaluate(
        (el) => getComputedStyle(el).opacity
      );
      expect(parseFloat(progressionOpacity)).toBeGreaterThan(0);

      const rosterOverlay = authedPage
        .locator(".room-overlay--roster")
        .first();
      await expect(rosterOverlay).toBeAttached({ timeout: 5000 });
      const rosterOpacity = await rosterOverlay.evaluate(
        (el) => getComputedStyle(el).opacity
      );
      expect(parseFloat(rosterOpacity)).toBe(0);
    }
  );
});
