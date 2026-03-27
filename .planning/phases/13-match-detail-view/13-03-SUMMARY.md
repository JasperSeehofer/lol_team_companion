---
phase: 13-match-detail-view
plan: "03"
subsystem: match-detail
tags: [leptos, ui, timeline, filter, event-markers]
dependency_graph:
  requires: [match_detail_data_layer, match_detail_page]
  provides: [timeline_section, timeline_filter_bar, event_detail_panel]
  affects: [src/pages/match_detail.rs, e2e/tests/match-detail.spec.ts]
tech_stack:
  added: []
  patterns:
    - timeline-bar-absolute-positioning (timestamp_ms / game_ms * 100%)
    - filter-toggle-pills (signal-driven active/inactive CSS classes)
    - click-to-expand-detail (selected_event: Option<usize> signal)
    - user-event-accent-ring (ring-2 ring-accent ring-offset-1 ring-offset-base)
key_files:
  created:
    - e2e/tests/match-detail.spec.ts
  modified:
    - src/pages/match_detail.rs
decisions:
  - "Event markers use index into full timeline_events vec (not filtered vec) as stable key — avoids index drift when filters toggle"
  - "Empty filter state rendered inside bar track div with absolute positioning — keeps bar height stable"
  - "Filter buttons use inline class closures per button rather than a shared helper fn — avoids Callback::new overhead for 6 simple toggles"
metrics:
  duration_minutes: 3
  completed_date: "2026-03-27"
  tasks_completed: 1
  files_modified: 2
---

# Phase 13 Plan 03: Timeline Section Summary

Timeline visualization for match detail page: filter bar with 6 event category toggles, horizontally-positioned event markers with team coloring and user accent ring highlight, click-to-expand event detail panel, and e2e smoke test.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Timeline section with filter bar, event markers, and detail panel | 314c8fe |

## Task 2: Checkpoint (Awaiting Human Verification)

Task 2 is a `checkpoint:human-verify` gate. It requires manual visual verification of the complete match detail page. See checkpoint details below.

## What Was Built

**Timeline section (`src/pages/match_detail.rs`):**
- `timeline_pct()` — converts event timestamp_ms to left percentage position on bar (clamped 0–100)
- `event_tooltip()` — generates human-readable tooltip text per event type (CHAMPION_KILL, BUILDING_KILL, ELITE_MONSTER_KILL, WARD_PLACED, TEAMFIGHT)
- 7 new signals: `show_objectives`, `show_towers`, `show_kills`, `show_wards`, `show_teamfights`, `show_recalls`, `selected_event`
- `TimelineFilterBar`: 6 pill toggle buttons with active (`bg-accent text-accent-contrast`) and inactive states
- `TimelineBar`: 40px track with absolutely-positioned event markers; marker size/shape by event category (8px/12px/16px circles and squares); team-colored (blue-400/red-400); user's own events ring-highlighted (`ring-2 ring-accent ring-offset-1 ring-offset-base`)
- Click handler toggles `selected_event` — second click on same marker clears selection
- Detail panel (bg-surface border rounded-lg p-4 mt-3) with event text + involved participants list
- Empty filter state: "No events match the current filters." centered in bar track

**E2e smoke test (`e2e/tests/match-detail.spec.ts`):**
- Navigates to `/match/test-match-id` as authenticated user
- Verifies body and nav are visible with no WASM panic (1s wait)

## Deviations from Plan

None — plan executed exactly as written. Event marker `class=move ||` closure uses pre-computed `btn_class`/`btn_class_selected` string locals to avoid capturing complex expressions in the reactive closure (CLAUDE.md rule 18).

## Checkpoint Details

**Type:** human-verify
**What to verify:**
1. Start dev server: `cargo leptos watch`
2. Register/login at http://127.0.0.1:3020
3. Link Riot account on profile page
4. Navigate to Stats page, sync matches
5. Click any match row to navigate to /match/{id}
6. Verify timeline section:
   - Timeline bar shows event markers at time positions
   - Filter toggles hide/show event categories
   - Clicking a marker shows detail panel below
   - User's own events have accent ring highlight
7. Verify scoreboard, performance bars, and back navigation
8. Test error state: disconnect API key, refresh, verify error banner + retry

## Known Stubs

None — all timeline functionality is wired to real data from `d.timeline_events` and `d.participants`.

## Self-Check: PASSED

Files exist:
- src/pages/match_detail.rs contains "show_objectives" (4 occurrences) ✓
- src/pages/match_detail.rs contains "show_towers" (4 occurrences) ✓
- src/pages/match_detail.rs contains "show_kills" (4 occurrences) ✓
- src/pages/match_detail.rs contains "show_wards" (4 occurrences) ✓
- src/pages/match_detail.rs contains "show_teamfights" (4 occurrences) ✓
- src/pages/match_detail.rs contains "selected_event" (6 occurrences) ✓
- src/pages/match_detail.rs contains "fn timeline_pct" ✓
- src/pages/match_detail.rs contains "fn event_tooltip" ✓
- src/pages/match_detail.rs contains "ring-2 ring-accent" ✓
- src/pages/match_detail.rs contains "No events match the current filters." ✓
- src/pages/match_detail.rs contains "Recalls" ✓
- src/pages/match_detail.rs contains "show_recalls" ✓
- src/pages/match_detail.rs contains "Objectives" ✓
- src/pages/match_detail.rs contains "Timeline" ✓
- e2e/tests/match-detail.spec.ts exists ✓
- cargo check --features ssr: PASSED ✓
- cargo check --features hydrate --target wasm32-unknown-unknown: PASSED ✓
- cargo test --features ssr --lib: 92 passed, 0 failed ✓

Commit 314c8fe verified in git log ✓
