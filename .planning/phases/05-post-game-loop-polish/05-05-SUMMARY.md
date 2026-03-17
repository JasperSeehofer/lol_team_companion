---
phase: 05-post-game-loop-polish
plan: "05"
subsystem: ui
tags: [leptos, tailwind, empty-states, toast, skeleton, ux]

# Dependency graph
requires:
  - phase: 05-01
    provides: ToastContext, ToastKind, EmptyState, NoTeamState, SkeletonCard, SkeletonGrid components

provides:
  - Empty states with icons on all 8 remaining pages
  - NoTeamState on team-scoped pages (dashboard, roster, opponents)
  - Toast feedback on profile, stats, team_builder, dashboard, roster mutations
  - SkeletonCard/SkeletonGrid replacing "Loading..." text in all Suspense fallbacks

affects:
  - future UI phases that add new pages

# Tech tracking
tech-stack:
  added: []
  patterns:
    - EmptyState with icon + optional CTA used on all pages with empty data states
    - NoTeamState with team-check resource to distinguish no-team vs no-data
    - ToastContext via use_context in all mutation handlers

key-files:
  created: []
  modified:
    - src/pages/profile.rs
    - src/pages/stats.rs
    - src/pages/team_builder.rs
    - src/pages/team/dashboard.rs
    - src/pages/team/roster.rs
    - src/pages/champion_pool.rs
    - src/pages/opponents.rs
    - src/pages/action_items.rs

key-decisions:
  - "opponents.rs uses has_team resource (calls get_team_dashboard) to distinguish no-team vs no-opponents for correct empty state selection"
  - "Removed StatusMessage signals from dashboard (edit_msg, leave_msg, note_msg) — all replaced with toast.show.run calls"
  - "champion_pool empty state shown at role-level when role_entries is empty, not whole-pool level"

patterns-established:
  - "NoTeamState check: add has_team Resource using get_team_dashboard().ok().flatten().is_some() when page needs to distinguish no-team vs empty-data"
  - "Toast in sub-components: use_context::<ToastContext>() works in any sub-component since ToastProvider is at app root"

requirements-completed:
  - UX-01
  - UX-02
  - UX-03

# Metrics
duration: 30min
completed: 2026-03-17
---

# Phase 05 Plan 05: Empty States, Skeletons, and Toast Migration (Remaining Pages) Summary

**Empty states with contextual icons on all 8 remaining pages, NoTeamState on 3 team-scoped pages, toast feedback replacing inline green/red divs and StatusMessage on 5 pages**

## Performance

- **Duration:** 30 min
- **Started:** 2026-03-17T15:43:00Z
- **Completed:** 2026-03-17T15:52:29Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- 8 pages updated with contextual empty states (profile 🔗, stats 📊, team_builder ⚗️, roster 👥, champion_pool 🎯, opponents 🎭, action_items ✅, dashboard uses NoTeamState)
- All "Loading..." Suspense fallbacks replaced with SkeletonCard/SkeletonGrid
- Toast replaces inline StatusMessage and green/red divs on all mutating pages

## Task Commits

Each task was committed atomically:

1. **Task 1: profile, stats, team_builder** - `a864b7c` (feat)
2. **Task 2: dashboard, roster, champion_pool, opponents, action_items** - `0292fd8` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `src/pages/profile.rs` - Toast for Riot link/unlink + profile update, EmptyState 🔗 for unlinked account, SkeletonCard/SkeletonGrid
- `src/pages/stats.rs` - EmptyState 📊 with CTA to /profile, SkeletonCard rows replace Loading...
- `src/pages/team_builder.rs` - Toast replaces save_result/save_error, EmptyState ⚗️ for empty roster
- `src/pages/team/dashboard.rs` - NoTeamState for no-team, SkeletonCard for 5 Suspense fallbacks, toast for all mutations (team edit, leave, notes), removed StatusMessage
- `src/pages/team/roster.rs` - Toast for join request/link riot, EmptyState 👥 above create/join forms
- `src/pages/champion_pool.rs` - EmptyState 🎯 for empty role pool
- `src/pages/opponents.rs` - EmptyState 🎭 and NoTeamState with has_team resource
- `src/pages/action_items.rs` - EmptyState ✅ for empty/filtered list

## Decisions Made
- Used `has_team` resource in opponents.rs to properly distinguish no-team vs no-opponents empty states, avoiding misleading "go join a team" message for users who already have a team
- Removed `edit_msg`, `leave_msg`, `note_msg` signals entirely from dashboard rather than keeping them unused — cleaner code
- champion_pool empty state at role-level (per active role tab) rather than whole-pool level, matching the tab-based UX

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Remaining save_error references after signal removal in team_builder.rs**
- **Found during:** Task 1 (team_builder.rs migration)
- **Issue:** Three `save_error.set(...)` calls remained after removing the signal declaration
- **Fix:** Replaced all remaining `save_error.set(...)` with `toast.show.run(...)` calls
- **Files modified:** src/pages/team_builder.rs
- **Verification:** cargo check passes
- **Committed in:** a864b7c

**2. [Rule 1 - Bug] StatusMessage in TeamNotebook sub-component referenced removed note_msg signal**
- **Found during:** Task 2 (dashboard.rs migration)
- **Issue:** TeamNotebook sub-component had `note_msg`/`set_note_msg` signals used in 3 places after signal was removed from declaration
- **Fix:** Added `use_context::<ToastContext>()` to TeamNotebook, replaced all 3 set_note_msg calls with toast.show.run
- **Files modified:** src/pages/team/dashboard.rs
- **Verification:** cargo check passes
- **Committed in:** 0292fd8

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug from missed signal references)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered
- File linter reverted profile.rs, stats.rs, team_builder.rs during initial edits — re-applied changes using Write tool instead of Edit

## Next Phase Readiness
- All app pages now have consistent empty states, skeleton loading, and toast feedback
- UX-01, UX-02, UX-03 requirements satisfied across all pages
- Phase 05 post-game loop polish is complete

---
*Phase: 05-post-game-loop-polish*
*Completed: 2026-03-17*
