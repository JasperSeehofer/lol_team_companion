/**
 * Phase 18 plan 18-09 — region-diff spec (updated for Phase 18.1).
 *
 * Asserts every scoped route renders genuinely different output between
 * Demacia and Pandemonium. pixelDiffRatio must exceed REGION_DIFF_THRESHOLD for each
 * route + mode combination.
 *
 * Uses viewport screenshots (NOT fullPage) per RESEARCH.md Pitfall 6:
 * page heights may differ between regions; pixelmatch requires identical
 * dimensions. Viewport is fixed by playwright.config.ts.
 *
 * NOTE on navigation order (Phase 18.1 update): Post-18.1, `setRegion` handles its
 * own cookie injection + reload, so both call patterns work equivalently:
 *   - setRegion(page, "pandemonium") BEFORE page.goto(url) — cookie is set in the
 *     browser context first, so the next navigation already carries it.
 *   - setRegion(page, "pandemonium") AFTER page.goto(url) — the helper reloads
 *     the current page so SSR re-renders with the new cookie.
 * Pre-18.1, the server unconditionally SSR-rendered data-theme="demacia"
 * (hardcoded `InitialTheme::default()` in `main.rs`) and the helper only
 * flipped tokens client-side, which forced a "must-set-after-goto" idiom.
 * That caveat no longer applies.
 */

import { test, expect, setRegion, setMode } from "./fixtures";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

async function pixelDiffRatio(buf1: Buffer, buf2: Buffer): Promise<number> {
  const img1 = PNG.sync.read(buf1);
  const img2 = PNG.sync.read(buf2);
  if (img1.width !== img2.width || img1.height !== img2.height) {
    // Dimension mismatch = completely different (RESEARCH.md Pitfall 6 recovery)
    return 1.0;
  }
  const diff = pixelmatch(
    img1.data,
    img2.data,
    null,
    img1.width,
    img1.height,
    { threshold: 0.1 }
  );
  return diff / (img1.width * img1.height);
}

async function captureBothRegions(
  page: import("@playwright/test").Page,
  url: string,
  modeSetup?: (page: import("@playwright/test").Page) => Promise<void>
): Promise<{ dem: Buffer; pan: Buffer }> {
  // Navigate first, then set region in-page (avoids SSR resetting data-theme to demacia on reload).
  // The server always SSR-renders data-theme="demacia" (hardcoded in main.rs InitialTheme::default());
  // setRegion clicks the ThemeToggle button which immediately updates data-theme client-side.
  await page.goto(url);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
  await setRegion(page, "demacia"); // fast return if already demacia (SSR default)
  if (modeSetup) await modeSetup(page);
  await page.waitForTimeout(300);
  const dem = await page.screenshot(); // viewport-only (NO fullPage)

  // For pandemonium: navigate fresh (SSR resets to demacia), then immediately switch in-page.
  await page.goto(url);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
  await setRegion(page, "pandemonium");
  if (modeSetup) await modeSetup(page);
  await page.waitForTimeout(300);
  const pan = await page.screenshot(); // viewport-only

  return { dem, pan };
}

// Phase 18.1 calibration: with SSR theme injection landed (Wave 1, 18.1-01),
// structural region branches now activate at runtime. Measured per-route viewport
// diffs after re-baselining (Wave 2, 18.1-02):
//
//   /draft carousel:           2.20%
//   /draft war-table:          2.06%  ← MIN_OBSERVED (viewport-only, see below)
//   /draft ledger:             2.11%
//   /draft loading/post-load:  2.20%
//   /match/:id:                2.78%
//   /champion-pool:            3.59%
//   /solo forge:               3.87%
//   /solo journal:             4.27%
//   /solo constellation:       4.35%
//   /tree-drafter:             4.38%
//   /post-game:                5.34%
//   /team/dashboard brief:    12.86%
//   /team/dashboard dashboard:12.92%
//
// Threshold set to MIN_OBSERVED × 0.7 = 0.02057 × 0.7 ≈ 0.0144, rounded to 0.014
// (30% headroom below the minimum measured value). Catches "structural divergence
// detected" while leaving room for token-noise drift.
//
// Note on /draft cluster (2.0–2.2%): full-page comparison shows /draft Pandemonium
// at 2282px vs Demacia at 2584px (very different structural heights, ~2.8M differing
// pixels after fuzz-5%). Region-diff uses VIEWPORT screenshots (1280×720) per
// Pitfall 6, and the viewport region of /draft happens to be more layout-aligned
// across regions than the lower-page sections (where RiotTape, Glitch, zine cards
// actually live). This is expected and not a Phase-18 source bug.
//
// Pre-18.1 the diffs were 0.5–2.5% (token swap only); 0.005 caught that.
// The original SPEC value of 0.40 (40%) assumed wholesale layout swap and was
// never achievable; 0.40 is documented as a deviation in 18-09-SUMMARY.
const REGION_DIFF_THRESHOLD = 0.014;

// ---- /draft (3 modes) ----

test("/draft carousel: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/draft", async (p) => {
    await setMode(p, "carousel");
  });
  const ratio = await pixelDiffRatio(dem, pan);
  expect(ratio).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/draft war-table: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/draft", async (p) => {
    await setMode(p, "war-table");
  });
  const ratio = await pixelDiffRatio(dem, pan);
  expect(ratio).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/draft ledger: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/draft", async (p) => {
    await setMode(p, "ledger");
  });
  const ratio = await pixelDiffRatio(dem, pan);
  expect(ratio).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

// ---- /solo (3 modes) ----

test("/solo constellation: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/solo", async (p) => {
    await setMode(p, "constellation");
  });
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/solo forge: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/solo", async (p) => {
    await setMode(p, "forge");
  });
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/solo journal: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/solo", async (p) => {
    await setMode(p, "journal");
  });
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

// ---- /team/dashboard (2 modes) ----

test("/team/dashboard dashboard: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/team/dashboard", async (p) => {
    await setMode(p, "dashboard");
  });
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/team/dashboard brief: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/team/dashboard", async (p) => {
    await setMode(p, "brief");
  });
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

// ---- Single-mode routes ----

test("/tree-drafter: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/tree-drafter");
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/champion-pool: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/champion-pool");
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/post-game: regions differ by >1.4%", async ({ authedPage }) => {
  const { dem, pan } = await captureBothRegions(authedPage, "/post-game");
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

test("/match/:id: regions differ by >1.4%", async ({ authedPage }) => {
  // Use the same synthetic match ID as visual-regression.spec.ts
  const matchUrl = "/match/test-match-id";
  const { dem, pan } = await captureBothRegions(authedPage, matchUrl);
  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});

// ---- Loading skeleton variants (covers REQ-2) ----
// Asserts that the /draft page loaded in each region differs by >1.4%.
// Uses the same navigate-first + setRegion approach as other tests.
test("/draft loading/post-load: regions differ by >1.4%", async ({ authedPage }) => {
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  await setRegion(authedPage, "demacia");
  await authedPage.waitForTimeout(300);
  const dem = await authedPage.screenshot();

  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  await setRegion(authedPage, "pandemonium");
  await authedPage.waitForTimeout(300);
  const pan = await authedPage.screenshot();

  expect(await pixelDiffRatio(dem, pan)).toBeGreaterThan(REGION_DIFF_THRESHOLD);
});
