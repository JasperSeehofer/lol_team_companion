/**
 * Hydration panic regression gate.
 *
 * Originally added in Phase 18.2 to lock in the fix for the tachys
 * `InertElement::hydrate Option::unwrap() on None` panic caused by SSR and
 * hydrate rendering different region-branching markup. M0 retired the
 * Pandemonium theme and M2 removed theme plumbing entirely, so the gate
 * covers the single theme plus a check that a stale theme cookie from an
 * old session is ignored without a panic.
 *
 * Coverage:
 * 1. Panic sweep: every scoped route hydrates without a WASM panic.
 *    Listeners are attached BEFORE navigation so first-hydrate panics
 *    are captured. PANIC_REGEX is intentionally narrow so unrelated
 *    console errors (favicon 404 etc.) do not fail the gate.
 * 2. Stale cookie: a leftover `lol_companion_theme=pandemonium` cookie
 *    changes nothing (no `data-theme`) and hydrates cleanly.
 * 3. Interactivity: mode toggles on /draft and /solo respond after
 *    hydration, and a draft slot can be filled from the champion grid.
 */

import { test, expect } from "./fixtures";
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
 * /auth/register?invite=$INVITE_CODE and lands the user on Home (/).
 */
const SCOPED_ROUTES = [
  "/",
  "/me",
  "/draft",
  "/team/dashboard",
  "/tree-drafter",
  "/champion-pool",
  "/post-game",
  "/match/test-match-id",
  "/bugs",
  "/what-next",
  "/changelog",
  "/transparency",
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
// 2. A stale theme cookie is ignored
// ---------------------------------------------------------------------------

test("stale theme cookie is ignored", async ({ authedPage }) => {
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
  expect(await authedPage.getAttribute("html", "data-theme")).toBeNull();
  expect(errors, `Panic with stale cookie: ${errors.join("\n")}`).toEqual([]);
});

// ---------------------------------------------------------------------------
// 3. Interactivity after hydration
// ---------------------------------------------------------------------------
//
// The draft page switches "Our side" and My games switches the LP period:
// a click must produce an observable `aria-checked` change after hydration.

// The draft editor needs a team (M160 solo mode shows NoTeamState without).
test("segmented control observable on /draft", async ({ teamPage: authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  const red = authedPage.locator('[data-segmented="our-side"] [role="radio"]:text-is("Red")');
  await red.click({ timeout: 10000 });
  await expect(red).toHaveAttribute("aria-checked", "true", { timeout: 2000 });
  expect(errors, `Panic during /draft side switch: ${errors.join("\n")}`).toEqual([]);
});

test("segmented control observable on /me", async ({ authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await authedPage.goto("/me");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);
  const week = authedPage.locator('[data-segmented="lp-window"] [role="radio"]:text-is("7 days")');
  await week.click({ timeout: 10000 });
  await expect(week).toHaveAttribute("aria-checked", "true", { timeout: 2000 });
  expect(errors, `Panic during /me period switch: ${errors.join("\n")}`).toEqual([]);
});

test("draft slot click-fill", async ({ teamPage: authedPage }) => {
  const errors = await attachPanicListeners(authedPage);
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(1500);

  // Click the first ban, then click Ahri in the picker beside the board.
  const slot = authedPage.locator("#draft-board-slot-0");
  await slot.click();
  await authedPage.locator("#draft-picker-search").fill("Ahri");
  await authedPage.locator('#draft-picker-list li[data-champion="Ahri"]').click();

  // Observable change: the slot is filled and names the champion.
  await expect(slot).toHaveAttribute("data-filled", "true", { timeout: 2000 });
  await expect(slot).toHaveAttribute("aria-label", /: Ahri/);
  expect(errors, `Panic during draft slot click-fill: ${errors.join("\n")}`).toEqual([]);
});
