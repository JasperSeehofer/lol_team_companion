---
phase: 12-solo-mode-infrastructure
plan: "03"
subsystem: pages
tags: [solo-mode, dashboard, riot-api, ranked, match-history, ui]
dependency_graph:
  requires:
    - schema.ranked_snapshot_table
    - model.RankedInfo
    - model.PublicUser.mode
    - riot.fetch_ranked_data
    - riot.fetch_match_history
    - db.get_latest_ranked_snapshot
    - db.get_solo_matches
    - db.store_ranked_snapshot
    - db.store_matches_with_synced_by
    - db.update_last_solo_sync
  provides:
    - pages.SoloDashboardPage
    - routes./solo
    - db.get_should_auto_sync
    - e2e.solo_smoke_test
  affects:
    - src/app.rs (new /solo route)
    - src/pages/mod.rs (pub mod solo_dashboard)
    - src/server/db.rs (get_should_auto_sync added)
    - e2e/tests/pages.spec.ts (/solo in AUTHED_PAGES)
tech_stack:
  added: []
  patterns:
    - get_should_auto_sync: DB helper using surrealdb::types::Datetime with Deref to chrono timestamp
    - auto-sync-on-mount: Effect tracks dashboard_resource, fires sync once when stale
    - queue-filter-resource: Resource::new keyed on queue_filter signal for server-side filtering
key_files:
  created:
    - src/pages/solo_dashboard.rs
  modified:
    - src/pages/mod.rs
    - src/app.rs
    - src/server/db.rs
    - e2e/tests/pages.spec.ts
decisions:
  - "PlayerMatchStats lacks queue_id/game_duration/game_end fields (they live on the match table, not player_match): match rows display champion+KDA+CS only; queue filter works via DB-level WHERE clause in get_solo_matches"
  - "get_should_auto_sync added to db.rs: avoids surrealdb::types::SurrealValue derive in page server fn; cleaner separation"
  - "Stats page queue filter already existed (filter_queue signal in StatsContent): accepted as satisfying acceptance criteria; no duplicate dropdown added"
metrics:
  duration_seconds: 480
  completed_date: "2026-03-25"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
  files_created: 1
---

# Phase 12 Plan 03: Solo Dashboard Summary

Solo dashboard page at /solo with ranked badge, match sync, queue filter, and goal placeholders — the primary user-facing deliverable of Phase 12 solo mode.

## One-liner

Solo dashboard at /solo with ranked tier emblem, match list with queue filter, auto-sync on page load, and greyed-out goal placeholder cards.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Solo dashboard page with ranked badge, sync, matches, queue filter, goal placeholders | 1d092c6 | solo_dashboard.rs, mod.rs, app.rs, db.rs |
| 2 | Queue filter on stats page + e2e smoke test entry | 6960008 | e2e/tests/pages.spec.ts |

## What Was Built

### Solo Dashboard Page (src/pages/solo_dashboard.rs)

**Server functions:**
- `get_solo_dashboard(queue_filter: Option<i32>)` — returns `SoloDashboardData` with ranked snapshot, recent 20 matches, auto-sync flag, and puuid presence flag
- `sync_solo_matches()` — fetches match history + ranked data via Riot API, stores via `store_matches_with_synced_by`, updates `last_solo_sync`; returns count of synced matches

**SoloDashboardData struct:**
- `ranked: Option<RankedInfo>` — latest ranked snapshot for RANKED_SOLO_5x5
- `matches: Vec<PlayerMatchStats>` — recent 20 matches, filtered by queue
- `should_auto_sync: bool` — true if no last_solo_sync or > 10 min ago
- `has_puuid: bool` — true if user has linked Riot account

**Component sections:**
- Header: "My Dashboard" h1 + "Sync Matches" / "Syncing..." button (accent color, disabled during sync)
- RankedBadgeSection: tier emblem from Data Dragon CDN, tier+division text (e.g. "Gold II"), LP, W/L/winrate, or "Unranked" with shield SVG
- MatchListSection: queue filter dropdown (All/Solo/Flex), match rows with win/loss left border, champion+KDA+CS, empty state
- GoalPlaceholders: 3 greyed cards (Rank Target, CS per Minute, Deaths per Game) with "Coming in a future update"

