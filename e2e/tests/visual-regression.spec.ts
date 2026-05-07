/**
 * Phase 17 plan 17-01 — Wave 0 visual baseline capture.
 *
 * Plans 02-04 (hub plans) and plan 06 (closed-beta) add per-page baselines.
 * For Wave 0 we capture only the most-stable surfaces:
 *   - /auth/login (public, no team-state variability)
 *   - /team/dashboard (authed default landing)
 *
 * First run captures the baseline (`--update-snapshots`). Subsequent runs
 * diff against baseline with maxDiffPixelRatio = 0.02 (2% leeway for
 * font subpixel rendering, animation frames, etc.).
 */
import { test, expect } from "./fixtures";

test("auth/login visual baseline", async ({ page }) => {
  await page.goto("/auth/login");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500); // let WASM hydrate + fonts settle
  await expect(page).toHaveScreenshot("auth-login.png", {
    maxDiffPixelRatio: 0.02,
    fullPage: false,
  });
});

test("team/dashboard visual baseline", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  await expect(authedPage).toHaveScreenshot("team-dashboard.png", {
    maxDiffPixelRatio: 0.02,
    fullPage: false,
  });
});
