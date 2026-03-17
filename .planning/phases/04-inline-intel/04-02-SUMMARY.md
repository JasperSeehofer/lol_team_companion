---
phase: 04-inline-intel
plan: 02
subsystem: ui
tags: [leptos, riot-api, riven, champion-mastery, opponent-intel, draft]

# Dependency graph
requires:
  - phase: 04-01
    provides: Opponent scouting model and basic opponent intel server fn
provides:
  - Enriched opponent intel with champion pick frequencies, OTP detection, and Riot API mastery data
  - fetch_champion_masteries in riot.rs using champion_mastery_v4 endpoint
  - OpponentPlayerIntel model struct in models/opponent.rs
  - get_opponent_intel_full server fn in draft.rs
  - Upgraded "Their Picks" sidebar in draft planner with badges and mastery section
affects: [draft, opponent-intel, riot-api]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Graceful Riot API degradation: check has_api_key() and Option<puuid> before calling API, return empty Vec on error"
    - "OTP detection: most-played champion >60% of scouted game count triggers badge"
    - "riven Champion::identifier() for Data Dragon canonical name — no manual lookup map needed"

key-files:
  created: []
  modified:
    - src/models/opponent.rs
    - src/server/riot.rs
    - src/pages/draft.rs

key-decisions:
  - "Used riven's Champion::identifier() for champion name mapping instead of building a Data Dragon key->name HashMap — riven enum already has static name lookup"
  - "get_opponent_intel_full is a new server fn alongside the existing get_opponent_intel (not a rename) to preserve backward compatibility"
  - "mastery_data stores top 10 entries; champion_frequencies stores all observed champions sorted descending"

patterns-established:
  - "Champion mastery section omitted entirely when empty — no placeholder needed, graceful absence is sufficient"
  - "No-puuid indicator shown only when puuid is None and mastery_data is also empty"

requirements-completed:
  - PIPE-04

# Metrics
duration: 40min
completed: 2026-03-17
---

# Phase 4 Plan 02: Opponent Intel Enhancement Summary

**Draft planner "Their Picks" sidebar upgraded with champion pick frequency counts, OTP detection badges, and Riot API champion mastery data via a new `get_opponent_intel_full` server fn that loads once per opponent selection.**

## Performance

- **Duration:** ~40 min
- **Started:** 2026-03-17T00:00:00Z
- **Completed:** 2026-03-17T00:40:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `OpponentPlayerIntel` model to `models/opponent.rs` with frequency, mastery, and OTP fields
- Added `fetch_champion_masteries` to `riot.rs` using riven's `champion_mastery_v4` endpoint — leverages `Champion::identifier()` for name lookup without a manual HashMap
- Added `get_opponent_intel_full` server fn that computes OTP detection, aggregates frequencies, and calls Riot API only when key and puuid are available
- Upgraded "Their Picks" sidebar: frequencies shown as `"Champion (N)"` badges, OTP badge in `bg-red-700`, mastery section with `"Lv.X (Yk)"` format, no-puuid indicator as muted italic text

## Task Commits

Each task was committed atomically:

1. **Task 1: Add fetch_champion_masteries to riot.rs and OpponentPlayerIntel model** - `31ba7bc` (feat)
2. **Task 2: Wire enhanced opponent intel server fn and upgrade Their Picks sidebar UI** - `c9e08ae` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/models/opponent.rs` - Added `OpponentPlayerIntel` struct with frequency, mastery, and OTP fields
- `src/server/riot.rs` - Added `fetch_champion_masteries` function calling `champion_mastery_v4` endpoint
- `src/pages/draft.rs` - Added `get_opponent_intel_full` server fn, updated Resource, upgraded sidebar rendering

## Decisions Made
- Used `riven::consts::Champion::identifier()` for Data Dragon canonical champion name — it returns the exact ID string (`"Jinx"`, `"AurelionSol"`, etc.) with no extra lookup needed
- Added `get_opponent_intel_full` as a new server fn rather than replacing `get_opponent_intel` — old fn retained for potential other callers
- Mastery section is entirely absent when empty — no placeholder text shown (cleaner UX)
- No-puuid indicator only shown when `riot_puuid.is_none()` AND `mastery_data.is_empty()` to avoid redundant messaging

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `OpponentPlayer` import after changing Resource type**
- **Found during:** Task 2 (updating import after changing Resource type from `OpponentPlayer` to `OpponentPlayerIntel`)
- **Issue:** Unused import warning from `cargo check --features ssr`
- **Fix:** Removed `OpponentPlayer` from the use statement, keeping only `OpponentPlayerIntel`
- **Files modified:** `src/pages/draft.rs`
- **Verification:** `cargo check --features ssr` reports no warnings
- **Committed in:** `c9e08ae` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug/cleanup)
**Impact on plan:** Minor unused import cleanup required after type migration. No scope change.

## Issues Encountered
- `riven::consts::Champion` is a newtype over `i16`, not a plain `i64` as the plan assumed. `Champion::identifier()` returns the Data Dragon canonical ID string directly, so no HashMap mapping from numeric ID is needed. This simplified the implementation.

## User Setup Required
None - Riot API mastery features degrade gracefully when `RIOT_API_KEY` is not set. No additional environment configuration required for this plan.

## Next Phase Readiness
- Opponent intel sidebar complete with frequency, OTP, and mastery data
- Sidebar keyed on `selected_opponent_id` only — no reactivity to pick changes
- Ready for phase 04-03 or further inline intel enhancements

---
*Phase: 04-inline-intel*
*Completed: 2026-03-17*
