---
phase: 02-aggregation-layer
plan: 02
subsystem: server
tags: [rust, surrealdb, aggregation, dashboard, champion-performance, pool-gap-analysis]

# Dependency graph
requires:
  - 02-01 (normalize_champion_name, DashboardSummary, ChampionPerformanceSummary, PoolGapWarning model types)
provides:
  - migrate_champion_names function in db.rs (startup champion normalization)
  - compute_pool_gaps pure function (class dominance and missing class detection)
  - get_dashboard_summary batched query (5-statement single round-trip)
  - aggregate_champion_performance pure helper (testable without DB)
  - get_champion_performance_summary per-player query with 30-day/20-game window logic
  - get_team_champion_performance team-wide query with same window logic
affects:
  - Phase 3 (smart dashboard uses get_dashboard_summary)
  - Phase 4 (inline intel panels use get_champion_performance_summary)
  - main.rs (migrate_champion_names called at startup via init_db)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "5-statement batched SurrealDB query for dashboard summary (single round-trip)"
    - "compute_pool_gaps: pure Rust function with 70% class dominance threshold"
    - "aggregate_champion_performance: pure Rust helper for cross-feature champion stats"
    - "30-day OR 20-games dual window logic for performance queries"
    - "Best-effort startup migration via migrate_champion_names (Data Dragon failure is non-fatal)"

key-files:
  created: []
  modified:
    - src/server/db.rs

key-decisions:
  - "migrate_champion_names is best-effort: Data Dragon failure logs warning and returns Ok() — startup should not fail due to network issues"
  - "compute_pool_gaps takes pool entries + members slice (not user_id) — pure function enables unit testing without DB"
  - "aggregate_champion_performance is a public pure function — extracted for testability, called by both per-player and team-wide queries"
  - "Post-game win/loss per-champion not tracked in post_game_learning schema (no win field) — post_game_champ_outcomes returns empty Vec for now"
  - "Pool gap computation uses class tag counts not champion entry counts — multi-class champions (Fighter+Tank) contribute to both class buckets"

patterns-established:
  - "Internal async helpers (compute_pool_gaps_for_team) for complex team-scoped pool queries"
  - "User IN [list] pattern for multi-user pool queries (no JOIN needed in SurrealDB)"
  - "CountResult inline struct pattern for SELECT count() GROUP ALL queries"

requirements-completed: []

# Metrics
duration: 16min
completed: 2026-03-15
---

# Phase 2 Plan 02: Cross-Table Aggregation Queries and Champion Name Migration

**migrate_champion_names startup migration, compute_pool_gaps class diversity analysis, get_dashboard_summary 5-statement batched query, and get_champion_performance_summary dual-window aggregation functions in db.rs**

## Performance

- **Duration:** 16 min
- **Started:** 2026-03-15T07:05:34Z
- **Completed:** 2026-03-15T07:05:34Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- `migrate_champion_names` normalizes 7 tables on startup (champion_pool, draft_action, game_plan.our_champions, game_plan.enemy_champions, opponent_player.recent_champions, ban_priority, tree_node_action); best-effort with warning on Data Dragon failure
- `compute_pool_gaps` detects class dominance (>=70% of class tag distribution) and missing class coverage with opponent escalation; pure function, unit-testable
- `get_dashboard_summary` executes 5 SurrealQL statements in a single batched call, returning open action item count, recent action item previews (top 3), recent post-game previews (last 5), pool gap warnings, and workflow completeness counts
- `aggregate_champion_performance` extracted as pure Rust function aggregating draft picks, match stats, game plan appearances, and post-game outcomes into `Vec<ChampionPerformanceSummary>`
- `get_champion_performance_summary` and `get_team_champion_performance` implement "30 days OR 20 games, whichever is more" dual-window logic; both return `Ok(Vec::new())` for empty teams
- All new functions return `Ok(empty/default)` not `Err` for missing teams (CLAUDE.md rule 44)
- 38 tests pass (8 new: test_compute_pool_gaps_dominant_class, test_compute_pool_gaps_missing_class_opponent, test_compute_pool_gaps_balanced, test_dashboard_summary_assembly, test_dashboard_summary_empty_is_default, test_aggregate_champion_performance, test_aggregate_champion_performance_empty, test_champion_performance_sorted_by_match_games)

## Task Commits

1. **Task 1 + Task 2 combined: Add all aggregation functions** - `6a7e7f5` (feat)

Note: Tasks 1 and 2 were implemented in a single code pass. Both tasks' implementations and tests are included in commit `6a7e7f5`.

## Files Created/Modified
- `src/server/db.rs` - Added imports, `migrate_champion_names`, `compute_pool_gaps`, `get_dashboard_summary`, `compute_pool_gaps_for_team`, `aggregate_champion_performance`, `get_champion_performance_summary`, `get_team_champion_performance`; updated `init_db` to call migration; added 8 unit tests

## Decisions Made
- `migrate_champion_names` called from `init_db` (before `Arc::new`) so the bare `&Surreal<Db>` reference is available; wrapped in `if let Err` so a failed migration never prevents startup
- Pool gap computation uses class tag counts (not champion entry counts) so multi-class champions like Garen (Fighter+Tank) contribute to both class buckets — this reflects actual class coverage
- `aggregate_champion_performance` made public to allow testing without DB; all DB-calling functions delegate aggregation to this helper
- Post-game win/loss per-champion attribution left as empty Vec — the `post_game_learning` schema stores `improvements: Vec<String>` but no explicit win/loss field, so per-champion outcome from post-game is deferred

## Deviations from Plan

### Minor Implementation Differences

**1. [Rule 2 - Missing Critical Functionality] Tasks 1 and 2 implemented in single pass**
- **Found during:** Task 2 planning
- **Issue:** Plan assumed two separate implementation passes (Task 1 first, then Task 2), but both share the same imports and aggregation infrastructure
- **Fix:** Implemented all functions in one pass; all tests were written before implementation as required by TDD
- **Files modified:** src/server/db.rs
- **Commit:** 6a7e7f5

**2. Post-game win/loss per-champion not available**
- **Found during:** Task 2 implementation
- **Issue:** `post_game_learning` schema has `improvements: Vec<String>` but no win/loss boolean field; per-champion post-game win attribution is impossible without schema change
- **Fix:** `post_game_champ_outcomes` returns empty Vec; documented in key-decisions; schema change deferred (Rule 4 boundary — would require new field)
- **Impact:** `post_game_wins` and `post_game_losses` in ChampionPerformanceSummary will always be 0 until schema is extended
- **Files modified:** src/server/db.rs

## Issues Encountered

None beyond the post-game win/loss schema gap noted above.

## User Setup Required

None.

## Next Phase Readiness
- All Phase 3 dashboard data dependencies satisfied: `get_dashboard_summary` returns complete `DashboardSummary`
- Phase 4 intel panel dependencies satisfied: `get_champion_performance_summary` and `get_team_champion_performance` ready
- Champion name migration active at startup — existing records will be normalized on next server start
- Pool gap analysis with Data Dragon class tags operational; opponent escalation detection works when opponent_player data exists

---
*Phase: 02-aggregation-layer*
*Completed: 2026-03-15*
