---
phase: 26-room-system-foundations
plan: 01
subsystem: database
tags: [rust, surrealdb, leptos, growth-model, war-room]

# Dependency graph
requires:
  - phase: solo-journey (pre-26)
    provides: GrowthCounts/GrowthState user-scoped pair (structural template mirrored here)
provides:
  - TeamGrowthCounts — six directly-team-scoped accumulation counts
  - TeamGrowthState — War Room tactics/intel/command tier derivation (D-06/07/08)
  - get_team_growth_counts — batched team-scoped count query
affects: [war-room, 27, library, throne, 28, 29]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Team-scoped growth pair mirrors the user-scoped GrowthCounts/GrowthState shape"
    - "Batched six count() GROUP ALL statements in one .query(), team-scoped via type::record('team', $team_key)"

key-files:
  created: []
  modified:
    - src/models/growth.rs
    - src/server/db.rs

key-decisions:
  - "Only War Room axes derived now (D-03); Library/Throne deferred to Phases 28/29"
  - "player_match dropped — user-scoped, has no team field (D-05)"
  - "D-09 thresholds documented inline as beta-tuning calibration hypotheses, non-blocking"

patterns-established:
  - "TeamGrowthState::from_counts reuses the existing module-level tier_for free fn"
  - "get_team_growth_counts mirrors get_growth_counts; key stripped + bound as owned String"

requirements-completed: [ROOM-01]

# Metrics
duration: 9min
completed: 2026-06-04
---

# Phase 26 / Plan 01: Team-Scoped Growth Model Summary

**TeamGrowthCounts + TeamGrowthState deriving War Room tactics/intel/command tiers, plus a batched team-scoped get_team_growth_counts query — the data foundation every new room compiles against.**

## Performance

- **Duration:** ~9 min (inline execution)
- **Completed:** 2026-06-04
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `TeamGrowthCounts` with the six directly-team-scoped fields (draft, draft_tree, opponent, ban_priority, post_game_learning, team_note)
- `TeamGrowthState::from_counts` deriving tactics (draft+draft_tree, [1,5,20]), intel (opponent, [1,3,10]), command (ban_priority+post_game_learning, [1,5,15])
- `get_team_growth_counts` — one round-trip, six `count() GROUP ALL` statements scoped by `type::record('team', $team_key)`
- Five new unit tests (all green); both compile targets clean

## Task Commits

1. **Task 1: TeamGrowthCounts + TeamGrowthState + War Room derivation + tests** - `8f60812` (feat)
2. **Task 2: get_team_growth_counts batched query** - `5acc538` (feat)

## Files Created/Modified
- `src/models/growth.rs` - Added team-scoped growth pair (counts + state + War Room tier derivation) and five unit tests
- `src/server/db.rs` - Added `get_team_growth_counts`; extended growth import to include `TeamGrowthCounts`

## Decisions Made
None beyond plan — followed D-03/04/05/06/07/08/09 as specified. Library/Throne axes intentionally absent.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None. (Note: this plan originally failed under parallel-worktree execution — three executor streams idle-timed-out on cold worktree compiles. Re-executed inline on the warm main-tree target.)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- War Room (Phase 27) can now consume `get_team_growth_counts` → `TeamGrowthState` to drive atmosphere overlays.
- Verification: `cargo test --features ssr --lib growth` → 11 passed (5 new team tests); `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` both exit 0.

## Self-Check: PASSED
- TeamGrowthCounts/TeamGrowthState present with correct fields; no records/tomes/analysis/roster/progression/crown fields.
- get_team_growth_counts present, team-scoped, player_match absent, key bound as owned String.
- Both targets compile; five team unit tests pass.

---
*Phase: 26-room-system-foundations*
*Completed: 2026-06-04*
