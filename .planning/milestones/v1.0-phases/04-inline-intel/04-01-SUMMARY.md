---
phase: 04-inline-intel
plan: 01
subsystem: ui
tags: [leptos, draft, champion-pool, warnings, memo, tdd]

# Dependency graph
requires:
  - phase: 03-context-and-pipeline
    provides: team_pools resource and champion pool data already loaded in draft page

provides:
  - "Pool warning badges on draft pick slots with class gap analysis"
  - "compute_slot_warnings() pure function with unit tests"
  - "warning_slots optional prop on DraftBoard component"
  - "role override dropdowns for per-slot role assignment"

affects:
  - 04-inline-intel
  - any future work touching draft_board.rs or draft.rs

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Memo-based client-side computation over already-loaded server data (no new server calls)"
    - "Optional Signal prop (Option<Signal<Vec<...>>>) for DraftBoard warnings"
    - "Dual lookup (name + canonical ID) for champion pool entries vs draft slot entries"

key-files:
  created: []
  modified:
    - src/components/draft_board.rs
    - src/pages/draft.rs

key-decisions:
  - "Warning triggers require BOTH 'not in pool' AND 'class gap exists' (coaching-quality insight, per locked CONTEXT.md decision)"
  - "champion_map keyed by display name; pool entries use canonical IDs - built reverse id_to_champion lookup inside compute_slot_warnings"
  - "Role overrides use a HashMap<usize, String> RwSignal keyed by slot index, defaulting to conventional draft order (top/jungle/mid/bot/support)"

patterns-established:
  - "compute_slot_warnings: pure fn taking slots/pools/champions/side/overrides, returns Vec<Option<(player, detail)>> length 20"
  - "warning badge at top-right of pick slot image container using absolute positioning inside the relative flex-shrink-0 div"

requirements-completed:
  - PIPE-03

# Metrics
duration: 33min
completed: 2026-03-17
---

# Phase 04 Plan 01: Pool Warning Badges Summary

**Client-side draft pool warning badges with class gap analysis: amber overlays on out-of-pool picks that expose class coverage gaps, computed via reactive Memo with no server calls**

## Performance

- **Duration:** 33 min
- **Started:** 2026-03-17T00:01:28Z
- **Completed:** 2026-03-17T00:35:07Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added optional `warning_slots: Option<Signal<Vec<Option<(String, String)>>>>` prop to `DraftBoard` rendering amber badge overlays (top-right of pick slot images) with tooltip "Not in {player}'s pool. No {Class} coverage."
- Implemented `compute_slot_warnings()` pure function in `draft.rs` with dual champion lookup (display name and canonical Data Dragon ID), class gap analysis, and role-override support
- Wired a reactive `Memo` in `DraftPage` tracking `draft_slots`, `our_side`, `role_overrides`, `team_pools`, and `champions_resource` - updates instantly on pick changes, no server calls
- Added role override dropdowns (subtle `text-xs` selects) below the draft board for each our-side pick slot
- All 6 plan-specified unit tests pass; full test suite passes (47 tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add warning_slots prop to DraftBoard and render badge overlay** - `e6bed7e` (feat)
2. **Task 2: Compute pool warnings with class gap analysis in draft.rs, wire to DraftBoard, add unit tests** - `96fc02f` (feat)

**Plan metadata:** (see final docs commit)

## Files Created/Modified
- `src/components/draft_board.rs` - Added optional `warning_slots` prop; amber badge at top-right of pick slot image with tooltip
- `src/pages/draft.rs` - Added `compute_slot_warnings()`, `normalize_role()`, `warning_slots_memo` Memo, `role_overrides` signal, role override dropdowns, 6 unit tests

## Decisions Made
- Warning dual-lookup strategy: `champion_map` in `draft.rs` is keyed by display name (`c.name`), but `ChampionPoolEntry.champion` stores canonical Data Dragon IDs (from autocomplete). Built a reverse `id_to_champion: HashMap<&str, &Champion>` inside `compute_slot_warnings` to handle both.
- Followed locked CONTEXT.md decision: warnings only when BOTH "not in pool" AND "class gap exists". Picking a different Fighter when you already have Fighter coverage = no warning.
- Role override dropdowns use conventional draft order default (pick1=top, pick2=jungle, pick3=mid, pick4=bot, pick5=support) which coaches can override per-slot.

## Deviations from Plan

None - plan executed exactly as written. The champion ID vs display name discrepancy was anticipated by the plan ("Phase 2 normalization") but the actual mapping strategy was clarified during implementation. The dual-lookup approach handles both scenarios correctly and all unit tests pass.

## Issues Encountered
- Minor: The plan stated "both use canonical Data Dragon IDs since Phase 2 normalization" but in practice `draft_slots` stores display names (from `champ.name` on pick). Resolved by building the dual-lookup pattern in `compute_slot_warnings` -- works transparently with the test cases which use matching id/name values.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Pool warning badges are live for the draft planner. The `warning_slots` prop is optional so tree_drafter and other DraftBoard callers are unaffected.
- Plan 04-02 can proceed (next inline intel feature).

## Self-Check: PASSED

- FOUND: src/components/draft_board.rs
- FOUND: src/pages/draft.rs
- FOUND: .planning/phases/04-inline-intel/04-01-SUMMARY.md
- FOUND commit e6bed7e (Task 1)
- FOUND commit 96fc02f (Task 2)
- All 47 tests pass
- SSR and WASM targets compile

---
*Phase: 04-inline-intel*
*Completed: 2026-03-17*
