# Roadmap: LoL Team Companion

## Milestones

- ✅ **v1.0 Cross-Feature Intelligence** — Phases 1-5 (shipped 2026-03-18)
- 🚧 **v1.1 Polish, Draft & Opponents Rework** — Phases 6-11 (in progress)

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

### 🚧 v1.1 Polish, Draft & Opponents Rework (In Progress)

**Milestone Goal:** Fix known bugs, elevate UX consistency, overhaul champion pool and draft tooling, redesign opponent scouting, and ship cross-feature intelligence with real test infrastructure.

## Phase Details

- [x] **Phase 6: Bug Fixes** - Eliminate known regressions and configure Playwright CLI so the app is stable and verifiable before adding new surface (completed 2026-03-22)
- [x] **Phase 7: UX Polish** - Small, independent UX improvements that make the app feel polished (completed 2026-03-22)
- [x] **Phase 8: Champion Pool & Role UI** - Overhaul champion pool management and add role assignment to draft picks (completed 2026-03-23)
- [x] **Phase 9: Draft & Game Plan** - Improve draft tooling with opponent context, pool notes sidebar, and game plan autofill (completed 2026-03-23)
- [x] **Phase 10: Opponents Redesign** - Rebuild opponent scouting with 5-role form, Riot API fetch, and pool analysis (completed 2026-03-23)
- [x] **Phase 11: Cross-Feature & Testing** - Plan effectiveness dashboard, draft-integrated notes, and real seeded test data (completed 2026-03-24)

### Phase 6: Bug Fixes
**Goal**: The app works without known regressions and Playwright CLI is configured for browser verification — tree drafting, game plan navigation, roster display, hydration, and drag-and-drop all behave correctly, and agents can run browser checks from Phase 6 onward
**Depends on**: Nothing (first v1.1 phase)
**Requirements**: BUG-01, BUG-02, BUG-03, BUG-04, BUG-05, PLAN-02, TEST-02
**Success Criteria** (what must be TRUE):
  1. User can branch from any tree drafter node and continue drafting without UI freezing
  2. Clicking "view game plan" from a draft opens the correct game plan with its data populated
  3. Team leader is always visible in the roster list regardless of member ordering
  4. No hydration mismatch warnings appear in the browser console on any page
  5. Dragging a champion pick/ban moves it to the target slot and clears the source slot
  6. Playwright CLI workflow is configured with auth fixture and page navigation helpers so agents can run browser verification without manual setup
**Plans**: 3 plans
Plans:
- [x] 06-01-PLAN.md — Test infrastructure: shared helpers, regression scaffold, agent-browser setup
- [x] 06-02-PLAN.md — Bug fixes: BUG-01 through BUG-05 + PLAN-02 with regression tests
- [x] 06-03-PLAN.md — Audit spec refactor + full suite verification

### Phase 7: UX Polish
**Goal**: Small, independent UX improvements are in place — toasts, timestamps, profile UI, team search, and role indicators all behave as expected
**Depends on**: Phase 6
**Requirements**: UX-04, UX-05, UX-06, UX-07, UX-09
**Success Criteria** (what must be TRUE):
  1. Toast notifications appear below the nav header without overlapping it
  2. All timestamps in the app display in human-readable format (e.g., "Mar 19, 2026 14:30") with no decimal seconds or "Z" suffix
  3. Profile page shows a single "Link Account" button instead of two
  4. Team join page has a search bar with autocomplete suggestions instead of a full team list
  5. Roster cards display role icons (top/jg/mid/bot/sup) as visual background indicators
**Plans**: 2 plans
Plans:
- [x] 07-01-PLAN.md — Toast positioning, timestamp formatter with unit tests, profile button dedup
- [x] 07-02-PLAN.md — Team join search with member count, roster role watermark icons

### Phase 8: Champion Pool & Role UI
**Goal**: Champion pool management is substantially improved with larger icons, drag-and-drop between tiers, and reworked matchup entry; draft picks have role assignment via clickable role icons
**Depends on**: Phase 6
**Requirements**: UX-08, UX-10
**Success Criteria** (what must be TRUE):
  1. Champion pool page shows larger champion icons and allows drag-and-drop reordering between tier buckets
  2. Adding a matchup in champion pool uses a searchable champion list for the opponent and has no redundant type selection
  3. Each picked champion in the draft has a clickable role icon that lets the user assign their lane
  4. Draft role icons default to an auto-guess based on champion class (e.g., marksman defaults to bot)
**Plans**: 2 plans
Plans:
- [x] 08-01-PLAN.md — Champion pool card grid, drag-and-drop between tiers, matchup autocomplete
- [x] 08-02-PLAN.md — Draft role badges with auto-guess, popover selection, DraftAction model extension

