/**
 * Phase 17 plan 17-07 — Final visual-regression baseline capture.
 *
 * Captures pixel baselines for every restyled route in Phase 17.
 * First run captures baselines (`--update-snapshots`). Subsequent runs diff
 * against baseline with `maxDiffPixelRatio: 0.02` (2% leeway for font
 * subpixel rendering, animation frames, anti-aliasing variance).
 *
 * Route taxonomy (from 17-UI-SPEC Route Inventory + src/app.rs):
 *   Public:  /closed-beta, /auth/login, /auth/register (with invite query),
 *            /legal/impressum, /legal/datenschutz
 *   Authed:  /profile, /team/dashboard, /team/roster, /team-builder,
 *            /draft, /tree-drafter, /champion-pool, /game-plan, /post-game,
 *            /opponents, /action-items, /stats, /match/:id,
 *            /personal-learnings, /analytics, /solo, /admin/invites
 *
 * Note on `/`: Leptos hard-redirects to /closed-beta or /team/dashboard
 * depending on auth state — covered indirectly by /closed-beta and
 * /team/dashboard baselines.
 *
 * Note on Pandemonium variant: theme switching is exercised separately by
 * theme.spec.ts. The visual-regression suite captures the Demacia (default)
 * baseline only — capturing both themes per route would double the snapshot
 * count without commensurate signal.
 *
 * Note on `/match/:id`: uses a synthetic ID. The page may render a "match
 * not found" state, which is itself a baseline-worthy surface (verifies
 * empty/error state styling).
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
// Auth-required routes — visited via authedPage fixture
// ---------------------------------------------------------------------------

test.describe("auth-required routes", () => {
  test("/profile visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/profile");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-profile.png", VR_OPTS);
  });

  test("/team/dashboard visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/team/dashboard");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-team-dashboard.png", VR_OPTS);
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

  test("/draft visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-draft.png", VR_OPTS);
  });

  test("/tree-drafter visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/tree-drafter");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-tree-drafter.png", VR_OPTS);
  });

  test("/champion-pool visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/champion-pool");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-champion-pool.png", VR_OPTS);
  });

  test("/game-plan visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/game-plan");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-game-plan.png", VR_OPTS);
  });

  test("/post-game visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/post-game");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-post-game.png", VR_OPTS);
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

  test("/match/:id visual baseline", async ({ authedPage }) => {
    // Synthetic match ID — page should render its empty/error state, which
    // is itself a baseline-worthy surface.
    await authedPage.goto("/match/test-match-id");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-match-detail.png", VR_OPTS);
  });

  test("/personal-learnings visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/personal-learnings");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-personal-learnings.png", VR_OPTS);
  });

  test("/analytics visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/analytics");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-analytics.png", VR_OPTS);
  });

  test("/solo visual baseline", async ({ authedPage }) => {
    await authedPage.goto("/solo");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await expect(authedPage).toHaveScreenshot("authed-solo.png", VR_OPTS);
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
