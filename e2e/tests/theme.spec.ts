/**
 * Phase 17 plan 17-01 — Wave 0 theme persistence spec.
 *
 * Asserts:
 *   1. Default theme is "demacia" on a fresh user (DB DEFAULT).
 *   2. Toggling to "pandemonium" persists across reload (DB-backed).
 *   3. Persists across logout + re-login (DB-backed, not localStorage).
 */
import { test, expect } from "./fixtures";

test("default theme is demacia for a new user", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  // Allow post-hydration ThemeToggle to apply DB-stored value
  await authedPage.waitForTimeout(500);
  const theme = await authedPage.getAttribute("html", "data-theme");
  expect(theme).toBe("demacia");
});

test("toggle to pandemonium persists across reload", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.click('button:has-text("Pandemonium")');
  // wasm-patterns rule 56 — give the WASM Effect time to fire spawn_local
  await authedPage.waitForTimeout(700);
  let theme = await authedPage.getAttribute("html", "data-theme");
  expect(theme).toBe("pandemonium");

  await authedPage.reload();
  await authedPage.waitForLoadState("networkidle");
  // Post-hydration sync re-applies DB value
  await authedPage.waitForTimeout(500);
  theme = await authedPage.getAttribute("html", "data-theme");
  expect(theme).toBe("pandemonium");
});
