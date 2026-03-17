# Roadmap: LoL Team Companion — Cross-Feature Intelligence

## Overview

This milestone transforms the app from a collection of siloed features into one connected tool. The build order follows a strict dependency chain: first wire the navigational connections between features, then add the aggregation queries those connections enable, then surface intelligence on the dashboard, then bring contextual intel inline on feature pages, and finally close the post-game loop with automation and apply polish app-wide. Each phase delivers a coherent, verifiable capability that the next phase depends on.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Pipeline CTAs** - Wire draft → game plan navigation with FK pre-fill (completed 2026-03-14)
- [x] **Phase 2: Aggregation Layer** - Add cross-table intelligence queries to db.rs (completed 2026-03-15)
- [x] **Phase 3: Smart Dashboard** - Surface prep priorities on team dashboard (completed 2026-03-15)
- [ ] **Phase 4: Inline Intel** - Opponent sidebar, pool warnings, and win condition tracker on feature pages (GAP CLOSURE)
- [ ] **Phase 5: Post-Game Loop + Polish** - Auto-generate action items from reviews; consistent empty states, skeletons, and mutation feedback app-wide (GAP CLOSURE)

## Phase Details

### Phase 1: Pipeline CTAs
**Goal**: Users can navigate the draft → game plan → post-game prep loop without copy-pasting IDs
**Depends on**: Nothing (first phase)
**Requirements**: PIPE-01
**Success Criteria** (what must be TRUE):
  1. User can click "Create game plan" on a completed draft and arrive at the game plan page with picks, bans, and side already filled in
  2. User can click "Write post-game review" from a game plan page and arrive at the post-game page pre-linked to that plan
  3. Game plan created from a draft retains a visible reference to the source draft
  4. Navigating to `/game-plan?draft_id=X` directly (e.g. bookmarked) correctly pre-fills from that draft
**Plans:** 3/3 plans complete

Plans:
- [ ] 01-01-PLAN.md — DB queries for draft prefill and game plan lookup
- [ ] 01-02-PLAN.md — Game plan page prefill, champion lock, badges, link dropdown
- [ ] 01-03-PLAN.md — Draft page CTAs, post-game badges, pipeline wiring

### Phase 01.1: Playwright bug audit — identify and document existing bugs across all features (INSERTED)

**Goal:** Systematically test all app features with Playwright e2e tests to identify and document existing bugs
**Requirements**: None (inserted testing phase)
**Depends on:** Phase 1
**Plans:** 3/3 plans complete

Plans:
- [ ] 01.1-01-PLAN.md — Fixtures extension + draft and tree-drafter audit (known bugs)
- [ ] 01.1-02-PLAN.md — Game plan, post-game, champion pool, team, and misc pages audit
- [ ] 01.1-03-PLAN.md — Full suite run, BUG-REPORT.md compilation, user review

### Phase 2: Aggregation Layer
**Goal**: Cross-table intelligence queries exist and are validated, ready to power UI panels
**Depends on**: Phase 1
**Requirements**: (infrastructure phase — no direct v1 requirements; enables INTL-01, PIPE-03, PIPE-04, INTL-02, PIPE-02)
**Success Criteria** (what must be TRUE):
  1. `get_dashboard_summary(team_id)` returns open action items, recent post-game summaries, and champion pool gaps in a single batched query
  2. `get_champion_performance_summary(team_id)` returns per-champion win rate data aggregated in Rust (no SurrealDB GROUP BY views)
  3. Champion name normalization function is applied at all ingestion points so cross-feature joins on champion names return correct results
  4. All new queries return `Ok(empty)` rather than `Err` when the team has no data
  5. Integration tests cover each new query function
**Plans:** 3 plans

Plans:
- [ ] 02-01-PLAN.md — Normalization utility, model structs, autocomplete canonical ID fix
- [ ] 02-02-PLAN.md — Dashboard summary query, champion performance queries, pool gap analysis, migration
- [ ] 02-03-PLAN.md — Page-level autocomplete enforcement, startup migration wiring

**Note:** Phase 2 has no direct v1 requirement assigned because it is a pure infrastructure phase whose outputs are consumed by Phases 3, 4, and 5. Skipping it is not possible — the upstream v1 requirements (INTL-01, PIPE-03, PIPE-04, INTL-02, PIPE-02) all depend on these queries.

### Phase 3: Smart Dashboard
**Goal**: The team dashboard surfaces what matters — prep priorities, open action items, pool gaps — at a glance
**Depends on**: Phase 2
**Requirements**: INTL-01
**Success Criteria** (what must be TRUE):
  1. Dashboard shows open action items panel with count and links to the action items page
  2. Dashboard shows recent post-game summaries with patterns identified in the last N reviews
  3. Dashboard shows champion pool gap warnings when a player has no coverage for a lane appearing in recent drafts
  4. Each dashboard panel loads independently (one slow query does not block others from rendering)
  5. New teams with no data see empty states with contextual guidance CTAs, not blank panels
**Plans:** 2/2 plans complete

Plans:
- [ ] 03-01-PLAN.md — Server functions for post-game and pool-gap panels, fix action items resource bug, add all three panel UIs
- [ ] 03-02-PLAN.md — E2e tests for dashboard panels, visual verification checkpoint

### Phase 4: Inline Intel
**Goal**: Draft planner and game plan pages surface contextual intelligence without disrupting existing workflows
**Depends on**: Phase 2
**Requirements**: PIPE-03, PIPE-04, INTL-02
**Success Criteria** (what must be TRUE):
  1. Draft planner shows a warning badge on a pick slot when the assigned player has no matching champion in their pool
  2. Draft planner shows an opponent tendency sidebar listing historical picks and bans from scouting data for the current opponent
  3. Win condition tracker page shows each declared win condition alongside a pass/fail record across recent post-game reviews
  4. Intel panels in the draft planner do not trigger reactive refetches on every champion pick — only on explicit user actions
**Plans:** 2/3 plans executed

Plans:
- [ ] 04-01-PLAN.md — Pool warning badges on draft pick slots (DraftBoard prop + client-side Memo computation)
- [ ] 04-02-PLAN.md — Opponent tendency sidebar enhancement (frequency counts, OTP detection, Riot API mastery)
- [ ] 04-03-PLAN.md — Win condition tracker panel on game plan page (historical stats + opponent filter)

### Phase 5: Post-Game Loop + Polish
**Goal**: Post-game reviews automatically create action items, and every page in the app has consistent empty states, loading skeletons, and mutation feedback
**Depends on**: Phase 4
**Requirements**: PIPE-02, UX-01, UX-02, UX-03
**Success Criteria** (what must be TRUE):
  1. Saving a post-game review with tagged patterns automatically creates action items; user sees "N new action items created" with a link
  2. Every page with no data shows a meaningful empty state with a contextual CTA (not a blank area or generic message)
  3. Every data-fetching page shows skeleton screens while loading (not blank space or a spinner)
  4. Every save, delete, and update action shows a visible success or error message within the same page context
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 1.1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Pipeline CTAs | 3/3 | Complete   | 2026-03-14 |
| 1.1 Playwright Bug Audit | 3/3 | Complete   | 2026-03-14 |
| 2. Aggregation Layer | 3/3 | Complete   | 2026-03-15 |
| 3. Smart Dashboard | 2/2 | Complete   | 2026-03-15 |
| 4. Inline Intel | 2/3 | In Progress|  |
| 5. Post-Game Loop + Polish | 0/TBD | Gap closure | - |
