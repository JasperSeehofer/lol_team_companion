---
phase: 06-bug-fixes
plan: 02
subsystem: ui
tags: [leptos, drag-drop, hydration, suspense, tree-drafter, game-plan, regression-tests]

requires:
  - phase: 06-01
    provides: regression.spec.ts scaffold, helpers.ts, fixtures.ts

provides:
  - BUG-01 fix: tree drafter branch no longer freezes (cancel_autosave_timer + suppress_autosave before select_node)
  - BUG-02/PLAN-02 fix: View Game Plan href includes plan_id= query param; game_plan.rs auto-loads by plan_id
  - BUG-03 fix: team leader always appears in bench section (fallback insertion if not in any partition)
  - BUG-04 fix: nav auth-gated links wrapped in Suspense+Suspend (eliminates SSR vs hydration mismatch)
  - BUG-05 fix: filled draft slots draggable + dragstart sets text/x-source-slot; drop handler clears source
  - 5 regression tests implemented in regression.spec.ts (none skipped)

affects:
  - draft-page
  - game-plan-page
  - tree-drafter-page
  - team-dashboard
  - nav-component
  - draft-board-component

tech-stack:
  added: []
  patterns:
    - "Suspense+Suspend for Resource-gated nav links prevents SSR hydration mismatch"
    - "text/x-source-slot DataTransfer key for slot-to-slot drag tracking in DraftBoard"
    - "cancel_autosave_timer + suppress_autosave.set(true) before select_node in spawn_local branch flow"
    - "query.with(|q| q.get(key).map(|s| s.clone())) for owned Option<String> from query map"

key-files:
  created:
    - e2e/tests/regression.spec.ts (5 tests, previously scaffolded placeholders)
  modified:
    - src/components/nav.rs
    - src/components/draft_board.rs
    - src/pages/team/dashboard.rs
    - src/pages/tree_drafter.rs
    - src/pages/draft.rs
    - src/pages/game_plan.rs

key-decisions:
  - "Clear source slot BEFORE fill_slot in drop handler so 'already_used' guard does not block slot-to-slot moves"
  - "Use plan_id query param (not draft_id) for 'View Game Plan' link so game_plan.rs can load the correct existing plan"
  - "Plan ID auto-load Effect defined after load_plan Callback to ensure correct ordering in component init"

patterns-established:
  - "Slot-to-slot drag: read text/x-source-slot from DataTransfer, call on_slot_clear before on_slot_drop"
  - "Branch from node: cancel_autosave_timer + suppress_autosave.set(true) before select_node in spawn_local"

requirements-completed: [BUG-01, BUG-02, BUG-03, BUG-04, BUG-05, PLAN-02]

duration: 45min
completed: 2026-03-22
---

# Phase 06 Plan 02: Bug Fixes (BUG-01 through BUG-05, PLAN-02) Summary

**Fixed all 5 known regressions: tree drafter freeze, game plan nav, roster leader, hydration mismatch, and drag-drop source clearing, with 5 passing regression tests**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-03-22T12:00:00Z
- **Completed:** 2026-03-22T12:45:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Fixed BUG-01: Tree drafter branch-from-here now explicitly cancels pending autosave timer and sets suppress_autosave=true before select_node, closing the race window that caused UI freezes
- Fixed BUG-02/PLAN-02: "View Game Plan" link in draft.rs now includes `plan_id=<id>` query param; game_plan.rs has a new Effect that auto-loads the matching plan from the list when `?plan_id=X` is in the URL
- Fixed BUG-03: Team leader is guaranteed to appear in bench/subs section via a fallback insertion if their member record isn't found in any standard partition
- Fixed BUG-04: Auth-gated nav links now wrapped in `<Suspense fallback=...>{move || Suspend::new(async move { ... })}</Suspense>` — SSR renders empty fallback, WASM hydrates with real links, eliminating the hydration mismatch
- Fixed BUG-05: Filled slots in DraftBoard have `draggable="true"` and `on:dragstart` that writes `text/plain` + `text/x-source-slot` to DataTransfer; drop handler reads source slot, calls `on_slot_clear(src)` before `on_slot_drop` so the "already used" guard doesn't block slot-to-slot moves
- Implemented all 5 regression tests in `regression.spec.ts` (zero `test.skip` remaining)

## Task Commits

Each task was committed atomically:

1. **Task 1: BUG-03 roster leader, BUG-04 hydration Suspense, BUG-05 drag-drop** - `8f8be9b` (fix)
2. **Task 2: BUG-01 tree drafter freeze, BUG-02/PLAN-02 game plan nav** - `543b2c4` (fix)

## Files Created/Modified

- `src/components/nav.rs` - Auth-gated links wrapped in Suspense+Suspend for hydration safety
- `src/components/draft_board.rs` - draggable + dragstart on filled slots; source slot cleared on drop
- `src/pages/team/dashboard.rs` - Leader fallback insertion into bench partition
- `src/pages/tree_drafter.rs` - cancel_autosave_timer + suppress_autosave before select_node in branch flow
- `src/pages/draft.rs` - "View Game Plan" href now includes plan_id= query param
- `src/pages/game_plan.rs` - Effect to auto-load plan by ?plan_id=X query param using load_plan Callback
- `e2e/tests/regression.spec.ts` - 5 regression tests implemented (BUG-01 through BUG-05)

## Decisions Made

- Used `plan_id` (not `draft_id`) as the query param for "View Game Plan" — draft_id already serves prefill, plan_id serves direct plan loading
- Clear source slot BEFORE fill_slot call in drop handler to bypass "already used" guard (simpler than changing Callback signature)
- Plan ID auto-load Effect defined after `load_plan` Callback definition to maintain correct init order in component
- nav_links function: `auth_link_cls` cloned once at call site and re-cloned inside the async Suspend block to satisfy Rust's move semantics

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Leptos Suspense+Suspend closure in nav_links initially failed with "FnOnce" error because link class Strings were moved into the async block. Fixed by cloning `auth_link_cls` inside the `move ||` wrapper closure so each reactive invocation gets fresh clones.
- `query.read().get("plan_id")` returns a reference with short lifetime (tied to ReadGuard); fixed with `query.with(|q| q.get("plan_id").map(|s| s.clone()))` for an owned `Option<String>`.

## Next Phase Readiness

- All 5 known bugs fixed and regression-tested
- Both SSR and hydrate compile targets pass cleanly
- 49 unit tests all passing
- Ready for Phase 06 Plan 03 (or next phase UX polish work)

---
*Phase: 06-bug-fixes*
*Completed: 2026-03-22*
