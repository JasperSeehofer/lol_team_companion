---
phase: 05-post-game-loop-polish
plan: "07"
subsystem: ui-empty-states
tags: [no-team-state, ux, empty-states, stats, champion-pool]
dependency_graph:
  requires: ["05-06"]
  provides: ["NoTeamState in stats.rs", "NoTeamState in champion_pool.rs"]
  affects: ["src/pages/stats.rs", "src/pages/champion_pool.rs"]
tech_stack:
  added: []
  patterns: ["has_team Resource via get_team_dashboard", "NoTeamState conditional rendering"]
key_files:
  created: []
  modified:
    - src/pages/stats.rs
    - src/pages/champion_pool.rs
decisions:
  - "Used same has_team Resource pattern from opponents.rs for consistency across all team-scoped pages"
metrics:
  duration: "~3 minutes"
  completed: "2026-03-17"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 2
---

# Phase 05 Plan 07: NoTeamState for Stats and Champion Pool Summary

**One-liner:** Added `NoTeamState` to stats.rs and champion_pool.rs using the `has_team` Resource pattern from opponents.rs, so users without a team see a "join a team" CTA instead of the generic empty-data state.

## What Was Done

Applied the established `has_team` Resource pattern (introduced in opponents.rs during plan 05-05) to two remaining team-scoped pages:

### stats.rs
- Added `NoTeamState` to the UI import
- Added `has_team` Resource (calls `get_team_dashboard()`) inside `StatsPage`
- Changed the `Ok(rows) if rows.is_empty()` branch to first check `user_has_team`: if true, shows the existing `EmptyState` with a "Link Riot Account" CTA; if false, shows `NoTeamState` with a /team/roster CTA

### champion_pool.rs
- Added `NoTeamState` to the UI import
- Added `has_team` Resource (calls `get_team_dashboard()`) inside `ChampionPoolPage`
- Changed the `if role_entries.is_empty()` early-return to check `user_has_team`: if true, shows the existing `EmptyState` with an "Add a Champion" CTA; if false, shows `NoTeamState`

Both pages correctly distinguish the "no team" condition (server returns `Ok(Vec::new())` for no-team users) from the "has team but no data" condition.

## Verification

- `grep -c "NoTeamState" src/pages/stats.rs` → 2 (import + usage)
- `grep -c "NoTeamState" src/pages/champion_pool.rs` → 2 (import + usage)
- `grep -c "has_team" src/pages/stats.rs` → 3 (resource definition + `.get()` + `user_has_team` binding)
- `grep -c "has_team" src/pages/champion_pool.rs` → 3 (resource definition + `.get()` + `user_has_team` binding)
- `cargo check --features ssr` → exit 0
- `cargo check --features hydrate --target wasm32-unknown-unknown` → exit 0

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- `src/pages/stats.rs` modified: FOUND
- `src/pages/champion_pool.rs` modified: FOUND
- Commit 40cbd1b: FOUND
