---
phase: 02-aggregation-layer
verified: 2026-03-15T08:00:00Z
status: gaps_found
score: 4/5 success criteria verified
re_verification: false
gaps:
  - truth: "Integration tests cover each new query function"
    status: failed
    reason: >
      The ROADMAP success criterion explicitly requires integration tests for each new
      query function. The tests/db_* integration suite has no file or test covering
      get_dashboard_summary, get_champion_performance_summary, get_team_champion_performance,
      compute_pool_gaps_for_team, or migrate_champion_names. The existing unit tests in
      db.rs cover only the pure-Rust helpers (aggregate_champion_performance,
      compute_pool_gaps, test_dashboard_summary_assembly), not the async DB-calling
      functions. No new integration test file was added to tests/.
    artifacts:
      - path: "tests/"
        issue: "No db_aggregation.rs or equivalent integration test file exists"
    missing:
      - "Create tests/db_aggregation.rs with at minimum: test_get_dashboard_summary_empty_team (verify Ok(DashboardSummary::default()) returned), test_get_champion_performance_empty (verify Ok(Vec::new()) returned), test_get_team_champion_performance_empty (verify Ok(Vec::new()) returned)"
      - "Optional but recommended: test_get_dashboard_summary_with_data (create team, action items, post-game records; assert counts returned)"
---

# Phase 2: Aggregation Layer Verification Report

**Phase Goal:** Build champion name normalization layer, cross-table aggregation queries, and migration tooling so downstream phases (dashboard, action items, pipeline) can query unified champion data.
**Verified:** 2026-03-15T08:00:00Z
**Status:** gaps_found — 4/5 success criteria verified; 1 gap (missing integration tests)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | `get_dashboard_summary(team_id)` returns open action items, recent post-game summaries, and champion pool gaps in a single batched query | VERIFIED | `db.rs:3490` — 5-statement batched query; `db.rs:3572` pool gaps via `compute_pool_gaps_for_team`; returns `Ok(DashboardSummary::default())` when team is empty |
| 2 | `get_champion_performance_summary(team_id)` returns per-champion win rate data aggregated in Rust (no SurrealDB GROUP BY views) | VERIFIED | `db.rs:3742` per-player, `db.rs:3897` team-wide; aggregation done in `aggregate_champion_performance` (pure Rust, `db.rs:3686`); both return `Ok(Vec::new())` on empty team |
| 3 | Champion name normalization function is applied at all ingestion points so cross-feature joins on champion names return correct results | VERIFIED | `normalize_champion_name` in `data_dragon.rs:13` (3-pass lookup); `ChampionAutocomplete` stores `champ.id` on selection (`champion_autocomplete.rs:31`); `champion_picker.rs:114,130` drag data and `is_used` use `champion.id`; `tree_drafter.rs:591` `fill_slot` stores `champ.id`; `champion_pool.rs` and `game_plan.rs` display human names via `c.id == champ` lookup |
| 4 | All new queries return `Ok(empty)` rather than `Err` when the team has no data | VERIFIED | `get_champion_performance_summary` early-returns `Ok(Vec::new())` when `team_id.is_empty()` (`db.rs:3751-3753`); `get_team_champion_performance` returns `Ok(Vec::new())` when no team members (`db.rs:3927-3929`); `get_dashboard_summary` uses `unwrap_or_default()` on all `.take()` calls |
| 5 | Integration tests cover each new query function | FAILED | No integration test file for the aggregation layer exists. `tests/` contains `db_champion_pool.rs`, `db_drafts.rs`, `db_game_plan_pipeline.rs`, `db_teams.rs`, `db_tree.rs`, `db_users.rs` — none of these test `get_dashboard_summary`, `get_champion_performance_summary`, `get_team_champion_performance`, `migrate_champion_names`, or `compute_pool_gaps_for_team` |

