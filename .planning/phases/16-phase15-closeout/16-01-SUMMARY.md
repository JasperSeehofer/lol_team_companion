---
phase: 16-phase15-closeout
plan: "01"
subsystem: pages/solo_dashboard
tags:
  - leptos
  - resource-refetch
  - signal-hoist
  - surrealdb
dependency_graph:
  requires: []
  provides:
    - "WR-01 closed: LP history graph and goal cards now refetch after sync without page reload"
    - "get_solo_matches fixed: matches now appear on /solo (RecordId field alias bug resolved)"
  affects:
    - src/pages/solo_dashboard.rs
    - src/server/db.rs
tech_stack:
  added: []
  patterns:
    - "Hoist Resource + RwSignal from child to parent, pass as props"
    - "DbSoloMatch intermediate struct for RecordId → String aliasing"
key_files:
  modified:
    - src/pages/solo_dashboard.rs
    - src/server/db.rs
decisions:
  - "D-01..D-04: Hoist both lp_window and lp_history_resource into SoloDashboardPage; pass as props; three explicit .refetch() calls per sync path"
  - "get_solo_matches fix: SELECT * returns match/user as RecordId — aliased via match.match_id AS match_id and <string>user AS user_id with DbSoloMatch intermediate struct"
  - "Rule 40: ORDER BY game_end DESC (alias) not match.game_end (idiom) for SurrealDB 3.x partial SELECT compatibility"
metrics:
  duration_minutes: 30
commits:
  task_1: "b5930bc — WR-01 hoist lp_window + lp_history_resource into SoloDashboardPage"
  bonus_fix_1: "922e0f8 — get_solo_matches RecordId field alias fix"
  bonus_fix_2: "4c9431f — add game_end to SELECT for ORDER BY (Rule 40)"
  bonus_fix_3: "293e43a — ORDER BY alias not idiom in get_solo_matches"
self_check: PASSED

## Summary

WR-01 (stale LP graph and goal cards after sync) resolved by hoisting `lp_window` and `lp_history_resource` from inside `LpHistoryGraph` up to `SoloDashboardPage`, then adding `goal_progress_resource.refetch()` and `lp_history_resource.refetch()` to both the auto-sync Effect and the `do_sync` click handler. Browser verification confirmed all three resources (dashboard, goal progress, LP history) refresh after sync without a page reload.

During verification, a separate pre-existing bug surfaced: `get_solo_matches` was doing `SELECT *` which returned `match` and `user` as `RecordId` types. These didn't match `PlayerMatchStats.match_id`/`.user_id` field names, causing `unwrap_or_default()` to silently return an empty vec. Fixed with a `DbSoloMatch` intermediate struct and explicit column aliases (`match.match_id AS match_id`, `<string>user AS user_id`, `<string>match.game_end AS game_end`). Also required ORDER BY to reference the alias `game_end`, not the idiom `match.game_end` (Rule 40 variant).

## Deviations

- **Bonus fix in scope**: `get_solo_matches` bug discovered during checkpoint verification was fixed inline (3 additional commits). Closes a silent data regression present since Phase 12.
- Both compile targets pass clean; 102 lib tests pass.
- Human checkpoint approved.
