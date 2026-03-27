---
phase: 14-personal-learnings-journal
plan: "03"
subsystem: ui
tags: [leptos, rust, match-detail, learning-journal, query-params]

# Dependency graph
requires:
  - phase: 14-01
    provides: personal_learning table and /personal-learnings/new route

provides:
  - Updated scoreboard "Add Learning" CTA with match_id and result query params
  - Timeline event "+ Add Learning from this event" CTA with 7 query params including event_ts, event_name, tag_hint
  - Event-type to tag mapping (ELITE_MONSTER_KILL/BUILDING_KILL -> Objective+Control, CHAMPION_KILL -> Teamfighting)

affects:
  - 14-01 (learning form reads the query params this plan passes)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pre-clone Strings before view! macro when same value is moved into a closure AND passed as a component prop"
    - "PerformanceSection component extended with match_id and user_win props for CTA context"

key-files:
  created: []
  modified:
    - src/pages/match_detail.rs

key-decisions:
  - "Pre-clone user_champion/opponent_champion before view! macro so they can be moved into the event detail closure AND separately passed to PerformanceSection"
  - "Use event.timestamp_ms (raw ms) as event_ts param value; formatting to M:SS is done by the receiving form page"
  - "Use detail_text.replace(' ', '+') for basic URL-safe encoding of event name (browser percent-encodes <a href> automatically)"

patterns-established:
  - "Pre-clone pattern: when a String is captured by a move closure inside view! AND also needed by a later component, clone before the view! block"

requirements-completed:
  - LEARN-02

# Metrics
duration: 6min
completed: 2026-03-27
---

# Phase 14 Plan 03: Match Detail CTAs Summary

**Scoreboard "Add Learning" CTA extended with match_id+result params; timeline events get a new "+ Add Learning from this event" link with 7 query params including event context and tag hints**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-27T16:30:00Z
- **Completed:** 2026-03-27T16:36:17Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Updated `PerformanceSection` component to accept `match_id` and `user_win` props, enabling the scoreboard CTA to pass 4 query params (champion, opponent, match_id, result)
- Added "+ Add Learning from this event" link to the timeline event detail panel with all 7 query params including event_ts, event_name, and tag_hint
- Implemented event-type to tag suggestion mapping: ELITE_MONSTER_KILL/BUILDING_KILL maps to "Objective+Control", CHAMPION_KILL maps to "Teamfighting"

## Task Commits

Each task was committed atomically:

1. **Task 1: Update scoreboard "Add Learning" CTA** — combined with Task 2 in `361b97e` (feat)
2. **Task 2: Add timeline event "Add Learning" CTA** — `361b97e` (feat)

**Plan metadata:** see final docs commit

## Files Created/Modified
- `src/pages/match_detail.rs` - Updated PerformanceSection props, scoreboard CTA href, added timeline event detail CTA with full query param construction

## Decisions Made
- Pre-cloned `user_champion` and `opponent_champion` before the `view!` macro as `user_champion_for_perf` and `opponent_champion_for_perf` — necessary because the `move ||` closure for the event detail panel captures the originals by move, and `PerformanceSection` needs owned String values separately
- Used `event.timestamp_ms` directly as the `event_ts` URL param value (raw milliseconds); the form receiving page is responsible for display formatting
- Kept Tasks 1 and 2 in a single commit since they are in the same file and logically cohesive

## Deviations from Plan

None — plan executed exactly as written. The only unplanned work was adding pre-clones before the `view!` macro to resolve ownership, which is a direct consequence of the plan's own requirement to use `user_champion`/`opponent_champion` in both the event detail closure and `PerformanceSection`.

## Issues Encountered
- Rust ownership constraint: `user_champion` and `opponent_champion` are `String` (non-Copy), so they cannot be both moved into the event detail `move ||` closure and also passed to `PerformanceSection`. Resolved by pre-cloning before the `view!` macro.

## Next Phase Readiness
- Match detail CTAs are fully wired; the learning form (14-01) already reads all query params via `use_query_map()`
- LEARN-02 requirement satisfied: one-click learning creation from match detail with full context pre-filled

---
*Phase: 14-personal-learnings-journal*
*Completed: 2026-03-27*
