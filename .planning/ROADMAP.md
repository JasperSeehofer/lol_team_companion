# Roadmap: LoL Team Companion

## Milestones

- ✅ **v1.0 Cross-Feature Intelligence** — Phases 1-5 (shipped 2026-03-18)
- ✅ **v1.1 Polish, Draft & Opponents Rework** — Phases 6-11 (shipped 2026-03-24)
- 🚧 **v1.2 Solo Mode & Match Intelligence** — Phases 12-16 (in progress)

## Phases

<details>
<summary>✅ v1.0 Cross-Feature Intelligence (Phases 1-5) — SHIPPED 2026-03-18</summary>

- [x] Phase 1: Pipeline CTAs (3/3 plans) — completed 2026-03-14
- [x] Phase 1.1: Playwright Bug Audit (3/3 plans) — completed 2026-03-14 (INSERTED)
- [x] Phase 2: Aggregation Layer (3/3 plans) — completed 2026-03-15
- [x] Phase 3: Smart Dashboard (2/2 plans) — completed 2026-03-15
- [x] Phase 4: Inline Intel (3/3 plans) — completed 2026-03-17
- [x] Phase 5: Post-Game Loop + Polish (7/7 plans) — completed 2026-03-18

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

<details>
<summary>✅ v1.1 Polish, Draft & Opponents Rework (Phases 6-11) — SHIPPED 2026-03-24</summary>

- [x] Phase 6: Bug Fixes (3/3 plans) — completed 2026-03-22
- [x] Phase 7: UX Polish (2/2 plans) — completed 2026-03-22
- [x] Phase 8: Champion Pool & Role UI (2/2 plans) — completed 2026-03-23
- [x] Phase 9: Draft & Game Plan (3/3 plans) — completed 2026-03-23
- [x] Phase 10: Opponents Redesign (3/3 plans) — completed 2026-03-23
- [x] Phase 11: Cross-Feature & Testing (4/4 plans) — completed 2026-03-24

See: `.planning/milestones/v1.1-ROADMAP.md` for full details.

</details>

### 🚧 v1.2 Solo Mode & Match Intelligence (In Progress)

**Milestone Goal:** Open the app to individual players — solo queue tracking, personal goals, and learnings — plus richer match detail from the Riot API that connects back into champion pool and draft decisions.

- [ ] **Phase 12: Solo Mode Infrastructure** - Mode toggle, region selector, solo sync, ranked display, solo dashboard shell
- [ ] **Phase 13: Match Detail View** - Full 10-player scoreboard, timeline events, performance breakdown, on-demand lazy fetch with DB cache
- [ ] **Phase 14: Personal Learnings Journal** - Personal journal CRUD, one-click creation from match detail, champion/matchup/tag filtering
- [ ] **Phase 15: Goals & LP History** - LP trend graph, rank goal setting, goal auto-tracking against match history, champion performance trends
- [ ] **Phase 16: Draft Integration** - Personal matchup notes in draft Intel sidebar, champion win rate badges on pool cards

## Phase Details

### Phase 12: Solo Mode Infrastructure
**Goal**: Solo players can use the app without a team — they can toggle into solo mode, link their Riot account with a region, sync their solo queue match history, and see a personal dashboard with their current rank
**Depends on**: Phase 11
**Requirements**: SOLO-01, SOLO-02, SOLO-03, SOLO-04, SOLO-05, RANK-01, RANK-03
**Success Criteria** (what must be TRUE):
  1. User can toggle between team mode and solo mode; the selected mode persists after navigating away and refreshing the page
  2. User can select their region (EUW, NA, KR, etc.) when linking a Riot account on the profile page
  3. User without a team can sync their solo queue match history and see matches appear in the stats page
  4. User can filter match history by queue type (solo queue, flex, all)
  5. User sees a personal solo dashboard at a dedicated route showing recent matches, current ranked badge, and goal status placeholders
  6. User's current ranked tier, division, and LP are displayed; a ranked snapshot is recorded in the DB on each sync
**Plans**: 3 plans
Plans:
- [ ] 12-01-PLAN.md — Schema, models, riot.rs region parameterization, ranked data infrastructure
- [ ] 12-02-PLAN.md — Mode toggle in nav, auth redirects, team-only page gates, region dropdown
- [ ] 12-03-PLAN.md — Solo dashboard page, ranked badge, queue filter, match sync, goal placeholders
**UI hint**: yes

