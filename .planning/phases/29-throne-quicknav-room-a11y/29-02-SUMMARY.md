---
phase: 29-throne-quicknav-room-a11y
plan: "02"
subsystem: css-assets-e2e
tags: [wave-0-blockers, growth-visibility, placeholder-assets, e2e-specs, a11y]
dependency_graph:
  requires: []
  provides:
    - input.css roster/progression/crown tier→opacity selectors (SC1 visibility blocker closed)
    - 12 placeholder WebP assets at public/img/rooms/throne/{demacia,pandemonium}/
    - /throne in hydration-no-panic SCOPED_ROUTES (SC4 A11Y-02 canary)
    - /throne in pages.spec.ts AUTHED_PAGES smoke
    - throne-growth.spec.ts (SC1/SC1b — computed opacity assertions)
    - quicknav-focus.spec.ts (SC2 restore / SC3 trap — skip-guarded, Plan 07 un-skips)
    - room-contrast.spec.ts (SC5 z-8 scrim — war-room/library live, throne skip-guarded)
  affects:
    - plans 29-05/29-06: green-able tests waiting when room/QuickNav ships
tech_stack:
  added: []
  patterns:
    - ImageMagick gradient WebP generation (mirror of 28-05 precedent, 1600x900 planes + 1200x800 overlays)
    - test.describe.skip / test.skip / test.fixme guarding — suite stays green; Plan 07 un-skips
    - getComputedStyle().opacity assertion (Phase 27 seed-and-assert-opacity lesson)
key_files:
  created:
    - e2e/tests/throne-growth.spec.ts
    - e2e/tests/quicknav-focus.spec.ts
    - e2e/tests/room-contrast.spec.ts
    - public/img/rooms/throne/demacia/plane-sky.webp
    - public/img/rooms/throne/demacia/plane-mid.webp
    - public/img/rooms/throne/demacia/plane-near.webp
    - public/img/rooms/throne/demacia/overlay-roster.webp
    - public/img/rooms/throne/demacia/overlay-progression.webp
    - public/img/rooms/throne/demacia/overlay-crown.webp
    - public/img/rooms/throne/pandemonium/plane-sky.webp
    - public/img/rooms/throne/pandemonium/plane-mid.webp
    - public/img/rooms/throne/pandemonium/plane-near.webp
    - public/img/rooms/throne/pandemonium/overlay-roster.webp
    - public/img/rooms/throne/pandemonium/overlay-progression.webp
    - public/img/rooms/throne/pandemonium/overlay-crown.webp
  modified:
    - input.css (9 Throne tier→opacity selectors added after Library analysis block)
    - e2e/tests/hydration-no-panic.spec.ts (/throne added to SCOPED_ROUTES)
    - e2e/tests/pages.spec.ts (/throne entry added; /profile + /team/dashboard content regex updated to be redirect-aware)
decisions:
  - "Room contrast spec (room-contrast.spec.ts) tests war-room and library immediately (live routes) and skip-guards /throne until Plan 29-05 — avoids suite breakage while preserving the correct final assertion shape"
  - "pages.spec.ts /profile and /team/dashboard entries updated to accept redirect-destination content (throne) rather than removed — keeps the redirect chain exercised without depending on standalone content that will 302 after Plan 29-05"
  - "throne-growth.spec.ts SC1b marked test.fixme (not test.skip) because the assertion shape is complete but the seeding fixture (teamless+ranked) is not yet available — fixme communicates 'todo for Plan 07' more precisely than a describe.skip"
metrics:
  duration: "~25 minutes"
  completed: "2026-06-08"
  tasks_completed: 2
  tasks_total: 2
  files_created: 18
  files_modified: 3
---

# Phase 29 Plan 02: Wave 0 Blockers + E2E Harness Summary

Closed the two Wave 0 BLOCKERS that would silently fail SC1 and eager-load 404 errors before the Throne room render plans (29-05/29-06), and wired the full e2e harness for the phase.

## What Was Built

**BLOCKER 1 closed — SC1 growth visibility gap:** Added nine `[data-{roster,progression,crown}-tier="N"] .room-overlay--{axis}` selectors to `input.css` immediately after the Library `analysis` block, using the identical opacity ramp (0.34 / 0.67 / 1). Without these, `.room-overlay` defaults to `opacity:0` at every tier — the Throne overlays would render fully transparent even with correct `data-tier` attributes, exactly the Phase 27 "room growth visibility gap" (MEMORY.md).

**BLOCKER 2 closed — 404 on eager WebP loads:** Generated 12 gradient placeholder WebP assets via ImageMagick `convert gradient:` at `public/img/rooms/throne/{demacia,pandemonium}/` (planes 1600×900, overlays 1200×800). Demacia: gold/azure regal tones. Pandemonium: crimson/dark tones. All 12 files are valid WebP (RIFF header verified) and well under the Phase 30 CI 100 KB budget (largest is ~6 KB).

**E2e harness wired:**
- `/throne` added to `SCOPED_ROUTES` in `hydration-no-panic.spec.ts` — the existing `for (const region of ["demacia", "pandemonium"])` loop auto-generates 2 new A11Y-02 canary tests (SC4) with no other edits.
- `/throne` added to `AUTHED_PAGES` in `pages.spec.ts`; `/profile` and `/team/dashboard` content regexes broadened to accept throne-destination content (redirect-aware, not removed).
- `throne-growth.spec.ts`: SC1 asserts `getComputedStyle(el).opacity` on `.room-overlay--roster` (not just `data-tier` attribute — the Phase 27 seed-and-assert-opacity lesson); SC1b blended-axes scaffold marked `test.fixme`.
- `quicknav-focus.spec.ts`: SC2 (focus-restore via Escape) and SC3 (focus-trap Tab cycling inside `[role="dialog"]`) — `test.describe.skip` until Plan 29-06 ships the implementation.
- `room-contrast.spec.ts`: SC5 asserts `div[style*="radial-gradient"]` + `aria-hidden="true"` on `/war-room` and `/library` (live — active tests); `/throne` entry is `test.skip` until Plan 29-05.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All 12 WebP assets are valid placeholder gradients (not empty files). CSS selectors are complete. Spec files encode correct final assertions; skip guards are explicit and documented with the un-skip plan number.

## Threat Flags

None. Static CSS, static WebP assets, and TypeScript test files only. No new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

**Files exist:**
- `input.css` — 9 selectors: `grep -c 'room-overlay--roster'` = 3 (and progression/crown = 3 each) ✓
- All 12 WebP files exist at `public/img/rooms/throne/{demacia,pandemonium}/` ✓
- `e2e/tests/throne-growth.spec.ts` ✓
- `e2e/tests/quicknav-focus.spec.ts` ✓
- `e2e/tests/room-contrast.spec.ts` ✓
- `/throne` in `hydration-no-panic.spec.ts` SCOPED_ROUTES ✓
- `/throne` in `pages.spec.ts` AUTHED_PAGES ✓

**Commits exist:**
- `8c16191` — feat(29-02): add 9 Throne tier→opacity selectors + 12 placeholder WebP assets ✓
- `aab7aa3` — feat(29-02): wire /throne into canary+smoke suites; author SC1/SC2/SC3/SC5 specs ✓

**Playwright --list:** 184 total tests found; `/throne` appears in hydration-no-panic (demacia + pandemonium), pages.spec, throne-growth, quicknav-focus, room-contrast — all loaded without transpile errors ✓
