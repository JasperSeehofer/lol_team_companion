# Requirements: LoL Team Companion

**Defined:** 2026-03-25
**Core Value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.

## v1.2 Requirements

Requirements for Solo Mode & Match Intelligence milestone. Each maps to roadmap phases.

### Solo Mode

- [ ] **SOLO-01**: User can toggle between team mode and solo mode, persisted across sessions
- [ ] **SOLO-02**: User can sync solo queue match history without being on a team
- [ ] **SOLO-03**: User can filter match history by queue type (solo queue, flex, all)
- [ ] **SOLO-04**: User sees a personal solo dashboard with recent matches, ranked badge, learnings, and goal status
- [ ] **SOLO-05**: User can select their region during Riot account linking (EUW, NA, KR, etc.)

### Ranked Tracking

- [ ] **RANK-01**: User can see their current ranked tier, division, and LP
- [ ] **RANK-02**: User can view LP history over time as a trend graph
- [ ] **RANK-03**: System snapshots LP on each stats sync for history tracking

### Match Detail

- [ ] **MATCH-01**: User can view all 10 participants in a match (KDA, items, damage, gold, vision)
- [ ] **MATCH-02**: User can view match timeline events (dragons, barons, towers, kills)
- [ ] **MATCH-03**: User can see personal performance breakdown (damage share, vision score, CS vs game average)
- [ ] **MATCH-04**: Match detail loads on-demand (not during sync) and caches in DB after first fetch

### Personal Improvement

- [ ] **LEARN-01**: User can write personal learnings tagged by champion and matchup
- [ ] **LEARN-02**: User can create a learning from match detail with one click (pre-fills champion/matchup)
- [ ] **LEARN-03**: User can browse and filter learnings by champion, matchup, or tag
- [ ] **LEARN-04**: User can set goals (rank target, CS/min threshold, death limit)
- [ ] **LEARN-05**: User can see goal progress auto-tracked from match history
- [ ] **LEARN-06**: User can view champion performance trends (win rate, KDA, CS/min per champion over time)

### Cross-Feature

- [ ] **XFEAT-03**: User sees personal matchup notes in the draft Intel sidebar when picking a champion
- [ ] **XFEAT-04**: User sees champion win rate badges on champion pool cards from solo match history

## Future Requirements

Deferred to v1.3+. Tracked but not in current roadmap.

### Practice & Automation

- **PRAC-01**: User can maintain a practice list of champions to learn
- **PRAC-02**: Practice list auto-tracks progress from match history
- **AUTO-01**: LP auto-sync polling without manual trigger
- **POOL-01**: Champion pool gap detection from solo match history

### Intelligence (carried from v1.1)

- **INTL-03**: Ban recommendations based on team champion win rate data
- **INTL-04**: Stats-informed draft pick scoring using match history
- **INTL-05**: Post-game lesson recall when facing previously-seen opponent
- **INTL-06**: Draft outcome correlation ("your team wins 70% with engage comps")

### Workflow

- **FLOW-01**: Game day guided wizard flow
- **FLOW-02**: Match as launchpad: create draft → game plan → review chain
- **VIEW-01**: Series view, champion pool health check, action item completion rate

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Practice list auto-progress | Requires stable match aggregation first; defer to v1.3 |
| Auto LP polling | Rate limit management complexity; manual sync sufficient for v1.2 |
| Per-minute timeline frames storage | 300 rows/game storage bloat; extract only objective/kill events |
| Raw timeline JSON storage | 300-500KB per match; extract and discard |
| Real-time collaborative editing | WebSocket complexity without proportional value |
| AI/LLM-generated draft picks | No comparable data corpus |
| Mobile responsive redesign | Desktop-first |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SOLO-01 | — | Pending |
| SOLO-02 | — | Pending |
| SOLO-03 | — | Pending |
| SOLO-04 | — | Pending |
| SOLO-05 | — | Pending |
| RANK-01 | — | Pending |
| RANK-02 | — | Pending |
| RANK-03 | — | Pending |
| MATCH-01 | — | Pending |
| MATCH-02 | — | Pending |
| MATCH-03 | — | Pending |
| MATCH-04 | — | Pending |
| LEARN-01 | — | Pending |
| LEARN-02 | — | Pending |
| LEARN-03 | — | Pending |
| LEARN-04 | — | Pending |
| LEARN-05 | — | Pending |
| LEARN-06 | — | Pending |
| XFEAT-03 | — | Pending |
| XFEAT-04 | — | Pending |

**Coverage:**
- v1.2 requirements: 20 total
- Mapped to phases: 0
- Unmapped: 20 ⚠️

---
*Requirements defined: 2026-03-25*
*Last updated: 2026-03-25 after initial definition*
