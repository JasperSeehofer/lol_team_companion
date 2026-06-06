---
phase: 28-library-ia-healing
plan: "02"
subsystem: e2e-tests
tags: [e2e, library, ia-healing, smoke, canary, tdd]
dependency_graph:
  requires: []
  provides: [e2e-contract-28-02]
  affects: [e2e/tests/pages.spec.ts, e2e/tests/hydration-no-panic.spec.ts]
tech_stack:
  added: []
  patterns: [test-first-contract, redirect-covers-removed-entries, per-region-canary-loop]
key_files:
  modified:
    - e2e/tests/pages.spec.ts
    - e2e/tests/hydration-no-panic.spec.ts
decisions:
  - "Remove /stats, /analytics, /post-game, /personal-learnings bare from AUTHED_PAGES — covered by /library after SC2 redirects ship (no /opponents entry existed)"
  - "Keep /personal-learnings/new in AUTHED_PAGES — that route is NOT redirected (SC3 / D-11)"
  - "Keep /post-game and /match/test-match-id in SCOPED_ROUTES — continue to exercise redirect-then-load canary path after Plan 05/06"
metrics:
  duration_minutes: 5
  completed: "2026-06-06T00:48:09Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 28 Plan 02: Library e2e Test Contract Summary

Wave 0 e2e contract for the Library IA healing: `/library` smoke entry authored in `AUTHED_PAGES`, four redirected flat-route entries removed (SC2), `/personal-learnings/new` preserved (SC3), and `/library` appended to `SCOPED_ROUTES` covering both regions (SC5). Tests are intentionally RED until Plans 05/06 ship the route and redirects; Plan 07 turns them GREEN as the phase gate.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Update pages.spec.ts AUTHED_PAGES | 4ee6fec | e2e/tests/pages.spec.ts |
| 2 | Add /library to SCOPED_ROUTES | ab466a6 | e2e/tests/hydration-no-panic.spec.ts |

## What Was Built

**`e2e/tests/pages.spec.ts` — AUTHED_PAGES changes:**
- Added `{ path: "/library", content: /stats|match|analytics|post.game|learnings/i }` near the `/war-room` entry (SC1 smoke)
- Removed `/stats`, `/post-game`, `/analytics`, `/personal-learnings` (bare) — these redirect to `/library` after Plans 05/06, so the `/library` entry covers SC2
- Confirmed no `/opponents` entry existed; nothing to remove there
- Kept `/personal-learnings/new` unchanged (SC3 / D-11 — that route is NOT redirected)
- Playwright `--list` confirms 13 tests parse correctly

**`e2e/tests/hydration-no-panic.spec.ts` — SCOPED_ROUTES changes:**
- Appended `"/library"` after `"/journey"` in the `SCOPED_ROUTES` array
- The existing per-region loop (`for region of ["demacia", "pandemonium"]`) automatically covers `/library` in both regions (SC5)
- Retained `/post-game` and `/match/test-match-id` — these exercise the redirect-then-load canary path after Plans 05/06 (D-11)
- Playwright `--list` confirms 23 tests parse correctly (18 panic-sweep matrix: 9 routes × 2 regions, up from 16)

## Verification

- Playwright `--list` parses both specs without errors (TypeScript not installed as a package dep — `npx tsc` unavailable in project; Playwright's own TS compilation used instead)
- `/library` present in `AUTHED_PAGES` (line 25) and `SCOPED_ROUTES` (line 88)
- `/personal-learnings/new` present in `AUTHED_PAGES` (line 18)
- `/post-game` present in `SCOPED_ROUTES` (line 78)
- `/match/test-match-id` present in `SCOPED_ROUTES` (line 79)
- Redirected entries (`/stats`, `/analytics`, `/post-game`, `/personal-learnings` bare) absent from `AUTHED_PAGES`

## Deviations from Plan

None — plan executed exactly as written. The `npx tsc --noEmit` verification step used Playwright's `--list` flag instead (equivalent parse check) because TypeScript is not a direct dependency in the e2e `package.json`.

## Known Stubs

None. This plan only edits test array declarations.

## Threat Flags

None. Test-only edits; no application attack surface introduced.

## Self-Check: PASSED

- e2e/tests/pages.spec.ts: FOUND (modified)
- e2e/tests/hydration-no-panic.spec.ts: FOUND (modified)
- Commit 4ee6fec: FOUND (pages.spec.ts)
- Commit ab466a6: FOUND (hydration-no-panic.spec.ts)
