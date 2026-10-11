/**
 * Hydration panic regression gate.
 *
 * Originally added in Phase 18.2 to lock in the fix for the tachys
 * `InertElement::hydrate Option::unwrap() on None` panic caused by SSR and
 * hydrate rendering different region-branching markup. M0 retired the
 * Pandemonium theme, so the gate now covers the single shipped theme
 * ("demacia") plus a check that a stale "pandemonium" cookie is clamped
 * back to it without a panic.
 *
 * Coverage:
 * 1. Panic sweep: every scoped route hydrates without a WASM panic.
 *    Listeners are attached BEFORE navigation so first-hydrate panics
 *    are captured. PANIC_REGEX is intentionally narrow so unrelated
 *    console errors (favicon 404 etc.) do not fail the gate.
 * 2. Stale-cookie clamp: a leftover `lol_companion_theme=pandemonium`
 *    cookie renders `<html data-theme="demacia">` and hydrates cleanly.
 * 3. Interactivity: mode toggles on /draft and /solo respond after
 *    hydration, and a draft slot can be filled from the champion grid.
 */

import { test, expect, setRegion, setMode } from "./fixtures";
import type { Page } from "@playwright/test";

/**
 * D-08 narrow panic filter. Matches the three observed WASM panic
 * shapes from the Phase 18.1 / 18.2 evidence trail:
 *   - `panicked at ...` (any Rust panic surfacing through
 *     `console_error_panic_hook::set_once()`)
 *   - `Option::unwrap() on None` (the specific tachys cursor-walk
 *     failure documented in 18.1-03-EVIDENCE.md)
 *   - `tachys` (any other tachys-namespaced runtime error)
 *
 * Intentionally case-insensitive. Does NOT match generic
 * console.error from missing favicon, Tailwind import 404, or
 * Suspense-outside-Resource warnings.
 */
const PANIC_REGEX = /panicked at|Option::unwrap.*None|tachys/i;

/**
 * The 7 routes that still use region-branching primitives. All require auth — the `authedPage` fixture registers via
 * /auth/register?invite=$INVITE_CODE and lands the user on /solo.
 */
const SCOPED_ROUTES = [
  "/solo",
  "/draft",
  "/team/dashboard",
  "/tree-drafter",
  "/champion-pool",
  "/post-game",
  "/match/test-match-id",
] as const;

/**
 * Attach `pageerror` + filtered `console.error` listeners BEFORE
 * navigation so first-hydrate panics are captured. Returns the
 * mutable errors array — callers `expect(errors).toEqual([])` after
 * the hydrate-settle pause.
 *
 * Lives in this spec file (not fixtures.ts) so the listener overhead
 * doesn't leak into tests that don't care about panics. The console.error
 * filter uses PANIC_REGEX directly rather than helpers.ts::captureErrors
 * (which captures every console.error and is false-positive prone here).
 */
async function attachPanicListeners(page: Page): Promise<string[]> {
  const errors: string[] = [];

  // pageerror = uncaught JS exception. WASM panics propagate here via
  // console_error_panic_hook (lib.rs:17) → throw.
  page.on("pageerror", (err) => {
    errors.push(`pageerror: ${err.message}`);
  });

  // console.error = WASM/JS console.error. Filter narrowly so unrelated
  // errors (favicon 404, Tailwind import) don't false-positive.
  page.on("console", (msg) => {
    if (msg.type() === "error" && PANIC_REGEX.test(msg.text())) {
      errors.push(`console.error: ${msg.text()}`);
    }
  });

  return errors;
}

// ---------------------------------------------------------------------------
// 1. Panic sweep over the scoped routes
// ---------------------------------------------------------------------------

for (const url of SCOPED_ROUTES) {
  test(`${url} hydrates without tachys panic`, async ({ authedPage }) => {
    const errors = await attachPanicListeners(authedPage);
    await setRegion(authedPage, "demacia");
    await authedPage.goto(url);
    await authedPage.waitForLoadState("networkidle");
    // 1500ms hydrate-settle: WASM may crash mid-hydrate AFTER networkidle
    // reports done, because the runtime walks the SSR DOM in a microtask
    // scheduled by the hydration script.
    await authedPage.waitForTimeout(1500);
    expect(
      errors,
      `Expected no panic on ${url}; got: ${errors.join("\n")}`
    ).toEqual([]);
  });
}

// ---------------------------------------------------------------------------
// 2. Stale "pandemonium" cookie is clamped to the single shipped theme
// ---------------------------------------------------------------------------

test("stale pandemonium cookie is clamped to demacia", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await authedPage.context().addCookies([
    {
      name: "lol_companion_theme",
      value: "pandemonium",
      domain: "127.0.0.1",
      path: "/",
      sameSite: "Lax",
    },
  ]);
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  expect(await authedPage.getAttribute("html", "data-theme")).toBe("demacia");
  expect(errors, `Panic with stale cookie: ${errors.join("\n")}`).toEqual([]);
});

// ---------------------------------------------------------------------------
// 3. Interactivity after hydration
// ---------------------------------------------------------------------------
//
// Default modes (src/pages/draft.rs::resolve_mode): /draft = carousel,
// /solo = constellation. We click a non-default mode so the toggle produces
// an observable `aria-pressed` change.

test("mode-toggle observable on /draft", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "demacia");
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await setMode(authedPage, "war-table");
  await expect(
    authedPage.locator('button[aria-pressed="true"]:has-text("War Table")')
  ).toBeVisible({ timeout: 2000 });
  expect(errors, `Panic during /draft mode-toggle: ${errors.join("\n")}`).toEqual([]);
});

test("mode-toggle observable on /solo", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "demacia");
  await authedPage.goto("/solo");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await setMode(authedPage, "journal");
  await expect(
    authedPage.locator('button[aria-pressed="true"]:has-text("Journal")')
  ).toBeVisible({ timeout: 2000 });
  expect(errors, `Panic during /solo mode-toggle: ${errors.join("\n")}`).toEqual([]);
});

test("draft slot click-fill", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "demacia");
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);

  const searchInput = authedPage.locator('input[placeholder="Search champion..."]');
  await expect(searchInput).toBeVisible({ timeout: 5000 });
  await searchInput.fill("Ahri");
  await authedPage.waitForTimeout(300);
  const ahriBtn = authedPage.locator('div[style*="display:grid"] button img[alt="Ahri"]');
  await ahriBtn.first().click();

  // Observable change: "Ahri" appears on the page (grid + filled slot).
  await expect(authedPage.locator("text=Ahri").first()).toBeVisible({ timeout: 2000 });
  expect(errors, `Panic during draft slot click-fill: ${errors.join("\n")}`).toEqual([]);
});
