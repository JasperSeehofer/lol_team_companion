---
phase: 28-library-ia-healing
plan: "04"
subsystem: library-surfaces
tags: [surface-extraction, analytics, post-game, personal-learnings, leptos, library-room]
dependency_graph:
  requires: []
  provides: [AnalyticsSurface, PostGameSurface, PersonalLearningsSurface]
  affects: [src/pages/analytics.rs, src/pages/post_game.rs, src/pages/personal_learnings.rs]
tech_stack:
  added: []
  patterns:
    - "Chrome-strip via AnalyticsSurface: thin AnalyticsPage delegates to surface component"
    - "Chrome-strip via PostGameSurface: CRUD Resource/signal state fully duplicated in surface, PostGamePage thin wrapper with SectionHead+Card retained"
    - "Chrome-strip via PersonalLearningsSurface: + New learning CTA moved to surface header row, /personal-learnings/new href preserved (D-11)"
    - "No new recursion_limit directive in post_game.rs — project-wide limit in lib.rs/main.rs is sufficient (rule #38)"
key_files:
  created: []
  modified:
    - src/pages/analytics.rs
    - src/pages/post_game.rs
    - src/pages/personal_learnings.rs
decisions:
  - "PostGameSurface carries all CRUD state (reviews/plans/drafts/match_ids Resources, editor signals, URL query param loading Effects) verbatim — PostGamePage becomes a canvas-grain/SectionHead/Card shell only; this avoids any shared-state coupling"
  - "PersonalLearningsSurface strips page <h1> but keeps + New learning CTA in a surface-level header div (D-11: /personal-learnings/new route must remain reachable)"
  - "AnalyticsPage delegates to AnalyticsSurface rather than duplicating — cleaner than PostGame approach because analytics has no region-aware Card/SectionHead wrapper in its standalone view"
metrics:
  duration_minutes: 35
  completed_date: "2026-06-06"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 3
---

# Phase 28 Plan 04: Analytics, Post-Game & Personal Learnings Surface Extraction Summary

