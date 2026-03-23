---
phase: 10-opponents-redesign
plan: "02"
subsystem: backend/api
tags: [opponent, riot-api, server-fn, intel, mastery, role-distribution]
dependency_graph:
  requires: [OpponentPlayer.mastery_data_json, OpponentPlayer.role_distribution_json, create_opponent_with_players, update_opponent_player_intel]
  provides: [fetch_player_intel, PlayerIntelData, create_opponent_with_players_fn, fetch_player_intel_fn]
  affects: [src/server/riot.rs, src/pages/opponents.rs]
tech_stack:
  added: []
  patterns: [serde_json for cross-wire serialization, HashMap for role distribution counting, HashSet for champion dedup]
key_files:
  created: []
  modified: [src/server/riot.rs, src/pages/opponents.rs]
decisions:
  - "Use engine::local::Db (not engine::any::Any) for DB type in server fns — consistent with all existing server functions in opponents.rs"
  - "Auth guard placed after empty-summoner check in fetch_player_intel_fn — empty name returns Ok(()) before any DB/auth work"
  - "Role distribution computed in server fn body as HashMap<String, u32> — avoids storing raw champion_with_role data and produces compact (role, count) pairs for DB storage"
metrics:
  duration_minutes: 15
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_changed: 2
---

# Phase 10 Plan 02: Riot API Data Pipeline Summary

Combined Riot API fetch function capturing champion names, per-match roles, and mastery data in a single call; two new server functions wiring the pipeline from Riot API through to DB persistence.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create fetch_player_intel in riot.rs | 7748e63 | src/server/riot.rs |
| 2 | Add new server functions in opponents.rs | 707ea69 | src/pages/opponents.rs |

## What Was Built

### PlayerIntelData struct and fetch_player_intel (src/server/riot.rs)

- `PlayerIntelData` struct with three fields:
  - `recent_champions: Vec<String>` — unique champion names played (deduped via HashSet)
  - `champion_with_role: Vec<(String, String)>` — (champion_name, team_position) per match, NOT deduplicated — needed for role distribution computation
  - `mastery_data: Vec<(String, i32, i32)>` — (champion_name, mastery_level, mastery_points)
- `fetch_player_intel(puuid, match_count)` — single function replacing separate `fetch_player_champions` + `fetch_champion_masteries` calls
- Extracts `p.team_position` from each match participant record (Riot API values: TOP, JUNGLE, MIDDLE, BOTTOM, UTILITY)
- Calls `fetch_champion_masteries(puuid)` after the match loop for mastery data

### New server functions (src/pages/opponents.rs)

**`create_opponent_with_players_fn(name, players_json) -> Result<(String, Vec<String>), ServerFnError>`**
- Accepts `players_json`: JSON-encoded `Vec<(String, Option<String>)>` where each tuple is (role, riot_summoner_name_or_none)
- Returns `(opponent_id, player_ids)` tuple so the UI can immediately trigger per-player fetches without a second round-trip
- Gets team_id via `db::get_user_team_id` — consistent with other server functions (AppUser has no team_id field)
- Returns Err("No team") if user has no team (creating an opponent without a team is not valid)

**`fetch_player_intel_fn(player_id, riot_summoner_name) -> Result<(), ServerFnError>`**
- Returns `Ok(())` immediately for empty summoner name — no error for optional field (per CLAUDE.md rule 44 pattern)
- Parses "GameName#TagLine" format, validates 2-part split
- Calls `riot::get_puuid` then `riot::fetch_player_intel(&puuid, 20)`
- Computes role distribution: `HashMap<String, u32>` counting occurrences of each team_position
- Serializes mastery_data and role_distribution to JSON strings via `serde_json::to_string`
- Persists all intel via `db::update_opponent_player_intel`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan spec referenced `user.team_id.unwrap()` but AppUser has no team_id field**
- **Found during:** Task 2 implementation
- **Issue:** The plan spec for `create_opponent_with_players_fn` said to call `user.team_id.unwrap()`, but `AppUser` in `src/server/auth.rs` has no `team_id` field — only `id`, `username`, `email`, `password_hash`, `riot_puuid`, `riot_summoner_name`
- **Fix:** Used `db::get_user_team_id(&surreal, &user.id)` to look up team_id, consistent with the pattern in all other team-scoped server functions (e.g., `get_opponents`)
- **Files modified:** src/pages/opponents.rs (no extra files — the fix was in the implementation choice)
- **Commit:** 707ea69

## Self-Check: PASSED

- src/server/riot.rs: FOUND
- src/pages/opponents.rs: FOUND
- commit 7748e63: FOUND
- commit 707ea69: FOUND
- PlayerIntelData struct: confirmed present
- fetch_player_intel function: confirmed present
- create_opponent_with_players_fn: confirmed present, returns (String, Vec<String>)
- fetch_player_intel_fn: confirmed present, calls riot::fetch_player_intel and db::update_opponent_player_intel
- cargo check --features ssr: passed
- cargo check --features hydrate --target wasm32-unknown-unknown: passed
- cargo test --features ssr --lib: 84 passed, 0 failed