### Phase 13: Match Detail View
**Goal**: Users can open any match from their history and see the full 10-player scoreboard with items, damage, gold, and vision — plus timeline events — without that data being fetched during the initial sync
**Depends on**: Phase 12
**Requirements**: MATCH-01, MATCH-02, MATCH-03, MATCH-04
**Success Criteria** (what must be TRUE):
  1. User can click any match in the history list and see all 10 participants with KDA, items, damage dealt, gold earned, and vision score
  2. User can view match timeline events (dragon kills, baron kills, tower kills, champion kills) for a given match
  3. User can see their own performance breakdown within the match — damage share percentage, vision score, and CS compared to game average
  4. Match detail data loads on-demand the first time a user opens a match, then loads instantly from the DB cache on subsequent opens without another Riot API call
**Plans**: TBD
**UI hint**: yes

### Phase 14: Personal Learnings Journal
**Goal**: Users can write personal post-game reflections tagged by champion and matchup, browse and filter their history, and start a new learning from a match detail with a single click
**Depends on**: Phase 13
**Requirements**: LEARN-01, LEARN-02, LEARN-03
**Success Criteria** (what must be TRUE):
  1. User can create a personal learning entry with free-text notes, tagged by champion played and opponent matchup
  2. User can click a "Add Learning" button on a match detail view and land on the learnings form with champion and matchup pre-filled from that match
  3. User can browse all their personal learnings and filter the list by champion, matchup, or tag
  4. Personal learnings are stored separately from team post-game reviews and are only visible to the user who created them
**Plans**: TBD
**UI hint**: yes

### Phase 15: Goals & LP History
**Goal**: Users can set personal improvement goals (rank target, CS/min, death limit), see their LP gain/loss over time as a graph, and view champion performance trends that auto-update from match history
**Depends on**: Phase 12
**Requirements**: RANK-02, LEARN-04, LEARN-05, LEARN-06
**Success Criteria** (what must be TRUE):
  1. User can view a line graph of LP history over time, derived from ranked snapshots captured at each sync
  2. User can set goals including a rank target tier, CS/min threshold, and per-game death limit
  3. User can see goal progress on the solo dashboard — each goal shows current average vs. the target derived from recent match history
  4. User can view champion performance trends showing win rate, KDA, and CS/min per champion over a selectable time window
**Plans**: TBD
**UI hint**: yes

### Phase 16: Draft Integration
**Goal**: Personal game experience feeds back into draft decisions — matchup notes surface in the draft Intel sidebar and solo match history win rates appear as badges on champion pool cards
**Depends on**: Phase 14, Phase 15
**Requirements**: XFEAT-03, XFEAT-04
**Success Criteria** (what must be TRUE):
  1. When a champion is highlighted in the draft Intel sidebar, any personal matchup notes for that champion are visible in the sidebar panel
  2. Champion pool cards display a win rate badge sourced from solo match history when the user has played that champion in recent matches
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 12 → 13 → 14 → 15 → 16

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Pipeline CTAs | v1.0 | 3/3 | Complete | 2026-03-14 |
| 1.1 Playwright Bug Audit | v1.0 | 3/3 | Complete | 2026-03-14 |
| 2. Aggregation Layer | v1.0 | 3/3 | Complete | 2026-03-15 |
| 3. Smart Dashboard | v1.0 | 2/2 | Complete | 2026-03-15 |
| 4. Inline Intel | v1.0 | 3/3 | Complete | 2026-03-17 |
| 5. Post-Game Loop + Polish | v1.0 | 7/7 | Complete | 2026-03-18 |
| 6. Bug Fixes | v1.1 | 3/3 | Complete | 2026-03-22 |
| 7. UX Polish | v1.1 | 2/2 | Complete | 2026-03-22 |
| 8. Champion Pool & Role UI | v1.1 | 2/2 | Complete | 2026-03-23 |
| 9. Draft & Game Plan | v1.1 | 3/3 | Complete | 2026-03-23 |
| 10. Opponents Redesign | v1.1 | 3/3 | Complete | 2026-03-23 |
| 11. Cross-Feature & Testing | v1.1 | 4/4 | Complete | 2026-03-24 |
| 12. Solo Mode Infrastructure | v1.2 | 0/3 | In progress | - |
| 13. Match Detail View | v1.2 | 0/? | Not started | - |
| 14. Personal Learnings Journal | v1.2 | 0/? | Not started | - |
| 15. Goals & LP History | v1.2 | 0/? | Not started | - |
| 16. Draft Integration | v1.2 | 0/? | Not started | - |
