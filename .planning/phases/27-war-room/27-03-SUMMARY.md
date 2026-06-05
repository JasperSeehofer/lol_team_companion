---
phase: "27"
plan: "03"
subsystem: e2e-tests
tags: [war-room, e2e, playwright, tdd, contract-first]
dependency_graph:
  requires: []
  provides: [war-room-e2e-contract, SC1-smoke, SC2-aria, SC3-growth-tiers, SC4-mousemove, SC5-canary]
  affects: [e2e/tests/war-room.spec.ts, e2e/tests/pages.spec.ts, e2e/tests/fixtures.ts]
tech_stack:
  added: []
  patterns: [playwright-spec-first, contract-before-implementation, local-panic-listeners]
key_files:
  created:
    - e2e/tests/war-room.spec.ts
  modified:
    - e2e/tests/pages.spec.ts
    - e2e/tests/fixtures.ts
decisions:
  - "seedTier1Team creates a team + saves 1 draft (tactics_tier threshold [1,5,20] from growth.rs) — minimum sufficient artifact for SC3"
  - "attachPanicListeners kept local per D-06 (not promoted to fixtures.ts)"
  - "mousemove guard test is advisory when getEventListeners unavailable (Assumption A2)"
metrics:
  duration: "~8 minutes"
  completed: "2026-06-05"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 27 Plan 03: War Room e2e Contract Summary

War Room Playwright spec and fixtures scaffolding: `war-room.spec.ts` covering SC2–SC5, a `/war-room` entry in the `pages.spec.ts` smoke suite (SC1), and a `seedTier1Team` helper in `fixtures.ts`. All tests are **intentionally RED** until Plan 27-07 ships the `/war-room` route; Plan 27-08 turns them GREEN as the phase gate.

## What Was Built

### Task 1: /war-room smoke entry + seedTier1Team helper (e6cda10)

**`e2e/tests/pages.spec.ts`** — added to `AUTHED_PAGES`:
```typescript
{ path: "/war-room", content: /draft|war|room|plan/i },
```
Covers SC1 (smoke load without JS errors).

**`e2e/tests/fixtures.ts`** — added and exported `seedTier1Team(page: Page)`:
- Creates a team (reuses `createTeam`) then navigates to `/draft` and clicks "Save Draft"
- Pushes `tactics_tier` to Sprout (tier 1) via 1 saved draft
- Tier-1 threshold justified by comment citing `growth.rs` `TeamGrowthState::from_counts` thresholds: `tier_for(draft + draft_tree, [1, 5, 20])` — tier 1 requires `draft + draft_tree ≥ 1`
- Idempotent: non-fatal if the "Update Draft" button doesn't appear (already seeded)

### Task 2: war-room.spec.ts covering SC2–SC5 (6789657)

**`e2e/tests/war-room.spec.ts`** — 6 tests in `test.describe("War Room", ...)`:

| Test name | SC | What it asserts |
|-----------|-----|-----------------|
| `renders the War surfaces over the parallax atmosphere` | SC2 (partial) | `.room-atmosphere` visible, ≥ 3 `.room-plane`, `[data-feedback-label="War Room"]` visible, no "Failed to load" |
| `aria tablist` | SC2 | One `[role="tablist"]`, exactly 5 `[role="tab"]`, exactly 1 `[aria-selected="true"]`, ≥ 1 `[role="tabpanel"]` |
| `growth tiers` | SC3 | Creates team + seeds, asserts all 3 `data-*-tier` attributes in 0..=3; at least one ≥ 1 after seed |
| `mousemove guard` | SC4 | `/draft`→`/war-room`→`/journey`→`/war-room` round-trip; `getEventListeners(window).mousemove.length === 1` (advisory if API unavailable) |
| `canary demacia` | SC5 | `setRegion(demacia)` + goto `/war-room` + tab click → zero tachys panics |
| `canary pandemonium` | SC5 | `setRegion(pandemonium)` + goto `/war-room` + tab click → zero tachys panics |

All grep targets verified via `npx playwright test war-room.spec.ts --list -g "<name>"`.

## Intentionally RED Tests

These tests are expected to FAIL at runtime until Plan 27-07 ships the `/war-room` route:
- All 6 tests in `war-room.spec.ts` (route returns 404 / redirect until the route is registered in `app.rs`)
- The `/war-room` entry in `pages.spec.ts` AUTHED_PAGES smoke loop

**This is correct per the contract-first approach.** The tests define the acceptance criteria; the implementation fulfils them.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None in test files. The `seedTier1Team` helper is UI-driven (no direct DB writes), consistent with the existing fixture pattern.

## Threat Surface Scan

No new application attack surface. These are test-only files that exercise existing authenticated flows.

## Self-Check: PASSED

- `e2e/tests/war-room.spec.ts` exists: FOUND
- `e2e/tests/pages.spec.ts` contains `/war-room`: FOUND (line 26)
- `e2e/tests/fixtures.ts` exports `seedTier1Team`: FOUND (line 119)
- Task 1 commit `e6cda10`: FOUND
- Task 2 commit `6789657`: FOUND
- `npx playwright test war-room.spec.ts --list` enumerates 6 tests matching all 5 SC grep targets: VERIFIED
