// Visual regression snapshots:
// - First run creates baselines in regression.spec.ts-snapshots/
// - To update baselines: npx playwright test regression.spec.ts --update-snapshots

/**
 * Regression test scaffold for BUG-01 through BUG-05.
 * Placeholder tests are skipped — implemented in Plan 2.
 *
 * Visual Regression describe block is active and creates the first snapshot baseline.
 *
 * Bug descriptions:
 * - BUG-01: Tree drafter branch switching freezes UI (suppress_autosave + signal lifecycle)
 * - BUG-02: "View Game Plan" navigates to wrong plan (wrong ID used in navigation)
 * - BUG-03: Team leader always visible in roster (owner not inserted as team_member on create)
 * - BUG-04: Hydration mismatch warnings on initial page load
 * - BUG-05: Drag-and-drop moves champion but does not clear source slot
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import {
  navigateTo,
  captureErrors,
  filterRealErrors,
  captureHydrationWarnings,
  takeSnapshot,
} from "./helpers";

// ---------------------------------------------------------------------------
// BUG-01: Tree drafter branch switching freezes UI
// ---------------------------------------------------------------------------
test.describe("BUG-01", () => {
  test.skip(
    true,
    "tree drafter branch does not freeze UI — TODO: implement in Plan 2"
  );
  test("tree drafter branch does not freeze UI", async ({ teamPage }) => {
    // TODO: implement in Plan 2
  });
});

// ---------------------------------------------------------------------------
// BUG-02: "View Game Plan" navigates to correct plan
// ---------------------------------------------------------------------------
test.describe("BUG-02", () => {
  test.skip(
    true,
    "view game plan navigates to correct plan — TODO: implement in Plan 2"
  );
  test("view game plan navigates to correct plan", async ({ teamPage }) => {
    // TODO: implement in Plan 2
  });
});

// ---------------------------------------------------------------------------
// BUG-03: Team leader always visible in roster
// ---------------------------------------------------------------------------
test.describe("BUG-03", () => {
  test.skip(
    true,
    "team leader always visible in roster — TODO: implement in Plan 2"
  );
  test("team leader always visible in roster", async ({ teamPage }) => {
    // TODO: implement in Plan 2
  });
});

// ---------------------------------------------------------------------------
// BUG-04: No hydration mismatch warnings
// ---------------------------------------------------------------------------
test.describe("BUG-04", () => {
  test.skip(
    true,
    "no hydration mismatch warnings — TODO: implement in Plan 2"
  );
  test("no hydration mismatch warnings", async ({ authedPage }) => {
    // TODO: implement in Plan 2
  });
});

// ---------------------------------------------------------------------------
// BUG-05: Drag-and-drop moves champion, clears source
// ---------------------------------------------------------------------------
test.describe("BUG-05", () => {
  test.skip(
    true,
    "drag-and-drop moves champion, clears source — TODO: implement in Plan 2"
  );
  test("drag-and-drop moves champion, clears source", async ({ teamPage }) => {
    // TODO: implement in Plan 2
  });
});

// ---------------------------------------------------------------------------
// Visual Regression — establishes baseline snapshots for key pages
// ---------------------------------------------------------------------------
test.describe("Visual Regression", () => {
  test("home page snapshot baseline", async ({ authedPage }) => {
    const page = authedPage;
    await navigateTo(page, "/");
    await takeSnapshot(page, "home-page");
  });
});
