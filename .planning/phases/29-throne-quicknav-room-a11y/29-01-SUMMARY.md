---
phase: 29-throne-quicknav-room-a11y
plan: "01"
subsystem: growth-model
tags: [growth, throne, data-layer, blended, tdd]
dependency_graph:
  requires: []
  provides: [ThroneGrowthCounts, ThroneGrowthState, get_throne_growth_counts]
  affects: [src/models/growth.rs, src/server/db.rs]
tech_stack:
  added: []
  patterns: [blended-growth-derivation, batch-count-query, tdd-red-green]
key_files:
  created: []
  modified:
    - src/models/growth.rs
    - src/server/db.rs
decisions:
  - "ThroneGrowthState has NO counts field (unlike Team/Library analogs) — progression axis is user-scoped, not team-scoped, so the blended constructor takes team counts + Option<RankedInfo> separately"
  - "Progression thresholds [1, 20, 60] are D-05/D-07 calibration hypotheses, tunable post-beta without touching room UI"
  - "get_throne_growth_counts mirrors get_team_growth_counts caller contract exactly — pre-authorized team_id, parameterized type::record(), aggregate counts only"
metrics:
  duration: "13 minutes"
  completed: "2026-06-08"
  tasks_completed: 2
  files_modified: 2
---

# Phase 29 Plan 01: Throne Growth Data Layer Summary

**One-liner:** Blended Throne growth data layer — `ThroneGrowthState::from_parts` derives roster/crown from team counts and progression independently from the viewing user's ranked games (D-05/D-07), backed by a 2-statement batch count query.

## What Was Built

### Task 1: ThroneGrowthCounts + ThroneGrowthState in growth.rs (TDD)

Added after the `LibraryGrowthState` block in `src/models/growth.rs`:

- **`ThroneGrowthCounts`** — `team_members: u32` (roster axis, team_member table) + `team_wins: u32` (crown axis, post_game_learning wins). Derives `Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize`.
- **`ThroneGrowthState`** — `roster_tier: u8`, `progression_tier: u8`, `crown_tier: u8`. No `counts` field (key divergence from Team/Library analogs).
- **`from_parts(team: ThroneGrowthCounts, ranked: Option<RankedInfo>) -> Self`** — roster from `tier_for(team_members, [1,3,5])`, crown from `tier_for(team_wins, [1,5,15])`, progression independently from `tier_for((wins+losses).max(0), [1,20,60])`.
- **`impl Default for ThroneGrowthState`** — calls `from_parts(default, None)` → all-zero tiers.
- **Five `throne_*` unit tests** all green, including `throne_progression_grows_without_team` which locks the D-05/D-07 divergence.

TDD cycle: RED (tests added first, compile failed on missing types) → GREEN (types implemented, all 5 pass).

### Task 2: get_throne_growth_counts in db.rs

Added `pub async fn get_throne_growth_counts(db, team_id) -> DbResult<ThroneGrowthCounts>` after `get_library_growth_counts`:

- Strips `team:` prefix into owned `team_key: String` and binds via `.bind(("team_key", team_key))`.
- Single `.query()` with two `COUNT() GROUP ALL` statements:
  1. `FROM team_member WHERE team = type::record('team', $team_key)` → `team_members`
  2. `FROM post_game_learning WHERE team = type::record('team', $team_key) AND win_loss = 'win'` → `team_wins`
- Reuses the local `CountResult` + `count_of` helper pattern from `get_team_growth_counts`.
- Doc comment documents caller contract (pre-authorized team_id, V4 access control) and win literal.

## Verification

- `cargo test --features ssr --lib -- throne`: 5/5 tests pass (EXIT 0)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: EXIT 0
- `cargo check --features ssr`: EXIT 0
- All acceptance criteria confirmed via grep assertions

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — this is a pure data layer with no UI rendering.

## Threat Flags

None — `get_throne_growth_counts` follows the same access-control contract as the existing `get_team_growth_counts` (pre-authorized team_id, aggregate counts only, parameterized query). No new trust boundary introduced.

## Self-Check: PASSED

- `src/models/growth.rs` modified: confirmed (ThroneGrowthCounts, ThroneGrowthState, from_parts, Default, 5 tests present)
- `src/server/db.rs` modified: confirmed (get_throne_growth_counts present with correct field names and 'win' literal)
- Task 1 commit e0a22ef: confirmed
- Task 2 commit 1507916: confirmed
