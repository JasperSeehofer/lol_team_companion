---
phase: 10-opponents-redesign
plan: "01"
subsystem: backend/models
tags: [opponent, model, db, intel, mastery, otp-detection, integration-tests]
dependency_graph:
  requires: []
  provides: [OpponentPlayer.last_fetched, OpponentPlayer.mastery_data_json, OpponentPlayer.role_distribution_json, compute_intel, is_stale, create_opponent_with_players, update_opponent_player_intel]
  affects: [src/models/opponent.rs, src/server/db.rs, schema.surql, tests/db_opponents.rs]
tech_stack:
  added: []
  patterns: [serde(default) for backward compat, <string>cast for datetime deserialization, BEGIN/COMMIT transaction for atomic creates]
key_files:
  created: [tests/db_opponents.rs]
  modified: [src/models/opponent.rs, src/server/db.rs, schema.surql]
decisions:
  - "Two-step create_opponent_with_players (opponent first, then transaction for players) chosen over single-transaction subquery approach due to SurrealDB result indexing reliability"
  - "get_opponent query updated with <string>last_fetched cast to handle SurrealDB datetime->String deserialization"
metrics:
  duration_minutes: 25
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_changed: 4
---

# Phase 10 Plan 01: Opponent Data Model Extension Summary

Extended the opponent data model with mastery, role distribution, and last_fetched fields; implemented OTP detection and pool analysis via compute_intel(); added atomic DB functions for batch opponent creation and enriched data persistence.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Extend schema, models, compute_intel with unit tests | 168c58e | schema.surql, src/models/opponent.rs, src/server/db.rs |
| 2 | Add DB functions for batch creation and intel persistence | 88005ae | src/server/db.rs, tests/db_opponents.rs |

## What Was Built

### OpponentPlayer model extensions (src/models/opponent.rs)
- Added `last_fetched: Option<String>`, `mastery_data_json: Option<String>`, `role_distribution_json: Option<String>` with `#[serde(default)]` for backward compatibility
- Implemented `compute_intel()` — builds champion frequencies, detects OTP at strictly >60% threshold, deserializes mastery data
- Implemented `comfort_picks()` — top 3 champions with percentage of total games
- Implemented `pool_size()` — unique champion count via HashSet
- Implemented `role_distribution()` and `mastery_data()` — deserialize JSON fields with safe defaults
- Added `is_stale()` and `is_stale_with_now()` free functions for 7-day staleness detection
- 13 unit tests covering all OTP thresholds, frequency sorting, comfort picks, role distribution, and stale checks

### Schema additions (schema.surql)
- `DEFINE FIELD IF NOT EXISTS last_fetched ON opponent_player TYPE option<datetime>`
- `DEFINE FIELD IF NOT EXISTS mastery_data_json ON opponent_player TYPE option<string>`
- `DEFINE FIELD IF NOT EXISTS role_distribution_json ON opponent_player TYPE option<string>`

### DB functions (src/server/db.rs)
- `create_opponent_with_players` — creates opponent record then atomically creates N player slots in a transaction, returns `(opponent_id, player_ids)`
- `update_opponent_player_intel` — persists Riot API enriched data including `last_fetched = time::now()` server-side
- `DbOpponentPlayer` struct updated with 3 new fields + From impl mapping
- `get_opponent` query updated to cast `last_fetched` as string for deserialization

### Integration tests (tests/db_opponents.rs)
- `test_create_opponent_with_players` — verifies 5 players created with correct roles and summoner names
- `test_update_opponent_player_intel` — verifies all enriched fields + last_fetched set after intel update
- `test_create_opponent_with_empty_summoner_names` — verifies None summoner names handled correctly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] DbOpponentPlayer missing new fields blocked test compilation**
- **Found during:** Task 1 (TDD RED phase)
- **Issue:** Adding new fields to `OpponentPlayer` caused compile error in `db.rs` where `From<DbOpponentPlayer>` was missing the 3 new fields
- **Fix:** Updated `DbOpponentPlayer` struct and `From` impl in the same commit as the model changes
- **Files modified:** src/server/db.rs
- **Commit:** 168c58e

**2. [Rule 1 - Bug] Transaction result indexing off-by-one**
- **Found during:** Task 2 integration test run
- **Issue:** `BEGIN TRANSACTION` itself is a statement that returns an empty result at index 0; player CREATE results start at index 1, not 0
- **Fix:** Changed `response.take(i)` to `response.take(i + 1)` for player ID extraction
- **Files modified:** src/server/db.rs
- **Commit:** 88005ae

**3. [Rule 1 - Bug] datetime field deserialization failure in get_opponent**
- **Found during:** Task 2 integration test (test_update_opponent_player_intel failing)
- **Issue:** `last_fetched` stored as SurrealDB `datetime` type cannot be deserialized directly as `Option<String>` without explicit SQL cast — consistent with pattern used for `created_at` throughout the codebase
- **Fix:** Updated `get_opponent` SELECT query to include `<string>last_fetched AS last_fetched` cast
- **Files modified:** src/server/db.rs
- **Commit:** 88005ae

**4. [Rule 4 - Architecture note] Single-transaction CREATE with subquery avoided**
- **Found during:** Task 2 implementation design
- **Considered:** Using a single `BEGIN TRANSACTION` that creates opponent + players with a subquery `(SELECT VALUE id FROM opponent WHERE name = $opp_name ...)[0]` to get the opponent ID within the same transaction
- **Decision:** Used two-step approach (create opponent first, then transaction for players) to avoid fragility with name-based subqueries and ambiguity if team has multiple opponents with the same name
- **Not a deviation from plan spec** — plan said "use BEGIN/COMMIT for multi-step writes" which we do for the player batch

## Self-Check: PASSED

- src/models/opponent.rs: FOUND
- tests/db_opponents.rs: FOUND
- schema.surql: FOUND
- commit 168c58e: FOUND
- commit 88005ae: FOUND
- compute_intel: 1 instance
- is_stale_with_now: 1 instance
- comfort_picks: 1 instance
- create_opponent_with_players: 1 instance
- update_opponent_player_intel: 1 instance
- last_fetched = time::now(): confirmed in query
- BEGIN TRANSACTION: confirmed in create_opponent_with_players
