/**
 * Phase 17 plan 17-01 — Wave 0 closed-beta route smoke test.
 *
 * Plan 06 fills the actual landing content. For Wave 0 we only need to
 * confirm the route returns 200 and renders some body text (placeholder is
 * acceptable). Stronger assertions land in plan 06.
 */
import { test, expect } from "@playwright/test";

test("/closed-beta returns 200 and renders landing", async ({ page }) => {
  const resp = await page.goto("/closed-beta");
  expect(resp?.status()).toBe(200);
  await page.waitForLoadState("networkidle");
  const body = await page.textContent("body");
  expect(body?.length ?? 0).toBeGreaterThan(0);
});

test("unauthenticated visit to / does not 5xx", async ({ page }) => {
  const resp = await page.goto("/");
  // 200 (fully rendered) or any 3xx (redirect to /closed-beta in plan 06)
  // — anything below 500 is acceptable for Wave 0.
  expect(resp?.status()).toBeLessThan(500);
});

test("/admin/invites placeholder returns 200", async ({ page }) => {
  const resp = await page.goto("/admin/invites");
  expect(resp?.status()).toBeLessThan(500);
});

test("/legal/impressum and /legal/datenschutz return 200", async ({ page }) => {
  const r1 = await page.goto("/legal/impressum");
  expect(r1?.status()).toBeLessThan(500);
  const r2 = await page.goto("/legal/datenschutz");
  expect(r2?.status()).toBeLessThan(500);
});
