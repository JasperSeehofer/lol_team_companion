---
phase: 07-ux-polish
plan: 02
subsystem: team-roster
tags: [ux, search, watermark, icons, roster]
dependency_graph:
  requires: ["07-01"]
  provides: ["team-search", "roster-watermarks"]
  affects: ["src/pages/team/roster.rs", "src/pages/team/dashboard.rs", "src/models/team.rs", "src/server/db.rs"]
tech_stack:
  added: []
  patterns: ["client-side filtering with search signal", "watermark absolute positioning with overflow-hidden clipping", "member count via SurrealDB subselect"]
key_files:
  created: []
  modified:
    - src/models/team.rs
    - src/server/db.rs
    - src/pages/team/roster.rs
    - src/pages/team/dashboard.rs
decisions:
  - "Bench card watermark uses inner overflow-hidden div to avoid clipping the role select dropdown"
  - "Coach watermark uses inline clipboard SVG (no Community Dragon icon available for coach)"
  - "member_count field is Option<u32> on Team so existing struct creation without the field still compiles"
metrics:
  duration: "~25 minutes"
  completed: "2026-03-22"
  tasks: 2
  files: 4
---

# Phase 07 Plan 02: Team Search and Roster Watermarks Summary

**One-liner:** Client-side team search with member count display and faded role icon watermarks on all roster card types.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Team search with member count and client-side filtering | 8e352ce | src/models/team.rs, src/server/db.rs, src/pages/team/roster.rs |
| 2 | Role watermark icons on starter, bench, and coach cards | c5db692 | src/pages/team/dashboard.rs |

## What Was Built

### Task 1: Team Search

- Added `member_count: Option<u32>` to the `Team` model struct (shared SSR/WASM)
- Added `member_count: Option<u32>` to `DbTeam` struct and its `From<DbTeam>` impl in `db.rs`
- Updated `list_all_teams()` query to compute member count via SurrealDB subselect: `(SELECT count() FROM team_member WHERE team = $parent.id GROUP ALL)[0].count AS member_count`
- Added `(search_query, set_search_query)` signal to `RosterPage`
- Replaced the direct team list with a three-state search UI:
  - Empty search query: "Type to search for teams..." prompt
  - Non-empty with no matches: "No teams match your search." message
  - Non-empty with results: horizontal flex cards showing name, region, member count, and styled "Request to Join" button

### Task 2: Roster Card Watermarks

- **Starter slot cards**: Added `relative overflow-hidden` to card container; watermark `<img>` using `role_icon_url(role)` at `opacity-10 invert`, positioned `absolute bottom-0 right-0` with slight overflow translate. No watermark rendered when `role_icon_url` returns empty string.
- **Bench cards**: Added `relative` to card container; inner `<div class="absolute inset-0 overflow-hidden pointer-events-none">` clips the watermark without affecting the role `<select>` dropdown that sits outside the inner div.
- **Coach cards**: Added `relative overflow-hidden` to card; inline clipboard SVG (`opacity-10 text-muted`) serves as watermark since Community Dragon has no coach icon.
- All watermark elements have `aria-hidden="true"` and `pointer-events-none select-none`.

## Verification

- `cargo check --features ssr`: clean
- `cargo check --features hydrate --target wasm32-unknown-unknown`: clean
- `cargo test --features ssr --lib`: 59 tests passed, 0 failed

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - all data is wired (member_count from real DB query, watermarks from real role_icon_url helper).

## Self-Check: PASSED

Files exist:
- src/models/team.rs — contains `pub member_count: Option<u32>`
- src/server/db.rs — contains `member_count` in DbTeam and list_all_teams query
- src/pages/team/roster.rs — contains `search_query`, "Type to search for teams..."
- src/pages/team/dashboard.rs — contains `opacity-10 invert pointer-events-none` and `opacity-10 text-muted pointer-events-none`

Commits verified:
- 8e352ce feat(07-02): add team search with member count to roster join section
- c5db692 feat(07-02): add watermark role icons to starter, bench, and coach cards
