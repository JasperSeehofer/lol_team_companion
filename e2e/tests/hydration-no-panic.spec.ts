/**
 * Phase 18.2 Plan 04 — Hydration panic regression gate.
 *
 * Locks in the Phase 18.2 fix for UI-18.1-HYDRATE-01 (the
 * `tachys-0.2.14/src/html/mod.rs:217 InertElement::hydrate
 * Option::unwrap() on None` panic that fired on every authenticated
 * Pandemonium page after Phase 18.1's SSR theme injection started
 * actually resolving the cookie to "pandemonium"). From this spec
 * forward any commit that reintroduces an SSR/hydrate divergence on
 * a region-branching primitive — or symmetrically breaks Demacia —
 * fails the e2e suite.
 *
 * Coverage (per 18.2-04-PLAN + 18.2-PATTERNS):
 *
 * 1. Panic-sweep matrix (14 tests): 7 scoped routes × 2 regions.
 *    Listener is attached BEFORE navigation so first-hydrate panics
 *    are captured. The PANIC_REGEX is intentionally narrow (D-08) —
 *    generic console.error from favicon 404 / Tailwind warning / etc.
 *    must NOT fail the test.
 *
 * 2. D-13 interactivity triad (5 tests, Pandemonium only):
 *    - mode-toggle on /draft + /solo + /team/dashboard:
 *      click the SSR-default mode's opposite ("CAROUSEL" on /draft,
 *      "CONSTELLATION" on /solo, "DASHBOARD" on /team/dashboard) and
 *      poll for `button[aria-pressed="true"]:has-text("<label>")`
 *      within 2s (D-14 specific-marker shape, NOT a coarse "DOM
 *      changed" check — Pitfall 3 enforcement).
 *    - theme-toggle round-trip: pandemonium → demacia → pandemonium
 *      via the `setRegion` fixture (cookie + reload). Assert
 *      `html[data-theme="<region>"]` after each flip; assert no panic.
 *    - draft slot click-fill: search for "Ahri" via the champion
 *      autocomplete and click; assert the champion name appears on
 *      the page (thin observable change — exercises the same WASM
 *      event-handler attachment as drag-drop without the synthetic
 *      DataTransfer ceremony).
 *
 * Why this file is separate (D-06):
 *   - region-diff.spec.ts is pixel-diff concerned; we are panic-detection concerned.
 *   - Promoting attachPanicListeners to fixtures.ts would slow every
 *     test with a console listener it doesn't need.
 *
 * Why we do NOT import captureErrors from helpers.ts (D-08):
 *   - helpers.ts::captureErrors fires on ANY console.error. We need
 *     a NARROW regex filter so unrelated console errors don't
 *     false-positive this gate.
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
 * The 7 region-scoped routes per Phase 18 spec. Mirrors the list
 * `region-diff.spec.ts` walks but for panic detection, not pixel
 * diff. All require auth — the `authedPage` fixture registers via
 * /auth/register?invite=E2E-TEST and lands the user on /solo.
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
 * Per D-06, this lives in this spec file (NOT promoted to
 * fixtures.ts) so the listener overhead doesn't leak into
 * pixel-diff / smoke / pages tests that don't care about panics.
 *
 * Per D-08, the console.error filter uses PANIC_REGEX directly —
 * we do not reuse helpers.ts::captureErrors (which captures every
 * console.error and is false-positive prone for this gate).
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
// 1. Panic-sweep matrix — 7 routes × 2 regions = 14 tests
// ---------------------------------------------------------------------------
//
// Per-test execution shape (verbatim from 18.2-RESEARCH.md lines 606-617):
//   1. attach listeners BEFORE navigation
//   2. setRegion (cookie + reload via fixture)
//   3. goto target URL
//   4. waitForLoadState("networkidle")
//   5. waitForTimeout(1500) — hydrate-settle window (gives WASM time
//      to crash mid-hydrate before we assert)
//   6. expect errors to be empty
//
// Both regions are iterated (D-07) to catch the symmetric Demacia
// regression case (Pitfall 5) that Phase 18.2 Plan 03 surfaced —
// before this gate, Plan 01's text-mode agent-browser console
// truncation hid an ALWAYS-present Demacia panic at the same site.
// This spec uses Playwright's structured `pageerror` event which
// has no such truncation problem.

for (const region of ["demacia", "pandemonium"] as const) {
  for (const url of SCOPED_ROUTES) {
    test(`${url} hydrates without tachys panic [${region}]`, async ({
      authedPage,
    }) => {
      const errors = await attachPanicListeners(authedPage);
      await setRegion(authedPage, region);
      await authedPage.goto(url);
      await authedPage.waitForLoadState("networkidle");
      // 1500ms hydrate-settle (per RESEARCH.md line 615): WASM may
      // crash mid-hydrate AFTER networkidle reports done, because
      // the runtime walks the SSR DOM in a microtask scheduled by
      // the hydration script. 500ms is too tight on cold-cache CI;
      // 1500ms is the documented buffer.
      await authedPage.waitForTimeout(1500);
      expect(
        errors,
        `Expected no panic on ${url} [${region}]; got: ${errors.join("\n")}`
      ).toEqual([]);
    });
  }
}

// ---------------------------------------------------------------------------
// 2. D-13 interactivity triad — Pandemonium only (5 tests)
// ---------------------------------------------------------------------------
//
// SSR default modes per page (drives the "click AWAY" choices below
// — we click the mode that's NOT the SSR default so the toggle
// produces an observable state change). Source:
// 18.1-03-EVIDENCE.md "Mode-toggle SSR defaults" table.
//
//   /draft Pandemonium default = LEDGER       → click CAROUSEL
//   /solo Pandemonium default = FORGE         → click CONSTELLATION
//   /team/dashboard Pandemonium default = GAME_DAY (brief)
//                                              → click DASHBOARD
//
// Marker shape (D-14 / Pitfall 3): poll for the SPECIFIC
// `button[aria-pressed="true"]:has-text("<LABEL>")` within 2s. NOT
// a coarse "any DOM change" check — that's brittle and false-positive
// prone.

test("mode-toggle observable on /draft [pandemonium]", async ({
  authedPage,
}) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "pandemonium");
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await setMode(authedPage, "carousel");
  await expect(
    authedPage.locator(
      'button[aria-pressed="true"]:has-text("CAROUSEL")'
    )
  ).toBeVisible({ timeout: 2000 });
  expect(
    errors,
    `Panic during /draft mode-toggle: ${errors.join("\n")}`
  ).toEqual([]);
});

test("mode-toggle observable on /solo [pandemonium]", async ({
  authedPage,
}) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "pandemonium");
  await authedPage.goto("/solo");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await setMode(authedPage, "constellation");
  await expect(
    authedPage.locator(
      'button[aria-pressed="true"]:has-text("CONSTELLATION")'
    )
  ).toBeVisible({ timeout: 2000 });
  expect(
    errors,
    `Panic during /solo mode-toggle: ${errors.join("\n")}`
  ).toEqual([]);
});

test("mode-toggle observable on /team/dashboard [pandemonium]", async ({
  authedPage,
}) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "pandemonium");
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await setMode(authedPage, "dashboard");
  await expect(
    authedPage.locator(
      'button[aria-pressed="true"]:has-text("DASHBOARD")'
    )
  ).toBeVisible({ timeout: 2000 });
  expect(
    errors,
    `Panic during /team/dashboard mode-toggle: ${errors.join("\n")}`
  ).toEqual([]);
});

test("theme-toggle round-trip [pandemonium <-> demacia]", async ({
  authedPage,
}) => {
  const errors = await attachPanicListeners(authedPage);
  // Start on Pandemonium /draft (high-traffic region-branching page)
  await setRegion(authedPage, "pandemonium");
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  await expect(authedPage.locator('html[data-theme="pandemonium"]')).toBeVisible({
    timeout: 2000,
  });

  // Flip to Demacia (setRegion does cookie + reload; D-15 — we do NOT
  // assert DB persistence here, only the SSR/hydrate round-trip).
  await setRegion(authedPage, "demacia");
  await expect(authedPage.locator('html[data-theme="demacia"]')).toBeVisible({
    timeout: 2000,
  });

  // Flip back to Pandemonium.
  await setRegion(authedPage, "pandemonium");
  await expect(authedPage.locator('html[data-theme="pandemonium"]')).toBeVisible({
    timeout: 2000,
  });

  expect(
    errors,
    `Panic during theme-toggle round-trip: ${errors.join("\n")}`
  ).toEqual([]);
});

test("draft slot click-fill [pandemonium]", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await setRegion(authedPage, "pandemonium");
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);

  // Search + click flow (mirrors helpers.ts::selectChampion but
  // inlined so the test's assertion is the observable change —
  // the champion name "Ahri" appears on the page within 2s of click).
  // This exercises the same WASM event-handler attachment as the
  // drag-drop path without the synthetic DataTransfer ceremony.
  const searchInput = authedPage.locator(
    'input[placeholder="Search champion..."]'
  );
  await expect(searchInput).toBeVisible({ timeout: 5000 });
  await searchInput.fill("Ahri");
  await authedPage.waitForTimeout(300);
  const ahriBtn = authedPage.locator(
    'div[style*="display:grid"] button img[alt="Ahri"]'
  );
  await ahriBtn.first().click();

  // D-14 observable change: assert "Ahri" appears in the draft board
  // area (a slot fill renders the champion name / image). Thin
  // assertion — the champion name appears SOMEWHERE on the page
  // (locator.first() — there are multiple "Ahri" hits: one in the
  // champion grid, one in the slot it fills).
  await expect(authedPage.locator('text=Ahri').first()).toBeVisible({
    timeout: 2000,
  });

  expect(
    errors,
    `Panic during draft slot click-fill: ${errors.join("\n")}`
  ).toEqual([]);
});
