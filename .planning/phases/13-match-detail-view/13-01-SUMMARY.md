---
phase: 13-match-detail-view
plan: "01"
subsystem: match-detail
tags: [riot-api, cache, data-layer, surrealdb, models]
dependency_graph:
  requires: []
  provides: [match_detail_data_layer, match_detail_server_fn, match_detail_cache]
  affects: [src/pages/match_detail.rs, src/server/db.rs, src/server/riot.rs, src/models/match_data.rs, schema.surql]
tech_stack:
  added: []
  patterns:
    - cache-check-then-fetch (DB cache → Riot API fallback)
    - participants_json stored as JSON string in SCHEMAFULL table
    - timeline events as individual DB rows with serde_json involved_participants
key_files:
  created:
    - src/pages/match_detail.rs
  modified:
    - schema.surql
    - src/models/match_data.rs
    - src/server/riot.rs
    - src/server/db.rs
    - src/pages/mod.rs
decisions:
  - "Store participants as a single JSON string (participants_json) rather than individual rows — avoids 10-row JOIN complexity with no query benefits since all 10 are always fetched together"
  - "timeline_events stored as individual rows (not JSON) to allow future per-event queries and filtering"
  - "classify_event keeps monster_type/building_type params prefixed with _ — plan code showed them unused but kept for API compatibility"
metrics:
  duration_minutes: 8
  completed_date: "2026-03-27"
  tasks_completed: 2
  files_modified: 5
---

# Phase 13 Plan 01: Match Detail Data Layer Summary

Data layer for match detail view: schema tables, shared model types, Riot API full match + timeline extraction, DB caching, and the server function tying it all together with cache-check-then-fetch.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Schema tables, model types, Riot API extraction | 808b9d3 |
| 2 | DB cache functions, server function, unit tests | b73b275 |

## What Was Built

**Schema (`schema.surql`):** Two new SCHEMAFULL tables — `match_detail` (stores 10-player scoreboard as JSON + metadata) and `match_timeline_event` (one row per filtered event with UNIQUE index on match_id for detail, non-unique index for timeline).

**Model types (`src/models/match_data.rs`):** Five new shared structs/enums compiling for both SSR and WASM:
- `MatchParticipant` — all 10 player fields (items[0..5], damage, gold, team_position)
- `TimelineEvent` — processed events with `EventCategory` enum
- `EventCategory` — Objective/Tower/Kill/Ward/Teamfight/Recall
- `PerformanceStats` — damage share %, CS/min, vision, gold with lane opponent comparison
- `MatchDetail` — complete return type for server function
- `ComparisonMode` — GameAverage/LaneOpponent for UI toggle

**Riot API (`src/server/riot.rs`):**
- `fetch_full_match_detail` — fetches all 10 participants + timeline, detects teamfights (4+ participants in 10s window), synthesizes TEAMFIGHT events per cluster, sorts by timestamp
- `classify_event` — maps riven event types to `EventCategory`
- `compute_performance` — damage share %, CS/min, averages, lane opponent comparison

**DB cache (`src/server/db.rs`):**
- `get_cached_match_detail` — SELECT by match_id (not record ID), deserializes participants_json, fetches timeline rows
- `store_match_detail` — CREATE with .check(), timeline events stored individually
- `DbMatchDetail` / `DbTimelineEvent` — internal structs with SurrealValue derive
- `classify_db_event` — reconstructs EventCategory from stored event_type + is_teamfight flag

**Server function (`src/pages/match_detail.rs`):**
- `fetch_match_detail(match_id)` — cache-check-then-fetch, non-fatal cache write failure (tracing::warn), placeholder `MatchDetailPage` component for Plan 02

## Unit Tests Added

- `test_classify_event_types` — verifies all 5 event type mappings + 2 ignored types
- `test_compute_performance_stats` — 10-player fixture, damage share %, CS/min, lane opponent detection

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unused `MatchDetail` import in riot.rs**
- **Found during:** Task 2 (cargo check warnings)
- **Issue:** Plan code imported `MatchDetail` from models but riot.rs only returns `FullMatchData`
- **Fix:** Removed `MatchDetail` from the use statement
- **Files modified:** src/server/riot.rs

**2. [Rule 1 - Bug] Dead code in teamfight detection loop in plan**
- **Found during:** Task 1 — plan had a malformed inner loop comment block
- **Fix:** Rewrote the kill_events loop with direct iteration over `kill_events[i..]` using a clean `for (jdx, jts, jparticipants)` pattern

**3. [Rule 2 - Missing critical functionality] serde_json import missing**
- **Found during:** Task 2 — db.rs had no serde_json import
- **Fix:** Added `use serde_json;` to db.rs imports (serde_json was already in Cargo.toml)

## Known Stubs

- `MatchDetailPage` component in `src/pages/match_detail.rs` is a placeholder `<p>"Match detail page placeholder"</p>` — will be fully implemented in Plan 02.

## Self-Check: PASSED

Files exist:
- schema.surql contains "DEFINE TABLE IF NOT EXISTS match_detail" ✓
- schema.surql contains "DEFINE TABLE IF NOT EXISTS match_timeline_event" ✓
- src/models/match_data.rs contains MatchParticipant, MatchDetail, TimelineEvent, PerformanceStats ✓
- src/server/riot.rs contains fetch_full_match_detail, compute_performance ✓
- src/server/db.rs contains get_cached_match_detail, store_match_detail ✓
- src/pages/match_detail.rs contains fetch_match_detail, MatchDetailPage ✓
- src/pages/mod.rs contains "pub mod match_detail" ✓

Tests: 92 passed, 0 failed (cargo test --features ssr --lib)
Commits: 808b9d3, b73b275 verified in git log