**Score:** 4/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|---------|--------|---------|
| `src/server/data_dragon.rs` | `normalize_champion_name` utility (3-pass: exact ID, case-insensitive display name, stripped fuzzy) | VERIFIED | Line 13; `#[cfg(feature = "ssr")]`; unit tests pass |
| `src/models/game_plan.rs` | `DashboardSummary`, `ActionItemPreview`, `PostGamePreview`, `PoolGapWarning`, `ChampionPerformanceSummary` structs | VERIFIED | Lines 63-110; `Default` derives on `DashboardSummary` and `ChampionPerformanceSummary`; serde round-trip unit tests pass |
| `src/components/champion_autocomplete.rs` | Dropdown selection stores canonical ID in `value` signal, human name in `filter_text` | VERIFIED | Line 31: `value.set(champ.id.clone())`, line 32: `set_filter_text.set(champ.name.clone())`; external sync Effect at line 39 |
| `src/server/db.rs` | `migrate_champion_names`, `compute_pool_gaps`, `get_dashboard_summary`, `aggregate_champion_performance`, `get_champion_performance_summary`, `get_team_champion_performance` | VERIFIED | All 6 functions present at lines 3260, 3392, 3490, 3686, 3742, 3897 |
| `src/server/db.rs` | Unit tests for aggregation logic | VERIFIED | 8 unit tests at lines 4078-4273: pool gap dominance, opponent escalation, balanced pool, dashboard assembly, dashboard default, aggregate performance (populated + empty + sorted) |
| `src/components/champion_picker.rs` | `is_used` check and drag data use `champion.id` | VERIFIED | Line 114: `used_champions.contains(&champion.id)`; line 130: `dt.set_data("text/plain", &champ_for_drag.id)` |
| `src/pages/tree_drafter.rs` | `fill_slot` and `champion_map` use `champ.id` | VERIFIED | Line 591: `fill_slot(slot, champ.id)`; lines 925, 1284, 1488: `champion_map` keyed by `c.id` |
| `src/pages/champion_pool.rs` | Image/name lookup by `c.id`; display name shown via canonical ID resolution | VERIFIED | Lines 546-577: lookup by `c.id == champ`, shows `display_name` from canonical ID |
| `src/pages/game_plan.rs` | Locked champion display and matchup labels resolve IDs to display names | VERIFIED | Lines 1129, 1300, 1304: `.find(|c| c.id == val/o/t)` for display name resolution |
| `src/server/db.rs` (init_db) | `migrate_champion_names` called after schema init, non-fatally | VERIFIED | Lines 116-119: `if let Err(e) = migrate_champion_names(&db).await { tracing::warn!(...) }` inside `init_db` |
| `tests/db_aggregation.rs` | Integration tests for each new async query function | MISSING | File does not exist; no integration test file covers the aggregation layer |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `champion_autocomplete.rs` | `models/champion.rs` | `champ.id` (canonical) stored in value signal | WIRED | `select_champion(champ: Champion)` uses `champ.id` and `champ.name` |
| `db.rs` | `models/game_plan.rs` | `DashboardSummary`, `ChampionPerformanceSummary`, `PoolGapWarning` return types | WIRED | All model types imported and used as return types in aggregation functions |
| `db.rs` | `data_dragon.rs` | `fetch_champions` for class tags; `normalize_champion_name` for migration | WIRED | `db.rs:3261`: `use crate::server::data_dragon`; `db.rs:3264`: `data_dragon::fetch_champions()`; `db.rs:3281`: `data_dragon::normalize_champion_name()` |
| `main.rs` / `db.rs` (init_db) | `db.rs` (migrate_champion_names) | Called after schema init | WIRED | `db.rs:117`: `migrate_champion_names(&db).await` inside `init_db`; plan 03 correctly identified this was pre-wired by plan 02 |
| `pages/game_plan.rs` | `champion_autocomplete.rs` | `ChampionAutocomplete` component used for champion inputs | WIRED | Import at line 1; used at lines 1140 and 1162 |
| `pages/champion_pool.rs` | `champion_autocomplete.rs` | `ChampionAutocomplete` for champion entry | WIRED | Import at line 1; used at line 487 |
| `pages/tree_drafter.rs` | `champion_picker.rs` | `ChampionPicker` (grid-based) for draft slot champion selection | WIRED | `ChampionPicker` imported and used; canonical IDs via `champ.id` in `on_champion_select` callback |

### Requirements Coverage

This is an infrastructure phase with no direct v1 requirements. All three plans declare `requirements: []`. The phase enables INTL-01, PIPE-03, PIPE-04, INTL-02, PIPE-02 but does not fulfill them directly. No orphaned requirements exist — all named IDs are mapped to downstream phases (3, 4, 5) in REQUIREMENTS.md.

| Requirement | Phase | Status |
|-------------|-------|--------|
| INTL-01 | Phase 3 | Enabled (not claimed by Phase 2) |
| PIPE-03 | Phase 4 | Enabled (not claimed by Phase 2) |
| PIPE-04 | Phase 4 | Enabled (not claimed by Phase 2) |
| INTL-02 | Phase 4 | Enabled (not claimed by Phase 2) |
| PIPE-02 | Phase 5 | Enabled (not claimed by Phase 2) |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/server/db.rs` | ~3840-3894 | `post_game_champ_outcomes` always returns empty `Vec` — post-game win/loss per-champion is untracked because `post_game_learning` schema has no win/loss field | Info | `post_game_wins` and `post_game_losses` in `ChampionPerformanceSummary` will always be 0; documented in SUMMARY as known deviation |

No placeholder components, TODO/FIXME blockers, or stub implementations found. The empty post-game outcomes is a documented schema limitation, not a stub.

### Human Verification Required

None required. All phase 2 deliverables are server-side query functions, model structs, and component wiring that can be fully verified programmatically.

### Gaps Summary

One gap blocks the stated success criteria:

**Missing integration tests for DB-calling aggregation functions.** The ROADMAP success criterion #5 explicitly requires "Integration tests cover each new query function." The 8 unit tests in `db.rs` are thorough for the pure-Rust helpers (`aggregate_champion_performance`, `compute_pool_gaps`, dashboard assembly) but the async functions that actually execute SurrealDB queries (`get_dashboard_summary`, `get_champion_performance_summary`, `get_team_champion_performance`, `migrate_champion_names`) have no integration test coverage. This mirrors the pattern already established for other db functions (e.g. `tests/db_game_plan_pipeline.rs`, `tests/db_champion_pool.rs`).

The gap is low-risk for downstream phases since the query logic is substantively implemented and the pure-Rust aggregation helpers are unit-tested — but it does not satisfy the stated success criterion and leaves the async DB path untested.

The four other success criteria are fully satisfied with substantive, wired implementations verified against the actual codebase.

---

_Verified: 2026-03-15T08:00:00Z_
_Verifier: Claude (gsd-verifier)_