Chrome-free `AnalyticsSurface`, `PostGameSurface`, and `PersonalLearningsSurface` extracted from their originating page files. All three strip page chrome (auth guard, canvas-grain/min-h-screen wrapper, page title block) per D-04 and keep every data Resource, signal, and CRUD flow verbatim. None reads `InitialTheme` (rule #58). The originating page components are retained for their flat routes. The `+ New learning` CTA in PersonalLearningsSurface still points at `/personal-learnings/new` (D-11). No new `recursion_limit` directive in `post_game.rs` (rule #38).

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Extract AnalyticsSurface (chrome-free) | 9141fb2 | src/pages/analytics.rs |
| 2 | Extract PostGameSurface (chrome-free, no new recursion_limit) | 399a356 | src/pages/post_game.rs |
| 3 | Extract PersonalLearningsSurface (chrome-free, /new href preserved) | 17e1206 | src/pages/personal_learnings.rs |

## What Was Built

### AnalyticsSurface (`src/pages/analytics.rs`)

Chrome-free component at `pub fn AnalyticsSurface()`. Strips: `#[cfg(feature="hydrate")]`-gated auth guard, outer `<div class="canvas-grain bg-base min-h-screen">` + `<main>` wrapper, Imperial header section (`The ledger` / `<h1>Analytics`). Keeps: `analytics` Resource, `sort_state: RwSignal<(SortColumn, SortDir)>`, `open_plan: RwSignal<Option<String>>`, and the full `Suspense` + `Suspend::new(async move { match analytics.await { ... } })` view verbatim including all tag-summary cards, the plan-effectiveness table with sortable columns, and the accordion expansion.

`AnalyticsPage` is a thin wrapper that keeps the auth guard and Imperial header, then delegates to `<AnalyticsSurface />` inside the existing `<main>` padding block.

### PostGameSurface (`src/pages/post_game.rs`)

Chrome-free component at `pub fn PostGameSurface()`. Strips: auth guard, `let theme = use_context::<InitialTheme>()...; let region = ...` lines, outer `<div class="canvas-grain bg-base min-h-screen" data-feedback-label="Post game">` wrapper, `SectionHead` header, outer `Card` wrapper. Keeps verbatim: all four Resources (`reviews`, `plans`, `drafts`, `match_ids`), URL query-param Effects (`?review_id`, `?plan_id`, `?draft_id`), `clear_editor`, `load_review`, `build_review`, `do_save`, `do_delete`, and the full two-column layout (review list + pattern analysis on the left, editor with linking/outcome/rating/feedback/save on the right).

`PostGamePage` is a thin wrapper with auth guard, `SectionHead` (region-aware), `Card` (region-aware), and delegates to `<PostGameSurface />` inside the card. No new `#![recursion_limit]` in `post_game.rs` — rule #38 already covers this project-wide.

### PersonalLearningsSurface (`src/pages/personal_learnings.rs`)

Chrome-free component at `pub fn PersonalLearningsSurface()`. Strips: auth guard, outer `<div class="canvas-grain bg-base min-h-screen">` wrapper, page header block with `<h1>Personal learnings</h1>` and eyebrow text. Keeps: `learnings` Resource, `filter_type`/`filter_champion`/`filter_tag`/`sort_mode` RwSignals, `expanded_id` RwSignal, and the full `Suspense` view including filter bar, card grid (flat and champion-grouped modes), and empty/filter-empty states.

The `+ New learning` CTA is preserved in a surface-level header div with `href="/personal-learnings/new"` (D-11). The empty-state inner CTA (`Write your first learning`) also preserves the same href. `NewLearningPage` is unmodified. `PersonalLearningsPage` is a thin wrapper with auth guard, the page title block, and delegates to `<PersonalLearningsSurface />`.

## Deviations from Plan

None — plan executed exactly as written. All three surfaces extracted cleanly following the D-04 recipe. No compile errors, no Rule 1/2/3 fixes needed.

## Known Stubs

None. All three surfaces wire real data from existing server functions:
- `AnalyticsSurface` → `get_analytics_data()`
- `PostGameSurface` → `list_reviews()`, `list_plans_for_postgame()`, `list_drafts_for_postgame()`, `get_recent_match_ids()`
- `PersonalLearningsSurface` → `list_learnings()`

The `use_context::<RwSignal<Option<String>>>()` shared match-selection signal (from Plan 03) is NOT used by any of these three surfaces — they are independent of Stats/Match Detail.

## Threat Flags

None. No new server functions, no new trust boundaries. All three surfaces call existing authorized server fns unchanged. Auth is preserved at the `/library` route level (Plan 05) and at each retained `*Page` flat route.

## Self-Check: PASSED

- `AnalyticsSurface` defined: `grep -q 'pub fn AnalyticsSurface' src/pages/analytics.rs` ✓
- `AnalyticsPage` retained: `grep -q 'pub fn AnalyticsPage' src/pages/analytics.rs` ✓
- `PostGameSurface` defined: `grep -q 'pub fn PostGameSurface' src/pages/post_game.rs` ✓
- `PostGamePage` retained: `grep -q 'pub fn PostGamePage' src/pages/post_game.rs` ✓
- No `recursion_limit` in post_game.rs ✓
- `PersonalLearningsSurface` defined: `grep -q 'pub fn PersonalLearningsSurface' src/pages/personal_learnings.rs` ✓
- `PersonalLearningsPage` retained: `grep -q 'pub fn PersonalLearningsPage' src/pages/personal_learnings.rs` ✓
- `NewLearningPage` retained: `grep -q 'pub fn NewLearningPage' src/pages/personal_learnings.rs` ✓
- `/personal-learnings/new` href preserved: `grep -q '/personal-learnings/new' src/pages/personal_learnings.rs` ✓
- No `canvas-grain`/`min-h-screen` in AnalyticsSurface body ✓
- No `canvas-grain`/`min-h-screen` in PostGameSurface body ✓
- No `canvas-grain`/`min-h-screen` in PersonalLearningsSurface body ✓
- No `use_context::<InitialTheme>` in any of the three surfaces ✓
- Commits 9141fb2, 399a356, 17e1206 exist in git log ✓
- SSR compile target: exit 0 ✓
- WASM/hydrate compile target: exit 0 ✓
