# Requirements: LoL Team Companion

**Defined:** 2026-03-19
**Core Value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.

## v1.1 Requirements

Requirements for v1.1 Polish, Draft & Opponents Rework. Each maps to roadmap phases.

### Bug Fixes

- [ ] **BUG-01**: Tree drafter "Branch from here" makes UI unresponsive — user can branch from any node and continue drafting
- [ ] **BUG-02**: "View game plan" from draft opens empty game plan — clicking navigates to the correct populated game plan
- [ ] **BUG-03**: Team leader disappears from roster under certain conditions — team leader always visible in roster
- [ ] **BUG-04**: Nav resource read outside Suspense causes hydration warning — no hydration mismatch warnings in console
- [ ] **BUG-05**: Draft drag-and-drop copies icon link instead of moving pick/ban — dragging a champion moves it to the target slot and empties the source

### UX Polish

- [ ] **UX-04**: Toast notifications render below the header section, not overlapping it
- [ ] **UX-05**: Timestamps display human-readable format without decimal digits or "Z" suffix (e.g. "Mar 19, 2026 14:30")
- [ ] **UX-06**: Profile page shows a single "Link Account" button, not two
- [ ] **UX-07**: Team join uses a search bar with autocomplete/suggested results instead of listing all teams
- [ ] **UX-08**: Champion pool has larger champion icons, drag-and-drop between tiers, and reworked matchup entry (no redundant type selection, opponent champion uses searchable champion list)
- [ ] **UX-09**: Team roster cards display role icons (top/jg/mid/bot/sup) as visual background indicators
- [ ] **UX-10**: Draft pick cards contain a clickable role icon for role assignment, defaulting to auto-guess from champion class (marksman → bot, etc.)

### Draft

- [ ] **DRFT-01**: User can select opponent from a dropdown of scouted opponents, with an "add new opponent" button
- [ ] **DRFT-02**: Draft intel panel has a link to opponents page that auto-saves the current draft before navigating
- [ ] **DRFT-03**: Banned champions are filtered out of the matchup champion selection in intel panel
- [ ] **DRFT-04**: User can navigate back from champion detail to champion selection via a back button in intel panel
- [ ] **DRFT-05**: Champion pool notes (matchups, power spikes, synergies) appear in a draft sidebar when a pooled champion is picked

### Game Plan

- [ ] **PLAN-01**: Game plan auto-fills strategy tag from source draft's tags and name as "<draft name> plan"
- [ ] **PLAN-02**: Clicking "view game plan" from draft navigates to the correct existing game plan with data populated
- [ ] **PLAN-03**: Creating game plan from draft pre-fills our champions and enemy champions from the draft

### Opponents

- [ ] **OPP-01**: New opponent form asks for team name and provides 5 role fields (top/jg/mid/bot/sup) with Riot ID input; on save, auto-fetches champion data from Riot API
- [ ] **OPP-02**: Opponent players show OTP warning badge when >60% of recent games are on a single champion
- [ ] **OPP-03**: Opponent player cards display champion mastery level badges and data recency indicator (last fetched date)
- [ ] **OPP-04**: Each opponent player has a champion pool depth/breadth analysis showing pool size, role flexibility, and comfort picks

### Cross-Feature

- [ ] **XFEAT-01**: Plan effectiveness view aggregates post-game review outcomes per game plan, showing strategy success rate
- [ ] **XFEAT-02**: Draft sidebar shows champion notes from the picker's champion pool (matchup tips, power spikes) when a pooled champion is selected (same data as DRFT-05, displayed contextually)

### Testing

- [ ] **TEST-01**: All test accounts and teams deleted; 2 test teams created with 5 real Riot-linked accounts each, populated with champion pools, notes, and example data
- [ ] **TEST-02**: Playwright CLI workflow configured for testing/verification agents with auth fixture and page navigation helpers

## v2.0 Requirements

Deferred to next milestone. Tracked but not in current roadmap.

### Solo Mode

- **SOLO-01**: Solo/team toggle woven into existing features
- **SOLO-02**: Solo queue stat tracking (rank, LP, winrate)
- **SOLO-03**: Per-match goals and learnings tracking
- **SOLO-04**: Champion improvement tracking over time
- **SOLO-05**: Profile page becomes personal dashboard

### Match Intelligence

- **MATCH-01**: "Gather more info" button on match cards pulls detailed Riot API data
- **MATCH-02**: Queue type displayed on match cards
- **MATCH-03**: Match as launchpad to create draft → game plan → review chain
- **MATCH-04**: Item builds, objective tracking, kill timeline from Riot API

### Intelligence (carried from v1.0)

- **INTL-03**: Ban recommendations based on team champion win rate data
- **INTL-04**: Stats-informed draft pick scoring using match history
- **INTL-05**: Post-game lesson recall when facing previously-seen opponent
- **INTL-06**: Draft outcome correlation ("your team wins 70% with engage comps")
- **FLOW-01**: Game day guided wizard flow

### Cross-Feature (deferred)

- **XFEAT-03**: Series view dashboard (best-of with linked drafts/plans/reviews)
- **XFEAT-04**: Champion pool health check (role coverage gaps, class diversity warnings)
- **XFEAT-05**: Action item completion rate on dashboard

## Out of Scope

| Feature | Reason |
|---------|--------|
| Real-time collaborative editing | WebSocket complexity without proportional value |
| AI/LLM-generated draft picks | No comparable data corpus |
| Video analysis / VOD review | Different product surface |
| Mobile responsive redesign | Desktop-first for v1.x |
| Background opponent data sync | Rate limits; v1.1 uses targeted fetch on save |
| Public leaderboards / social | Shifts product from private tool to platform |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BUG-01 | — | Pending |
| BUG-02 | — | Pending |
| BUG-03 | — | Pending |
| BUG-04 | — | Pending |
| BUG-05 | — | Pending |
| UX-04 | — | Pending |
| UX-05 | — | Pending |
| UX-06 | — | Pending |
| UX-07 | — | Pending |
| UX-08 | — | Pending |
| UX-09 | — | Pending |
| UX-10 | — | Pending |
| DRFT-01 | — | Pending |
| DRFT-02 | — | Pending |
| DRFT-03 | — | Pending |
| DRFT-04 | — | Pending |
| DRFT-05 | — | Pending |
| PLAN-01 | — | Pending |
| PLAN-02 | — | Pending |
| PLAN-03 | — | Pending |
| OPP-01 | — | Pending |
| OPP-02 | — | Pending |
| OPP-03 | — | Pending |
| OPP-04 | — | Pending |
| XFEAT-01 | — | Pending |
| XFEAT-02 | — | Pending |
| TEST-01 | — | Pending |
| TEST-02 | — | Pending |

**Coverage:**
- v1.1 requirements: 28 total
- Mapped to phases: 0
- Unmapped: 28 ⚠️

---
*Requirements defined: 2026-03-19*
*Last updated: 2026-03-19 after initial definition*
