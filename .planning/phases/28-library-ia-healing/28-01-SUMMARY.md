---
phase: 28-library-ia-healing
plan: "01"
subsystem: database
tags: [surrealdb, growth-tiers, leptos, rust, models]

# Dependency graph
requires:
  - phase: 27-war-room
    provides: TeamGrowthCounts/TeamGrowthState pattern + get_team_growth_counts batch-count shape
provides:
  - LibraryGrowthCounts struct (match_syncs / post_game_learning / team_note)
  - LibraryGrowthState struct with from_counts + Default impl
  - get_library_growth_counts 3-statement batch COUNT query in db.rs
  - Four library_* unit tests pinning D-13 calibration thresholds
affects:
  - 28-05 (library.rs — consumes LibraryGrowthState via get_library_growth_state server fn)
  - Any future phase adjusting growth tier thresholds

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Library growth tier derivation: records [1,10,30], tomes/analysis [1,5,15] — D-13 calibration hypotheses"
    - "Batch COUNT GROUP ALL with per-table field name distinction (match.team_id vs *.team)"

key-files:
  created: []
  modified:
    - src/models/growth.rs
    - src/server/db.rs

key-decisions:
  - "D-13 thresholds are calibration hypotheses — records [1,10,30], tomes [1,5,15], analysis [1,5,15] — tunable post-beta without touching room UI"
  - "match table uses team_id field; post_game_learning and team_note use team field — Pitfall 5 documented in fn doc comment"

patterns-established:
  - "Library growth model mirrors TeamGrowth*: Counts struct + State struct + from_counts + Default, all in growth.rs"
  - "get_library_growth_counts: 3-statement batch, strip_prefix team:, owned bind, CountResult inner struct, count_of helper"

requirements-completed: [ROOM-05]

# Metrics
duration: 15min
completed: 2026-06-06
---

# Phase 28 Plan 01: Library Growth Data Layer Summary

**LibraryGrowthCounts + LibraryGrowthState with D-13 tier derivation and a 3-statement SurrealDB batch-count query, all unit-tested on both SSR and WASM targets**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-06T00:40:00Z
- **Completed:** 2026-06-06T00:55:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `LibraryGrowthCounts` (match_syncs / post_game_learning / team_note) and `LibraryGrowthState` (records_tier / tomes_tier / analysis_tier) to `src/models/growth.rs`, mirroring the `TeamGrowth*` template exactly
- Implemented `LibraryGrowthState::from_counts` with D-13 calibration thresholds and `impl Default` for the teamless-caller path
- Added four `library_*` unit tests that pin the threshold boundaries; all pass on SSR and WASM targets
- Added `get_library_growth_counts` to `src/server/db.rs` as a 3-statement batch COUNT query with correct per-table field names (`match.team_id` vs `post_game_learning.team` / `team_note.team`)

## Task Commits

1. **Task 1: Add LibraryGrowthCounts + LibraryGrowthState + unit tests to growth.rs** - `593bd65` (feat)
2. **Task 2: Add get_library_growth_counts batch count query to db.rs** - `0d34079` (feat)

## Files Created/Modified

- `src/models/growth.rs` — Added `LibraryGrowthCounts`, `LibraryGrowthState`, `from_counts`, `Default`, and four `library_*` unit tests
- `src/server/db.rs` — Added `LibraryGrowthCounts` to growth import, added `get_library_growth_counts` fn

## Decisions Made

- D-13 thresholds are calibration hypotheses as specified: records [1, 10, 30], tomes/analysis [1, 5, 15]. Documented with a one-line comment in `from_counts` so the context is clear without reading the plan.
- Pitfall 5 (field name mismatch between tables) documented directly in the `get_library_growth_counts` doc comment — this is the most likely source of silent bugs when the query is adapted later.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 05 (`library.rs`) can now call `get_library_growth_counts` and `LibraryGrowthState::from_counts` to derive the three atmosphere tier signals for `AtmosphereRoot`
- Both compile targets clean; four unit tests green
- No blockers

---
*Phase: 28-library-ia-healing*
*Completed: 2026-06-06*
