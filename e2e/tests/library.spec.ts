/**
 * Library Room — immersive room e2e contract (Phase 28 plan 28-07).
 *
 * Coverage (per 28-VALIDATION.md success criteria):
 *
 *   SC4 — Growth tiers: data-records-tier / data-tomes-tier / data-analysis-tier
 *          on .room-atmosphere in 0..=3; seeded team ≥ 1 on at least one axis.
 *          Part B (gap-closing): tier ≥ 1 axis renders computed overlay opacity > 0;
 *          tier-0 axis stays at opacity 0.
 *
 * This is the test that would have caught the Phase 27 SC3 CSS gap (data-*-tier
 * attributes were wired but the input.css selectors were missing → overlays were
 * invisible).  Plan 28-05 adds the 9 Library CSS selectors; this spec asserts they
 * are effective in a real browser.
 *
 * NOTE on records axis e2e seeding: the records axis (data-records-tier) counts
 * distinct matches associated with team members via player_match rows.  These rows
 * are written by store_matches_with_synced_by, which requires a real Riot API
 * response — there is no UI path to seed player_match rows without a live API key.
 * Consequently the records axis is expected to be tier 0 in this e2e test.
 * The correctness of the records COUNT QUERY itself (CR-01 fix: team_member JOIN
 * player_match → distinct match count) is covered at the unit level in
 * src/server/db.rs — see the `library_records_counts_distinct_matches_via_player_match`
 * and `library_records_zero_when_no_team_members` test cases.
 *
 * Part B asserts the CSS wiring: a tier-0 records overlay stays at opacity 0
 * (guards the CSS-gap regression), and tier ≥ 1 tomes/analysis overlays are
 * visible (guards the active-tier rendering path).
 *
 * attachPanicListeners and PANIC_REGEX are intentionally kept out of fixtures.ts —
 * per D-06 of hydration-no-panic.spec.ts, the listener overhead should not leak
 * into tests that don't need it.
 */

import { test, expect, seedLibraryTier1Team, createTeam } from "./fixtures";

// ---------------------------------------------------------------------------
// Library test suite
// ---------------------------------------------------------------------------

test.describe("Library", () => {
  // SC4 — Growth tiers (seeded team + tier→opacity CSS assertion)
  test("growth tiers", async ({ authedPage }) => {
    // --- Part A: seeded team should push at least one axis to tier ≥ 1 ---
    //
    // seedLibraryTier1Team: creates a post-game review →
    //   tomes_tier = tier_for(post_game_learning + team_note, [1,5,15]) = 1
    //   analysis_tier = tier_for(post_game_learning, [1,5,15]) = 1
    //
    // records_tier stays 0: seeding player_match rows requires a Riot API
    // response; no e2e-reachable UI path exists without a live API key.
    // The records COUNT QUERY correctness is verified by unit tests in db.rs:
    //   - library_records_counts_distinct_matches_via_player_match
    //   - library_records_zero_when_no_team_members
    await createTeam(authedPage, `LibrarySeed_${Date.now()}`);
    await seedLibraryTier1Team(authedPage);

    await authedPage.goto("/library");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(800); // growth resource resolve

    const root = authedPage.locator(".room-atmosphere");

    // All three axes must be present and in valid range 0..=3.
    for (const axis of ["records", "tomes", "analysis"]) {
      const val = await root.getAttribute(`data-${axis}-tier`);
      expect(val, `data-${axis}-tier present`).not.toBeNull();
      const n = parseInt(val ?? "", 10);
      expect(Number.isNaN(n), `data-${axis}-tier is numeric`).toBe(false);
      expect(n, `data-${axis}-tier ≥ 0`).toBeGreaterThanOrEqual(0);
      expect(n, `data-${axis}-tier ≤ 3`).toBeLessThanOrEqual(3);
    }

    // At least one axis must be ≥ 1 (the seed guarantee).
    const recordsVal = await root.getAttribute("data-records-tier");
    const tomesVal = await root.getAttribute("data-tomes-tier");
    const analysisVal = await root.getAttribute("data-analysis-tier");
    const maxTier = Math.max(
      parseInt(recordsVal ?? "0", 10),
      parseInt(tomesVal ?? "0", 10),
      parseInt(analysisVal ?? "0", 10)
    );
    expect(maxTier, "seeded team has at least one axis at tier ≥ 1").toBeGreaterThanOrEqual(1);

    // --- Part B (SC4 "visibly"): tier must translate to overlay opacity ---
    // Guards against the data-attrs-wired-but-no-CSS regression (the Phase 27
    // SC3 blind spot): an axis at tier ≥ 1 must have a visible overlay
    // (opacity > 0); a tier-0 axis stays hidden (opacity 0).
    const axisTiers: Record<string, number> = {
      records: parseInt(recordsVal ?? "0", 10),
      tomes: parseInt(tomesVal ?? "0", 10),
      analysis: parseInt(analysisVal ?? "0", 10),
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
});
