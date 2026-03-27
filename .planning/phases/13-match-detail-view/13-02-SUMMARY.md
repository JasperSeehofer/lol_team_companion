---
phase: 13-match-detail-view
plan: "02"
subsystem: match-detail
tags: [leptos, ui, riot-api, routing, scoreboard, performance]
dependency_graph:
  requires: [match_detail_data_layer, match_detail_server_fn]
  provides: [match_detail_page, match_detail_route, match_row_navigation]
  affects: [src/pages/match_detail.rs, src/app.rs, src/pages/stats.rs, src/pages/solo_dashboard.rs]
tech_stack:
  added: []
  patterns:
    - scoreboard-table-with-item-icons (Data Dragon CDN item images)
    - performance-bar-chart (normalized fill + avg marker)
    - comparison-toggle (GameAverage vs LaneOpponent mode)
    - auth-redirect (Resource::new get_current_user + Effect)
key_files:
  created: []
  modified:
    - src/pages/match_detail.rs
    - src/app.rs
    - src/pages/stats.rs
    - src/pages/solo_dashboard.rs
decisions:
  - "retain stats.rs expand-on-click behavior; added Details link inside row that stops propagation — preserves existing UX while adding detail navigation"
  - "solo_dashboard match rows wrapped in <a> element; inner <div> retains styling including cursor-pointer"
  - "_game_duration_secs prefixed with underscore — PerformanceSection currently unused but kept for future timeline integration"
metrics:
  duration_minutes: 20
  completed_date: "2026-03-27"
  tasks_completed: 2
  files_modified: 4
---

# Phase 13 Plan 02: Match Detail Page UI Summary

Full MatchDetailPage component with scoreboard tables, performance breakdown bars, and click-to-navigate from stats page and solo dashboard.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Match detail page with scoreboard and performance sections | a36234b |
| 2 | Click-to-navigate from stats page and solo dashboard | e331e41 |

## What Was Built

**Route (`src/app.rs`):** Added `<Route path=path!("/match/:id") view=MatchDetailPage />` and import. The `/match/:id` route is now registered in the Leptos router.

**Match detail page (`src/pages/match_detail.rs`):** Complete replacement of the Plan 01 placeholder:
- Auth redirect: `Resource::new(get_current_user)` + `Effect` with `#[cfg(feature = "hydrate")]` guard
- Route param: `use_params_map()` extracts `id`, fed into `Resource::new(match_id, fetch_match_detail)`
- Skeleton loading: 5 `SkeletonCard` placeholders during Suspense
- Error state: `ErrorBanner` + Retry button that calls `detail.refetch()`
- Page header: champion icon, summoner name, KDA, Victory/Defeat badge, game mode + duration
- `TeamScoreboard` component: team-colored header (blue-500/20 or red-500/20), column headers, `ParticipantRow` for each of 5 players
- `ParticipantRow`: user row highlighted with `border-l-4 border-accent bg-accent/10`; item icons from `ddragon.leagueoflegends.com/cdn/15.6.1/img/item/`; empty slots rendered as `<div class="w-6 h-6 rounded bg-elevated border border-divider/30" />`
- `PerformanceSection`: heading "My Performance"; two-button comparison toggle (vs Game Average / vs Lane Opponent); 4 `PerformanceBar` components (Damage Share, Vision Score, CS per Minute, Gold Earned)
- `PerformanceBar`: label, fill bar (bg-accent/70), avg marker (bg-muted/50), numeric value, verdict ("Above average" / "Below average" / "Average")
- "Add Learning ->" CTA linking to `/personal-learnings/new?champion=...&opponent=...` (will 404 until Phase 14)

**Stats page navigation (`src/pages/stats.rs`):** Added "Details" link inside each match summary row. The link uses `format!("/match/{}", riot_id)` and calls `ev.stop_propagation()` to prevent triggering the expand toggle.

**Solo dashboard navigation (`src/pages/solo_dashboard.rs`):** Wrapped each match row `<div>` in an `<a href=format!("/match/{}", m.match_id)>` element with `cursor-pointer hover:bg-elevated/50 transition-colors` classes.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

- "Add Learning ->" CTA in PerformanceSection: links to `/personal-learnings/new` which does not exist yet. Will be built in Phase 14. The link is intentional as a forward-pointing CTA.

## Self-Check: PASSED

Files exist:
- src/pages/match_detail.rs contains "use_params_map" ✓
- src/pages/match_detail.rs contains "Resource::new" ✓
- src/pages/match_detail.rs contains "SkeletonCard" ✓
- src/pages/match_detail.rs contains "ErrorBanner" ✓
- src/pages/match_detail.rs contains "bg-accent/10" ✓
- src/pages/match_detail.rs contains "border-l-4 border-accent" ✓
- src/pages/match_detail.rs contains "ddragon.leagueoflegends.com/cdn/15.6.1/img/item" ✓
- src/pages/match_detail.rs contains "My Performance" ✓
- src/pages/match_detail.rs contains "vs Game Average" ✓
- src/pages/match_detail.rs contains "vs Lane Opponent" ✓
- src/pages/match_detail.rs contains "ComparisonMode" ✓
- src/pages/match_detail.rs contains "Add Learning" ✓
- src/app.rs contains 'path!("/match/:id")' ✓
- src/app.rs contains "match_detail::MatchDetailPage" ✓
- src/pages/stats.rs contains 'format!("/match/' ✓
- src/pages/stats.rs contains "cursor-pointer" ✓
- src/pages/solo_dashboard.rs contains 'format!("/match/' ✓
- src/pages/solo_dashboard.rs contains "cursor-pointer" ✓

Tests: 92 passed, 0 failed (cargo test --features ssr --lib)
Commits: a36234b, e331e41 verified
