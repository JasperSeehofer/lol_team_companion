/**
 * Phase 17 plan 17-07 — Final visual-regression baseline capture.
 * Phase 18 plan 18-09 — Refactored: scoped routes now use subfolder paths
 * per D-07/D-08/D-09 (region × mode combinations).
 *
 * Captures pixel baselines for every restyled route.
 * First run captures baselines (`--update-snapshots`). Subsequent runs diff
 * against baseline with `maxDiffPixelRatio: 0.02` (2% leeway for font
 * subpixel rendering, animation frames, anti-aliasing variance).
 *
 * Route taxonomy:
 *   Public:  /closed-beta, /auth/login, /auth/register (with invite query),
 *            /legal/impressum, /legal/datenschutz
 *   Utility authed (flat baselines, Phase 17, unchanged):
 *            /profile, /team/roster, /team-builder, /game-plan,
 *            /opponents, /action-items, /stats, /analytics,
 *            /personal-learnings, /admin/invites
 *   Scoped authed (subfolder baselines, Phase 18):
 *            /draft (3 modes), /solo (3 modes), /team/dashboard,
 *            /tree-drafter, /champion-pool, /match/:id, /post-game
 *
 * M0: the Pandemonium theme and the Game Day Brief were removed, so their
 * baselines are gone. Remaining baselines predate the M0 footer and must be
 * re-captured (`--update-snapshots`) once the M2 design system lands.
 */
import { test, expect, setRegion, setMode } from "./fixtures";

const VR_OPTS = {
  maxDiffPixelRatio: 0.02,
  fullPage: true,
} as const;

// ---------------------------------------------------------------------------
// Public routes — no auth required
// ---------------------------------------------------------------------------

test.describe("public routes", () => {
  test("/closed-beta visual baseline", async ({ page }) => {
    await page.goto("/closed-beta");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
    await expect(page).toHaveScreenshot("public-closed-beta.png", VR_OPTS);
  });

  test("/auth/login visual baseline", async ({ page }) => {
    await page.goto("/auth/login");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
    await expect(page).toHaveScreenshot("public-auth-login.png", VR_OPTS);
  });

  test("/auth/register?invite=TEST123 visual baseline", async ({ page }) => {
    await page.goto("/auth/register?invite=TEST123");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
    await expect(page).toHaveScreenshot("public-auth-register-invited.png", VR_OPTS);
  });

  test("/legal/impressum visual baseline", async ({ page }) => {
    await page.goto("/legal/impressum");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
    await expect(page).toHaveScreenshot("public-legal-impressum.png", VR_OPTS);
  });

  test("/legal/datenschutz visual baseline", async ({ page }) => {
    await page.goto("/legal/datenschutz");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
    await expect(page).toHaveScreenshot("public-legal-datenschutz.png", VR_OPTS);
  });
});

// ---------------------------------------------------------------------------
// Utility auth-required routes — flat baselines, unchanged from Phase 17
// ---------------------------------------------------------------------------

test.describe("utility auth-required routes", () => {
  test("/profile visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/profile");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-profile.png", VR_OPTS);
  });

  test("/team/roster visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/team/roster");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-team-roster.png", VR_OPTS);
  });

  test("/team-builder visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/team-builder");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-team-builder.png", VR_OPTS);
  });

  test("/game-plan visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/game-plan");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-game-plan.png", VR_OPTS);
  });

  test("/opponents visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/opponents");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-opponents.png", VR_OPTS);
  });

  test("/action-items visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/action-items");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-action-items.png", VR_OPTS);
  });

  test("/stats visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/stats");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-stats.png", VR_OPTS);
  });

  test("/analytics visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/analytics");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-analytics.png", VR_OPTS);
  });

  test("/personal-learnings visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/personal-learnings");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-personal-learnings.png", VR_OPTS);
  });

  test("/admin/invites visual baseline", async ({ authedPage }) => {
    // Non-admin users see a denied/redirect surface, which is the relevant
    // baseline for a fresh fixture user (admins are seeded separately).
    await authedPage.goto("/admin/invites");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-admin-invites.png", VR_OPTS);
  });
});

// ---------------------------------------------------------------------------
// Scoped routes — subfolder baselines per region × mode (Phase 18)
// ---------------------------------------------------------------------------

test.describe("scoped routes - /draft (3 baselines)", () => {
  test("/draft demacia carousel baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/draft");
    await setMode(authedPage, "carousel");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-draft", "demacia-carousel.png"],
      VR_OPTS
    );
  });

  test("/draft demacia war-table baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/draft");
    await setMode(authedPage, "war-table");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-draft", "demacia-war-table.png"],
      VR_OPTS
    );
  });

  test("/draft demacia ledger baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/draft");
    await setMode(authedPage, "ledger");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-draft", "demacia-ledger.png"],
      VR_OPTS
    );
  });

});

test.describe("scoped routes - /solo (3 baselines)", () => {
  test("/solo demacia constellation baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/solo");
    await setMode(authedPage, "constellation");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-solo", "demacia-constellation.png"],
      VR_OPTS
    );
  });

  test("/solo demacia forge baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/solo");
    await setMode(authedPage, "forge");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-solo", "demacia-forge.png"],
      VR_OPTS
    );
  });

  test("/solo demacia journal baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/solo");
    await setMode(authedPage, "journal");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-solo", "demacia-journal.png"],
      VR_OPTS
    );
  });

});

test.describe("scoped routes - /team/dashboard (1 baseline)", () => {
  test("/team/dashboard demacia dashboard baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/team/dashboard");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-team-dashboard", "demacia-dashboard.png"],
      VR_OPTS
    );
  });

});

test.describe("scoped routes - single-mode (4 baselines)", () => {
  test("/tree-drafter demacia baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/tree-drafter");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-tree-drafter", "demacia.png"],
      VR_OPTS
    );
  });

  test("/champion-pool demacia baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/champion-pool");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-champion-pool", "demacia.png"],
      VR_OPTS
    );
  });

  test("/match/:id demacia baseline", async ({ authedPage }) => {
    // Synthetic match ID — page renders empty/error state, which is
    // itself a baseline-worthy surface.
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/match/test-match-id");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-match-detail", "demacia.png"],
      VR_OPTS
    );
  });

  test("/post-game demacia baseline", async ({ authedPage }) => {
    await setRegion(authedPage, "demacia");
    await authedPage.goto("/post-game");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-post-game", "demacia.png"],
      VR_OPTS
    );
  });

});
