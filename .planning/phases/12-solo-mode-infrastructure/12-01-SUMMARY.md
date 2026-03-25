---
phase: 12-solo-mode-infrastructure
plan: "01"
subsystem: infrastructure
tags: [solo-mode, riot-api, schema, db, region, ranked]
dependency_graph:
  requires: []
  provides:
    - schema.mode_on_user
    - schema.riot_region_on_user
    - schema.last_solo_sync_on_user
    - schema.synced_by_on_match
    - schema.ranked_snapshot_table
    - model.PublicUser.mode
    - model.PublicUser.riot_region
    - model.RankedInfo
    - model.AppUser.mode
    - model.AppUser.riot_region
    - riot.platform_route_from_str
    - riot.account_region_for
    - riot.fetch_ranked_data
    - riot.RankedEntry
    - db.get_user_mode
    - db.set_user_mode
    - db.set_user_region
    - db.update_last_solo_sync
    - db.store_ranked_snapshot
    - db.get_latest_ranked_snapshot
    - db.get_solo_matches
    - db.store_matches_with_synced_by
  affects:
    - pages/profile.rs (get_current_user now returns mode + riot_region)
    - pages/stats.rs (fetch_match_history platform parameterized)
    - pages/post_game.rs (fetch_match_history uses user.riot_region)
    - pages/opponents.rs (get_puuid, fetch_player_intel, fetch_player_champions parameterized)
    - pages/team/roster.rs (link_riot_account uses user.riot_region)
    - pages/draft.rs (fetch_champion_masteries parameterized)
    - bin/seed.rs (get_puuid parameterized)
tech_stack:
  added: []
  patterns:
    - platform_route_from_str/account_region_for: two-stage region routing (platform -> regional)
    - store_matches_with_synced_by: backward-compatible wrapper pattern
    - ignored tokio::test stubs for DB behavioral contracts
key_files:
  created: []
  modified:
    - schema.surql
    - src/models/user.rs
    - src/server/auth.rs
    - src/pages/profile.rs
    - src/server/riot.rs
    - src/server/db.rs
    - src/pages/stats.rs
    - src/pages/post_game.rs
    - src/pages/opponents.rs
    - src/pages/team/roster.rs
    - src/pages/draft.rs
    - src/bin/seed.rs
decisions:
  - "EUW fallback for opponents: opponent players have no associated user region — hardcode EUW as safe fallback; can be parameterized in Phase 13 when solo sync UI is added"
  - "store_matches_with_synced_by wrapper: preserved backward-compatible store_matches() API to avoid touching all call sites; solo sync will call store_matches_with_synced_by directly"
  - "get_solo_matches uses SELECT * pattern: matches existing get_player_stats approach; avoids complex aliased SELECT that could break SurrealValue derive"
  - "ranked division field: riven e.rank field is Option<Division> not String — used Debug format for clean string output"
metrics:
  duration_seconds: 390
  completed_date: "2026-03-25"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 12
  files_created: 0
---

# Phase 12 Plan 01: Solo Mode Infrastructure Foundation Summary

Schema additions, model updates, Riot API multi-region support, ranked data fetch infrastructure, and solo match DB queries.

## One-liner

Multi-region Riot API support with schema `mode`/`riot_region`/`ranked_snapshot` additions and solo match DB query layer.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Schema additions + model/auth struct updates | 8a013a2 | schema.surql, models/user.rs, auth.rs, profile.rs |
| 2 | Riot API region parameterization + ranked data + DB functions | 59130eb | riot.rs, db.rs, stats.rs, post_game.rs, opponents.rs, roster.rs, draft.rs, seed.rs |
| 3 | Unit tests for region mapping + DB function stubs | f953876 | riot.rs, db.rs |

## What Was Built

### Schema (schema.surql)
- `mode` field on `user` (string, DEFAULT 'solo')
- `riot_region` field on `user` (option<string>)
- `last_solo_sync` field on `user` (option<datetime>)
- `synced_by` field on `match` (option<record<user>>)
- `ranked_snapshot` table (user, queue_type, tier, division, lp, wins, losses, snapshotted_at)
- Index: `ranked_snapshot_user_queue` on (user, queue_type)

### Models
- `PublicUser` extended with `mode: String` and `riot_region: Option<String>`
- `RankedInfo` struct added (queue_type, tier, division, lp, wins, losses) — shared SSR+WASM
- `AppUser` and `DbUser` in auth.rs extended with same fields
- `get_current_user` in profile.rs now populates mode and riot_region

### Riot API (riot.rs)
- `platform_route_from_str(region: &str) -> PlatformRoute`: maps 15 region codes + fallback
- `account_region_for(platform: PlatformRoute) -> RegionalRoute`: maps platform to account regional grouping
- All 5 existing functions parameterized with `platform: PlatformRoute`
- `fetch_ranked_data(puuid, platform)` added calling `league_v4().get_league_entries_by_puuid`
- `RankedEntry` struct (queue_type, tier, division, lp, wins, losses)
- No hardcoded `RegionalRoute::EUROPE` or `PlatformRoute::EUW1` in API call paths

### DB Functions (db.rs)
- `get_user_mode` / `set_user_mode`: mode CRUD with "solo" default
- `set_user_region`: update riot_region on user record
- `update_last_solo_sync`: set last_solo_sync = time::now()
- `store_ranked_snapshot`: create ranked_snapshot record
- `get_latest_ranked_snapshot`: fetch most recent snapshot by user+queue_type
- `get_solo_matches`: SELECT player_match by user with optional queue_id filter
- `store_matches_with_synced_by`: backward-compatible wrapper adding synced_by to match CREATE

### Tests
- `platform_route_from_str_all_regions`: 16 assertions covering all 15 regions + fallback
- `account_region_mapping`: 16 assertions covering all 4 regional groupings
- 5 `#[tokio::test] #[ignore]` DB stubs documenting expected behavior

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `fetch_ranked_data` division field type mismatch**
- **Found during:** Task 2 compilation
- **Issue:** riven's `LeagueEntryDto.rank` is `Option<Division>` (not `String`), causing E0308 type mismatch
- **Fix:** Added `.map(|d| format!("{:?}", d)).unwrap_or_default()` to convert Option<Division> to String via Debug
- **Files modified:** src/server/riot.rs
- **Commit:** 59130eb (included in same task commit)

None other — plan executed cleanly after the one type fix.

## Known Stubs

None — all functions return real data structures. DB test stubs are intentionally marked `#[ignore]` as documented behavioral contracts, not functional stubs.

## Self-Check: PASSED

- [x] schema.surql contains `DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo'`
- [x] schema.surql contains `DEFINE TABLE IF NOT EXISTS ranked_snapshot SCHEMAFULL`
- [x] src/models/user.rs contains `pub mode: String` in PublicUser
- [x] src/models/user.rs contains `pub struct RankedInfo`
- [x] src/server/riot.rs contains `pub fn platform_route_from_str`
- [x] src/server/riot.rs contains `pub fn account_region_for`
- [x] src/server/riot.rs contains `pub async fn fetch_ranked_data`
- [x] src/server/db.rs contains `pub async fn get_user_mode`
- [x] src/server/db.rs contains `pub async fn store_ranked_snapshot`
- [x] src/server/db.rs contains `pub async fn get_solo_matches`
- [x] Commits 8a013a2, 59130eb, f953876 exist
- [x] `cargo check --features ssr` passes
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` passes
- [x] `cargo test --features ssr --lib` passes (90 passed, 5 ignored)