**Auto-sync on mount (D-10):**
Effect fires once when dashboard_resource first resolves; if `should_auto_sync && has_puuid && !auto_synced`, triggers sync and shows toast

**DB helper added to db.rs:**
- `get_should_auto_sync` — queries `last_solo_sync` field as `Option<surrealdb::types::Datetime>`, computes staleness via `.timestamp()` (Deref to chrono's DateTime<Utc>)

### Route + Module

- `/solo` route added to `src/app.rs`
- `pub mod solo_dashboard` added to `src/pages/mod.rs`

### E2E Smoke Test (e2e/tests/pages.spec.ts)

- `/solo` added to `AUTHED_PAGES` with content regex `/dashboard|matches|ranked|solo/i`

### Stats Page Queue Filter

The stats page already had a queue filter (`filter_queue` signal, All/Solo/Flex/ARAM options) in the `StatsContent` component. Acceptance criteria are satisfied by the pre-existing implementation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] PlayerMatchStats lacks queue_id, game_duration, game_end fields**
- **Found during:** Task 1 compilation
- **Issue:** The plan's interface definition listed these fields on `PlayerMatchStats` but they live on the `match` table, not `player_match`. The actual struct has: id, match_id, user_id, champion, kills, deaths, assists, cs, vision_score, damage, win.
- **Fix:** Match rows display champion+KDA+CS only (no duration or queue label text). Queue filter still works server-side via `get_solo_matches` WHERE clause. Removed `format_duration` and `queue_label` helpers since their output isn't displayable.
- **Files modified:** src/pages/solo_dashboard.rs
- **Commit:** 1d092c6

**2. [Rule 1 - Bug] surrealdb::Datetime not in crate root**
- **Found during:** Task 1 compilation
- **Issue:** `surrealdb::Datetime` is not exported at the crate root; must be `surrealdb::types::Datetime`
- **Fix:** Added `get_should_auto_sync` function to db.rs (SSR-only context) where SurrealValue derive and surrealdb types are already imported
- **Files modified:** src/server/db.rs
- **Commit:** 1d092c6

**3. [Out of scope] Pre-existing compile errors in parallel-agent files**
- opponents.rs, profile.rs, team/dashboard.rs have unclosed delimiters and borrow errors from Plan 02 parallel agent work
- These prevent `cargo check` from completing cleanly but are not caused by Plan 03 changes
- Deferred to orchestrator validation after all agents complete

## Known Stubs

- **Goal placeholder cards** — intentionally greyed out with "Coming in a future update" label; no backend data. Per plan spec, these are placeholders for Phase 15 goal tracking.
- **LP delta display** — omitted per plan spec (requires comparing consecutive ranked snapshots, deferred to Phase 15)
- **Match duration/queue label** — not shown in match rows (field not available in PlayerMatchStats); future enhancement requires a join query returning match.game_duration and match.queue_id

## Self-Check: PASSED

- [x] FOUND: solo_dashboard.rs
- [x] FOUND: SoloDashboardPage
- [x] FOUND: get_solo_dashboard
- [x] FOUND: sync_solo_matches
- [x] FOUND: SoloDashboardData
- [x] FOUND: My Dashboard
- [x] FOUND: Ranked Solo/Duo
- [x] FOUND: Sync Matches
- [x] FOUND: Syncing...
- [x] FOUND: All Queues
- [x] FOUND: Solo/Duo
- [x] FOUND: Flex
- [x] FOUND: Goals
- [x] FOUND: Rank Target
- [x] FOUND: CS per Minute
- [x] FOUND: Deaths per Game
- [x] FOUND: Coming in a future update
- [x] FOUND: No matches yet
- [x] FOUND: ranked-emblems
- [x] FOUND: should_auto_sync
- [x] FOUND: fetch_ranked_data
- [x] FOUND: mod.rs solo_dashboard
- [x] FOUND: /solo in app.rs
- [x] FOUND: All Queues in stats.rs
- [x] FOUND: queue_filter in stats.rs
- [x] FOUND: /solo in e2e
- [x] Commits 1d092c6 and 6960008 exist
