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
 *            /profile, /team/roster, /game-plan,
 *            /opponents, /action-items, /stats,
 *            /personal-learnings, /admin/invites
 *   Scoped authed (subfolder baselines, Phase 18):
 *            /draft (one board), /me, / (no team), /team/dashboard,
 *            /tree-drafter, /champion-pool, /match/:id, /post-game
 *
 * M0 removed the second theme and the Game Day Brief, so their
 * baselines are gone. M2 re-captured every remaining baseline on the Survey
 * design system (`--update-snapshots`); theme names left the file names.
 * M3 re-captured the rebuilt strategy pages (draft, tree drafter, champion
 * pool, game plan, post-game, opponents, action items).
 */
import { test, expect } from "./fixtures";

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

test.describe("scoped routes - /draft (1 baseline)", () => {
  // M3: one tournament board replaced the carousel, table and ledger views.
  test("/draft board baseline", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(["authed-draft", "board.png"], VR_OPTS);
  });
});

test.describe("personal layer (2 baselines)", () => {
  test("/me baseline", async ({ authedPage }) => {
    await authedPage.goto("/me");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-me.png", VR_OPTS);
  });

  test("/ home (no team) baseline", async ({ authedPage }) => {
    await authedPage.goto("/");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-home-no-team.png", VR_OPTS);
  });
});

test.describe("scoped routes - /team/dashboard (1 baseline)", () => {
  test("/team/dashboard dashboard baseline", async ({ authedPage }) => {
    await authedPage.goto("/team/dashboard");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-team-dashboard", "dashboard.png"],
      VR_OPTS
    );
  });

});

test.describe("scoped routes - single-mode (4 baselines)", () => {
  test("/tree-drafter baseline", async ({ authedPage }) => {
    await authedPage.goto("/tree-drafter");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-tree-drafter", "page.png"],
      VR_OPTS
    );
  });

  test("/champion-pool baseline", async ({ authedPage }) => {
    await authedPage.goto("/champion-pool");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-champion-pool", "page.png"],
      VR_OPTS
    );
  });

  test("/match/:id baseline", async ({ authedPage }) => {
    // Synthetic match ID — page renders empty/error state, which is
    // itself a baseline-worthy surface.
    await authedPage.goto("/match/test-match-id");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-match-detail", "page.png"],
      VR_OPTS
    );
  });

  test("/post-game baseline", async ({ authedPage }) => {
    await authedPage.goto("/post-game");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot(
      ["authed-post-game", "page.png"],
      VR_OPTS
    );
  });

});
