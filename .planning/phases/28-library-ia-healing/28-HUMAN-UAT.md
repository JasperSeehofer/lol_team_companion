---
status: passed
phase: 28-library-ia-healing
source: [28-VERIFICATION.md]
started: 2026-06-06T12:14:10Z
updated: 2026-06-06T13:30:00Z
---

## Current Test

[complete]

## Tests

### 1. Tab-switch atmosphere stability
expected: Navigating between the five Library tabs does NOT remount/flash the parallax background (D-05 keep-mounted panels).
result: passed — atmosphere stays mounted across tabs (the "no movement on tab switch" the user observed is the intended keep-mounted behavior, SC1).

### 2. /personal-learnings/new preserved (not redirected)
expected: `/personal-learnings/new` renders the new-learning form, not a redirect to `/library` (D-10/D-11).
result: passed — covered by pages.spec.ts smoke (SC3); only bare `/personal-learnings` redirects.

### 3. Growth overlays visibly enrich on a seeded team
expected: A team with post-game/notebook data shows more atmospheric detail at `/library` than a fresh team; a fresh team shows transparent overlays.
result: passed — overlays fade in by tier (library.spec.ts SC4 asserts opacity > 0). NOTE: UAT surfaced that the original demacia overlay placeholders were opaque BLUE, washing the gold room to gray-blue as growth rose; fixed in commit ffc06f2 (warm low-alpha overlays + eager atmosphere load). Growth now enriches the gold instead of graying it.

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

(none — both UAT-surfaced defects resolved)

## UAT-surfaced fixes (resolved during this phase)

- **Library overlay wash (gold→gray):** opaque blue placeholder overlays washed the gold room to gray as growth tiers rose, inverting the growth-visibility intent. Resolved (ffc06f2) — regenerated all 6 Library overlays as warm low-alpha glows (demacia gold, pandemonium ember); Phase 30 swaps real art at the same paths.
- **Atmosphere color pop-in on load:** planes/overlays were `loading=lazy` → popped in seconds after paint. Resolved (ffc06f2) — switched to `loading=eager`; the existing 400ms overlay opacity transition now fades growth in smoothly.

## Out-of-scope fixes made on request (not Phase 28 work)

- **Legacy `user.draft_mode` NONE coercion blocking account linking:** pre-existing schema/data bug unrelated to Library/IA. Fixed (8ae7b3d) with a startup migration backfilling all five legacy NONE mode/theme fields.

## Deferred to Phase 30 (art production)

- Parallax layer/movement is subtle in War Room and Library because they ship placeholder gradient art (the parallax engine itself is verified working). Real layered FLUX art lands in Phase 30 at the existing asset paths.
