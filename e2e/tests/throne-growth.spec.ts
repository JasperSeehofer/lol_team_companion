/**
 * Phase 29 Plan 02 — Throne room growth SC1 / SC1b assertions.
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
 *       Marked test.fixme until a teamless+ranked seeding fixture is available
 *       (Plan 07 un-skips it) — the assertion shape is correct so Plan 07 can
 *       simply remove the fixme without rewriting the test.
 *
 * NOTE: /throne does not exist until Plan 29-05 ships. Until then these tests
 * will gracefully skip/fail-cleanly on "page not found" — they do NOT break
 * the existing green suite because they are guarded with test.skip when the
 * route 404s. Plan 07 removes the skip guards once the room is built.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect, createTeam } from "./fixtures";

// Skip the whole suite until the /throne route exists (Plan 29-05).
// Plan 07 removes this guard.
test.describe.skip("Throne room growth (SC1/SC1b) — pending Plan 29-05", () => {
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
   * Marked fixme: seeding a teamless+ranked fixture requires Riot API mock
   * or a direct DB insert for a ranked_snapshot row — not yet available.
   * Plan 07 un-skips this test by providing the seeding helper.
   */
  test.fixme(
    "SC1b: progression-tier > 0 for teamless ranked user; roster-tier == 0",
    async ({ authedPage }) => {
      // TODO (Plan 07): seed a ranked_snapshot row for the test user WITHOUT
      // creating a team. Then navigate to /throne and assert:
      //   data-progression-tier matches /[1-3]/
      //   data-roster-tier == "0"
      //   .room-overlay--progression computed opacity > 0
      //   .room-overlay--roster computed opacity == "0"

      await authedPage.goto("/throne");
      await authedPage.waitForLoadState("networkidle");
      await authedPage.waitForTimeout(1500);

      // Progression axis: user-scoped (advances with no team — D-05/D-07).
      const root = authedPage.locator("[data-progression-tier]").first();
      await expect(root).toHaveAttribute("data-progression-tier", /[1-3]/);

      // Roster axis: team-scoped (stays 0 for a teamless user).
      await expect(root).toHaveAttribute("data-roster-tier", "0");

      // Computed opacity checks.
      const progressionOverlay = authedPage
        .locator(".room-overlay--progression")
        .first();
      const progressionOpacity = await progressionOverlay.evaluate(
        (el) => getComputedStyle(el).opacity
      );
      expect(parseFloat(progressionOpacity)).toBeGreaterThan(0);

      const rosterOverlay = authedPage
        .locator(".room-overlay--roster")
        .first();
      const rosterOpacity = await rosterOverlay.evaluate(
        (el) => getComputedStyle(el).opacity
      );
      expect(parseFloat(rosterOpacity)).toBe(0);
    }
  );
});
