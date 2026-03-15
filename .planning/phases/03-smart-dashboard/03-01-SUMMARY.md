---
phase: 03-smart-dashboard
plan: 01
subsystem: ui
tags: [leptos, dashboard, post-game, champion-pool, action-items, suspense]

# Dependency graph
requires:
  - phase: 02-aggregation-layer
    provides: get_dashboard_summary db function, DashboardSummary/PostGamePreview/PoolGapWarning model structs
provides:
  - get_post_game_panel server function returning Vec<PostGamePreview>
  - get_pool_gap_panel server function returning Vec<PoolGapWarning>
  - Three independently-loading Suspense panels on team dashboard
  - Empty state CTAs linking to /post-game and /champion-pool
affects: [dashboard, post-game, champion-pool, action-items]

# Tech tracking
tech-stack:
  added: []
  patterns: [independent-suspense-boundaries, resource-at-component-scope]

key-files:
  created: []
  modified:
    - src/pages/team/dashboard.rs

key-decisions:
  - "Action items Resource moved to component top-level (was inside Suspense closure — prevented independent loading)"
  - "Three separate Resources at component scope ensure three separate Suspense boundaries that load independently"
  - "get_post_game_panel and get_pool_gap_panel each call get_dashboard_summary independently rather than sharing a single aggregation Resource"

patterns-established:
  - "Dashboard panel pattern: server fn + Resource at component top-level + Suspense with .into_any() on all branches"
  - "Empty state CTA pattern: empty branch renders text + <A> link to relevant feature page"

requirements-completed: [INTL-01]

# Metrics
duration: 15min
completed: 2026-03-15
---

# Phase 03 Plan 01: Smart Dashboard Panels Summary

**Three independently-loading dashboard panels added: action items (fixed), post-game reviews, and pool gap warnings — each with empty state CTAs linking to relevant feature pages**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-15T08:00:00Z
- **Completed:** 2026-03-15T08:15:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Added `get_post_game_panel` server function returning `Vec<PostGamePreview>` from db aggregation layer
- Added `get_pool_gap_panel` server function returning `Vec<PoolGapWarning>` from db aggregation layer
- Fixed action items Resource bug: moved from inside Suspense closure to component top-level so it loads independently
- Added Recent Reviews panel with date, top 2 improvements per review, and "+N more" indicator
- Added Pool Gap Warnings panel with yellow "!" attention indicator, username/role, missing classes, dominant class badge, and opponent threat escalation label
- Empty states for both new panels provide contextual CTAs to /post-game and /champion-pool
- All 38 tests pass; both SSR and WASM/hydrate targets compile cleanly

## Task Commits

Each task was committed atomically:

1. **Task 1: Add server functions and fix action items resource** - `dd9b0ac` (feat)
2. **Task 2: Add post-game and pool-gap panels to view** - `68c9b99` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/pages/team/dashboard.rs` - Added 2 server fns, 3 Resources at component scope, 2 new panel sections in view

## Decisions Made
- Action items Resource was incorrectly created inside a `move ||` closure wrapping the Suspense, which caused it to be recreated on each render cycle rather than loading once at component mount. Moving it to component top-level (alongside `dashboard`, `requests`, `recent_matches`) is the correct Leptos 0.8 pattern.
- Each of the three panels gets its own server function and Resource rather than a single shared aggregation Resource — this matches the research recommendation for independent loading so a slow panel doesn't block others.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Smart dashboard panels complete and independently loading
- Phase 03 Plan 02 can proceed: further dashboard intelligence features
- The panel pattern (server fn + Resource + Suspense + empty state CTA) is established and ready to replicate

---
*Phase: 03-smart-dashboard*
*Completed: 2026-03-15*
