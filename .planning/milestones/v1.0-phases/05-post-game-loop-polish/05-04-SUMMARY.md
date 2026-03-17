---
phase: 05-post-game-loop-polish
plan: 04
subsystem: ui
tags: [leptos, toast, skeleton, ux, feedback]

# Dependency graph
requires:
  - phase: 05-post-game-loop-polish
    plan: 01
    provides: "ToastContext, ToastKind, SkeletonCard, SkeletonLine, SkeletonGrid primitives"

provides:
  - "champion_pool.rs: toast for all mutation feedback, skeleton Suspense fallbacks"
  - "opponents.rs: toast for all mutation feedback, skeleton Suspense fallbacks, error_msg prop removed from OpponentDetail/PlayerCard"
  - "post_game.rs: toast for save/update/delete feedback, action_item_count banner preserved, skeleton fallbacks"
  - "action_items.rs: toast for add/status-change/delete feedback, skeleton Suspense fallbacks"

affects: [ux, ui-feedback, pages]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "All mutation feedback via ToastContext.show.run — no local signal rendering"
    - "ErrorBanner retained for load-failure Err branches (resource-level errors)"
    - "SkeletonCard/SkeletonLine in Suspense fallbacks replacing Loading... text"

key-files:
  created: []
  modified:
    - src/pages/champion_pool.rs
    - src/pages/opponents.rs
    - src/pages/post_game.rs
    - src/pages/action_items.rs

key-decisions:
  - "Removed error_msg prop from OpponentDetail and PlayerCard — both now use_context::<ToastContext>() directly"
  - "action_item_count inline banner in post_game.rs preserved as-is (PIPE-02 banner is not toast)"
  - "change_status shows 'Marked complete' when transitioning to done, 'Status updated' otherwise"

patterns-established:
  - "Sub-components that need toast access call use_context::<ToastContext>() themselves rather than receiving a signal prop"

requirements-completed: [UX-02, UX-03]

# Metrics
duration: 20min
completed: 2026-03-17
---

# Phase 05 Plan 04: Toast + Skeleton Migration (Batch 2) Summary

**Toast migration for champion_pool, opponents, post_game, and action_items — removing 4 local status/error signal patterns and replacing Loading... fallbacks with skeleton components**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-03-17T16:00:00Z
- **Completed:** 2026-03-17T16:20:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Migrated 4 pages from local status_msg/error_msg signals to ToastContext
- Removed error_msg prop from OpponentDetail and PlayerCard sub-components (they now use_context directly)
- Preserved post_game action_item_count inline banner from PIPE-02
- Replaced all "Loading..." Suspense fallbacks with SkeletonCard/SkeletonLine

## Task Commits

1. **Task 1: Migrate champion_pool.rs and opponents.rs to toast + skeletons** - `cd51974` (feat)
2. **Task 2: Migrate post_game.rs and action_items.rs to toast + skeletons** - `743699d` (feat)

## Files Created/Modified

- `src/pages/champion_pool.rs` - Removed status_msg signal; added ToastContext; all mutations show toast; skeleton fallbacks for notes sections
- `src/pages/opponents.rs` - Removed error_msg/status_msg; added ToastContext to page + OpponentDetail + PlayerCard; skeleton fallbacks for list and detail panel
- `src/pages/post_game.rs` - Removed status_msg signal; added ToastContext; Review saved/updated/deleted toasts; skeleton fallback for review list; action_item_count preserved
- `src/pages/action_items.rs` - Removed error_msg signal; added ToastContext; Action item add/status toasts; skeleton fallbacks for stats and item list

## Decisions Made

- Removed `error_msg: RwSignal<Option<String>>` prop from `OpponentDetail` and `PlayerCard`. These sub-components now call `use_context::<ToastContext>()` directly — cleaner than prop-drilling a signal.
- `change_status` in action_items distinguishes between "Marked complete" (done transition) and "Status updated" (other transitions) for clearer feedback.

## Deviations from Plan

None — plan executed exactly as written. The sub-component prop removal was implicit in the plan since it told us to "Remove error_msg signal declarations" and both OpponentDetail and PlayerCard were directly using that signal.

## Issues Encountered

Pre-existing compile errors exist in draft.rs and tree_drafter.rs (from prior plans in the working tree, not yet committed). These errors were confirmed present before this plan's execution and are out of scope.

## Next Phase Readiness

- All 4 pages now fully use ToastContext for mutation feedback
- Plans 03 and 04 together cover the full UX-02/UX-03 page surface
- Ready for plan 05 (final polish/verification)

---
*Phase: 05-post-game-loop-polish*
*Completed: 2026-03-17*
