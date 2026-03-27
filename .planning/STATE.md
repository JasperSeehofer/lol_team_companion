---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Solo Mode & Match Intelligence
status: Ready to execute
stopped_at: Completed 14-01-PLAN.md
last_updated: "2026-03-27T16:32:19.994Z"
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 9
  completed_plans: 7
---

## Current Position

Phase: 14 (personal-learnings-journal) — EXECUTING
Plan: 2 of 3

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25)

**Core value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.
**Current focus:** Phase 14 — personal-learnings-journal

## Decisions

- [Roadmap]: Phase 12 must establish `synced_by` on `match` table and `mode` on `user` table before any other solo work — data model correctness gates all subsequent phases
- [Roadmap]: Match timeline fetch is on-demand + DB-cached only (never eager during sync) — rate limit protection
- [Roadmap]: `personal_learning` is a separate table from `post_game_learning` — solo learnings must not mix with team-scoped records
- [Roadmap]: Mode toggle persisted in DB user record, not WASM signal — survives hard navigation on login/logout
- [Roadmap]: Phase 15 (Goals & LP History) depends on Phase 12 only, not Phase 13 or 14 — can be sequenced after either
- [Phase 12]: EUW fallback for opponent Riot API calls: opponent players have no associated user region, hardcode EUW as safe fallback for get_puuid/fetch_player_intel/fetch_player_champions
- [Phase 12]: store_matches_with_synced_by wrapper: preserved backward-compatible store_matches() API; solo sync calls store_matches_with_synced_by directly with user ID
- [Phase 12]: PlayerMatchStats lacks queue_id/game_duration: match rows show champion+KDA+CS only; queue filter works via DB WHERE clause
- [Phase 13-match-detail-view]: Store participants as single JSON string in match_detail table — avoids 10-row JOIN with no query benefits
- [Phase 13-match-detail-view]: Timeline event markers use index into full timeline_events vec as stable key to avoid index drift when filters toggle
- [Phase 14-personal-learnings-journal]: personal_learning is user-scoped (not team-scoped) — matches D-01 design decision; created_at deserialized as Option<String> consistent with other models

## Blockers/Concerns

- [Phase 12]: Region support scope — decide EUW-only vs. multi-region before writing solo sync; retrofitting touches every Riot API call
- [Phase 15]: LP delta boundary logic across tier/division changes (promos, relegation, Master+ no-division) needs validation before implementing LP graph

## Session Continuity

Last session: 2026-03-27T16:32:19.993Z
Stopped at: Completed 14-01-PLAN.md
Resume file: None
