---
phase: 04-inline-intel
plan: 03
subsystem: ui
tags: [leptos, surrealdb, rust, win-conditions, game-plan, analytics]

# Dependency graph
requires:
  - phase: 04-inline-intel
    provides: game_plan page with strategy tag and post-game learning infrastructure
provides:
  - Win condition history panel with visual bars on game plan page
  - filter_win_condition_stats pure helper function in db.rs
  - get_win_condition_stats_vs_opponent opponent-filtered DB query
  - get_strategy_win_rates_vs_opponent server fn in game_plan.rs
affects: [post-game, analytics, 05-gap-closure]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Pure helper functions for in-memory filtering extracted from async DB functions to enable unit testing
    - Two-tab (All-Time / vs Opponent) panel pattern with reactive Resource switching
    - Collapsible panels with toggle signal in Leptos view

key-files:
  created: []
  modified:
    - src/server/db.rs
    - src/pages/game_plan.rs

key-decisions:
  - "Used in-memory filter (Approach B) instead of complex SurrealQL join for opponent filtering — data volume is small (tens of game plans) and keeps code testable"
  - "Draft.opponent field (not opponent_name) is the correct field to join on for draft-based filtering"
  - "Tracker panel placed between Strategy Tag and Macro Strategy sections for contextual proximity to tag selection"

patterns-established:
  - "filter_* pure helpers: extract filtering logic from async DB fns into pure functions for unit testability"

requirements-completed:
  - INTL-02

# Metrics
duration: 35min
completed: 2026-03-17
---

# Phase 4 Plan 3: Win Condition Tracker Panel Summary

**Collapsible win condition history panel on game plan page showing per-tag win/loss bars with opponent-filtered view via Draft.opponent join**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-03-17T00:00:25Z
- **Completed:** 2026-03-17T00:35:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `filter_win_condition_stats` pure helper and `get_win_condition_stats_vs_opponent` to db.rs with full opponent-join logic
- Added `get_strategy_win_rates_vs_opponent` server fn to game_plan.rs following established auth/team-check pattern
- Added collapsible Win Condition History panel below Strategy Tag section with visual win/loss bars and win % display
- Current game plan's selected strategy tag is highlighted with accent border/background
- Two-tab toggle: All-Time (always enabled) vs Opponent (enabled when draft with opponent is linked)
- Empty state guidance message for users with no post-game data
- 3 unit tests covering filter_win_condition_stats edge cases (empty, no match, matching)

## Task Commits

Each task was committed atomically:

1. **Task 1: opponent-filtered win condition stats + server fn** - `53d9ab2` (feat)
2. **Task 2: WinConditionTracker panel** - `b7830b3` (feat)

## Files Created/Modified
- `src/server/db.rs` - Added filter_win_condition_stats pure helper, get_win_condition_stats_vs_opponent async fn, 3 unit tests
- `src/pages/game_plan.rs` - Added get_strategy_win_rates_vs_opponent server fn, tracker state signals, tracker panel view block

## Decisions Made
- In-memory filter (Approach B) chosen over SurrealQL join: data volume is small and pure helper is unit-testable without DB
- Draft field is `opponent` (not `opponent_name`) — discovered by reading DbDraft struct in db.rs
- DB query selects `draft` field from game_plan table (not `draft_id`) — field name follows DB schema, mapped to `draft_id` in GamePlan model

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed field name mismatch: draft.opponent_name -> draft.opponent**
- **Found during:** Task 1 (implementing get_win_condition_stats_vs_opponent)
- **Issue:** Plan specified `opponent_name` as the draft field to join on, but the actual DB struct uses `opponent`
- **Fix:** Changed DbDraftOpponent struct field and SurrealQL query to use `opponent`
- **Files modified:** src/server/db.rs
- **Verification:** cargo check --features ssr passes
- **Committed in:** 53d9ab2 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed field name mismatch: game_plan.draft_id -> game_plan.draft**
- **Found during:** Task 1 (implementing get_win_condition_stats_vs_opponent)
- **Issue:** Plan specified `draft_id` as the field name on game_plan, but the actual DB column is `draft`
- **Fix:** Changed PlanTagWithDraft struct field and SurrealQL query to use `draft`
- **Files modified:** src/server/db.rs
- **Verification:** cargo check --features ssr passes, all 47 tests pass
- **Committed in:** 53d9ab2 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs — field name mismatches between plan and actual DB schema)
**Impact on plan:** Both corrections necessary for correctness. No scope creep.

## Issues Encountered
- Rust compilation is slow (2+ min first build, 20s incremental) requiring patience during verification

## Next Phase Readiness
- Win condition tracker complete; INTL-02 satisfied
- Opponent-filtered stats available via get_strategy_win_rates_vs_opponent for future features
- Ready for remaining phase 04 plans

---
*Phase: 04-inline-intel*
*Completed: 2026-03-17*