### Phase 9: Draft & Game Plan
**Goal**: Draft page connects to opponent context, surfaces pool notes inline, and game plan creation auto-fills from the source draft
**Depends on**: Phase 8
**Requirements**: DRFT-01, DRFT-02, DRFT-03, DRFT-04, DRFT-05, PLAN-01, PLAN-03
**Success Criteria** (what must be TRUE):
  1. User can select a scouted opponent from a dropdown on the draft page, or add a new opponent inline
  2. The draft intel panel has a link to the opponents page that auto-saves the draft before navigating away
  3. Banned champions are excluded from champion selection in the intel matchup panel
  4. User can navigate back from champion detail view to the champion selection list via a back button
  5. When a pooled champion is picked in a draft, a sidebar shows that champion's pool notes (matchups, power spikes, synergies)
  6. Creating a game plan from a draft pre-fills the strategy tag from the draft's tags, names it "<draft name> plan", and populates our champions and enemy champions
**Plans**: 3 plans
Plans:
- [x] 09-01-PLAN.md — Opponent autocomplete dropdown, ban filtering, back button in matchup detail
- [x] 09-02-PLAN.md — Notes tab in intel sidebar with champion sub-tabs and pool notes
- [x] 09-03-PLAN.md — Game plan autofill: name, strategy tag, per-role strategy from draft

### Phase 10: Opponents Redesign
**Goal**: Opponent profiles are rebuilt with a structured 5-role form, auto-fetched Riot data, OTP detection, mastery badges, and pool analysis
**Depends on**: Phase 9
**Requirements**: OPP-01, OPP-02, OPP-03, OPP-04
**Success Criteria** (what must be TRUE):
  1. New opponent form accepts a team name and five Riot ID fields (one per role); saving auto-fetches champion data from the Riot API
  2. Opponent player cards show an OTP warning badge when more than 60% of their recent games are on a single champion
  3. Opponent player cards display champion mastery level badges and a data recency indicator showing when data was last fetched
  4. Each opponent player has a pool analysis section showing pool size, role flexibility score, and identified comfort picks
**Plans**: 3 plans
Plans:
- [x] 10-01-PLAN.md — Schema extension, model enrichment, compute_intel with unit tests, batch DB functions
- [x] 10-02-PLAN.md — Combined Riot API fetch function, new server functions for creation and intel fetch
- [x] 10-03-PLAN.md — Full UI redesign: 5-role form, enriched player cards, pool analysis, e2e test

### Phase 11: Cross-Feature & Testing
**Goal**: Plan effectiveness is visible in the UI, champion pool notes appear in the draft sidebar, and the app has real seeded test data
**Depends on**: Phase 10
**Requirements**: XFEAT-01, XFEAT-02, TEST-01
**Success Criteria** (what must be TRUE):
  1. A plan effectiveness view aggregates post-game review outcomes per game plan and shows each strategy's success rate
  2. Draft sidebar shows champion pool notes (matchup tips, power spikes) contextually when a pooled champion is selected
  3. All legacy test accounts are removed; 2 test teams exist with 5 Riot-linked accounts each, populated with champion pools, notes, and example data
**Plans**: 4 plans
Plans:
- [x] 11-01-PLAN.md — Model extension: PostGameLearning win_loss + rating fields, schema, DB layer, analytics query
- [x] 11-02-PLAN.md — Post-game form: win/loss selector, star rating, auto-detect outcome from Riot API
- [x] 11-03-PLAN.md — Analytics page: strategy tag cards, sortable table, accordion, route + nav link
- [x] 11-04-PLAN.md — Seed binary: 2 teams, 10 users, full demo data with champion pools, drafts, plans, reviews

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Pipeline CTAs | v1.0 | 3/3 | Complete | 2026-03-14 |
| 1.1 Playwright Bug Audit | v1.0 | 3/3 | Complete | 2026-03-14 |
| 2. Aggregation Layer | v1.0 | 3/3 | Complete | 2026-03-15 |
| 3. Smart Dashboard | v1.0 | 2/2 | Complete | 2026-03-15 |
| 4. Inline Intel | v1.0 | 3/3 | Complete | 2026-03-17 |
| 5. Post-Game Loop + Polish | v1.0 | 7/7 | Complete | 2026-03-18 |
| 6. Bug Fixes | v1.1 | 3/3 | Complete   | 2026-03-22 |
| 7. UX Polish | v1.1 | 2/2 | Complete   | 2026-03-22 |
| 8. Champion Pool & Role UI | v1.1 | 2/2 | Complete   | 2026-03-23 |
| 9. Draft & Game Plan | v1.1 | 3/3 | Complete   | 2026-03-23 |
| 10. Opponents Redesign | v1.1 | 3/3 | Complete    | 2026-03-23 |
| 11. Cross-Feature & Testing | v1.1 | 4/4 | Complete    | 2026-03-24 |
