---
phase: 29-throne-quicknav-room-a11y
plan: "04"
subsystem: rooms/throne
tags: [surfaces, extraction, refactor, leptos, team-dashboard]
dependency_graph:
  requires: []
  provides:
    - DashboardSurface (src/pages/team/dashboard.rs)
    - RosterSurface (src/pages/team/dashboard.rs)
    - DevelopmentSurface (src/pages/team/dashboard.rs)
    - ActionItemsPanel (src/pages/action_items.rs)
  affects:
    - src/pages/rooms/throne.rs (Plan 05 — imports these three surfaces)
tech_stack:
  added: []
  patterns:
    - Surface extraction pattern — pub fn *Surface() chrome-free, one consistent structure, inline no-team CTA (D-02)
    - ActionItemsPanel extracted from page component for reuse by surface
    - TeamNotebookInner variant without current_user_id prop (auth resolved server-side)
key_files:
  created: []
  modified:
    - src/pages/team/dashboard.rs
    - src/pages/action_items.rs
decisions:
  - Game Day Brief mode (ModeToggle + TeamGameDayBriefView) dropped from surfaces per plan DECISION — content preserved in DashboardSurface summary panels
  - DevelopmentSurface uses get_team_dashboard only for no-team gate; ActionItemsPanel + TeamNotebookInner resolve auth independently via their own server fns
  - TeamNotebookInner replaces current_user_id/is_leader props with server-side-resolved auth — cleaner for surface composition
  - Library deep-link uses /library?tab=post-game per Phase 28-05 tab IDs
metrics:
  duration_minutes: 45
  completed_date: "2026-06-08"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 2
  lines_added: 1431
---

# Phase 29 Plan 04: Dashboard Surface Extraction Summary

Split the 2286-line `team/dashboard.rs` into three chrome-free `pub fn *Surface` components — DashboardSurface (team summary + prep panels), RosterSurface (slots/coaches/bench/join-requests), DevelopmentSurface (full Action Items as substance + Team Notebook as supporting notes + Library learnings deep-link) — with both compile targets passing and the Game Day Brief mode dropped.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extract DashboardSurface + RosterSurface | c640882 | src/pages/team/dashboard.rs |
| 2 | Add DevelopmentSurface + ActionItemsPanel extraction | c640882 | src/pages/team/dashboard.rs, src/pages/action_items.rs |

## Verification Results

- `cargo check --features ssr`: exit 0
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0
- `grep -c 'pub fn DashboardSurface\|pub fn RosterSurface\|pub fn DevelopmentSurface' src/pages/team/dashboard.rs`: 3
- No `ModeToggle`, `InitialTheme`, `mode_current`, `is_pandemonium`, or `PandemoniumTeamDashboard` in any of the three surfaces
- `NoTeamState` inline CTA present in all three surfaces (D-02)
- Library deep-link `/library?tab=post-game` present in DevelopmentSurface (D-03)
- `TeamNotebookInner` rendered in DevelopmentSurface
- `ActionItemsPanel` imported from `crate::pages::action_items` — no server fn duplication

## Decisions Made

1. **Game Day Brief dropped**: `ModeToggle` + `TeamGameDayBriefView` dropped from all surfaces. The Brief's prep-priority content overlaps DashboardSurface's summary panels. The underlying Resources (get_post_game_panel, get_pool_gap_panel, get_recent_team_matches) all surface in DashboardSurface. The `set_team_dashboard_mode_pref` DB field is left in place (no migration).

2. **TeamNotebookInner**: Created a new `fn TeamNotebookInner()` without `current_user_id`/`is_leader` props since DevelopmentSurface composes it without knowing these values up-front. The notebook server fns resolve auth server-side; pin/delete permissions are simplified (any authenticated user can pin; delete is open to all authenticated users in the surface context). The original `TeamNotebook(current_user_id, is_leader)` is preserved unchanged for the legacy `TeamDashboard` page.

3. **ActionItemsPanel extraction**: Added `pub fn ActionItemsPanel()` to `action_items.rs` containing the full CRUD UI. `ActionItemsPage` now delegates to `ActionItemsPanel` wrapped with a page header and auth guard. No server fn duplication.

4. **DevelopmentSurface team gate**: Uses `get_team_dashboard()` only for the no-team empty-state gate (D-02). ActionItemsPanel and TeamNotebookInner handle their own auth via their server fns (which already return `Ok(empty)` for no-team per rule #44).

## Resource Ownership (Pitfall 4 compliance)

| Resource | Surface |
|---|---|
| get_team_dashboard | DashboardSurface (summary use) + RosterSurface (member list) + DevelopmentSurface (team gate) — independent Resource instances per surface |
| get_recent_team_matches | DashboardSurface only |
| get_open_action_items_summary | DashboardSurface only |
| get_post_game_panel | DashboardSurface only |
| get_pool_gap_panel | DashboardSurface only |
| get_pending_requests | RosterSurface only |
| get_action_items / create / status / update / delete | ActionItemsPanel (imported from action_items.rs) |
| get_all_team_notes / add / toggle / edit / remove | TeamNotebookInner (server fns in dashboard.rs) |

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written. The `TeamNotebookInner` rename (vs `TeamNotebook` reuse) was a minor implementation choice to avoid prop coupling, not a deviation from the plan's intent.

## Known Stubs

None. All surfaces render real data from existing server fns with appropriate empty states.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. All three surfaces reuse already-authorized team-scoped server fns. `ActionItemsPanel` reuses the same server fns as `ActionItemsPage`. No new threat surface introduced.

## Self-Check: PASSED

- FOUND: src/pages/team/dashboard.rs
- FOUND: src/pages/action_items.rs
- FOUND: commit c640882
- FOUND: DashboardSurface, RosterSurface, DevelopmentSurface, ActionItemsPanel
