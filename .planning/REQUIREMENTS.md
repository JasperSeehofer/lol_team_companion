# Requirements: LoL Team Companion — Cross-Feature Intelligence

**Defined:** 2026-03-14
**Core Value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.

## v1 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Cross-Feature Pipelines

- [ ] **PIPE-01**: User can create a game plan from a completed draft, with picks/bans/side prefilled
- [ ] **PIPE-02**: Post-game review auto-generates action items from identified patterns
- [ ] **PIPE-03**: Draft planner warns when a drafted champion is not in the assigned player's pool
- [ ] **PIPE-04**: Draft planner shows opponent tendency sidebar (historical picks/bans from scouting data)

### Intelligence

- [ ] **INTL-01**: Smart dashboard surfaces prep priorities (upcoming game context, incomplete workflows, recent action items)
- [ ] **INTL-02**: Win condition tracker shows whether declared win conditions were achieved over time

### UX Polish

- [ ] **UX-01**: All data pages show meaningful empty states with contextual CTAs when no data exists
- [ ] **UX-02**: All data-fetching pages use skeleton loading screens instead of blank/spinner
- [ ] **UX-03**: All mutations (save, delete, update) show consistent success/error feedback

## v2 Requirements

Deferred to future milestone. Tracked but not in current roadmap.

### Intelligence (Deferred)

- **INTL-03**: Ban recommendations based on team's own champion win rate data
- **INTL-04**: Stats-informed draft pick scoring using team match history per champion
- **INTL-05**: Post-game lesson recall when creating game plan vs previously-faced opponent
- **INTL-06**: Draft outcome correlation ("your team wins 70% with engage comps")

### Workflow

- **FLOW-01**: Game day guided wizard flow (linear prep sequence replacing checklist)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Real-time collaborative editing | Infrastructure complexity (WebSocket, conflict resolution) without proportional value for async team prep |
| AI/LLM-generated draft picks | No comparable data corpus; surfaces own-data win rates instead |
| Video analysis / VOD review | Completely different product surface requiring upload/storage/ML |
| Mobile responsive redesign | Desktop-first per constraints; mobile comes after intelligence |
| Public leaderboards / social | Shifts product from private team tool to public platform |
| Riot API live in-game overlay | Requires desktop client; fundamentally different distribution |
| Auto opponent stats via Riot API | Rate limits make per-game scouting expensive; keep manual + structured |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PIPE-01 | Phase 1 | Pending |
| PIPE-02 | Phase 5 | Pending |
| PIPE-03 | Phase 4 | Pending |
| PIPE-04 | Phase 4 | Pending |
| INTL-01 | Phase 3 | Pending |
| INTL-02 | Phase 4 | Pending |
| UX-01 | Phase 5 | Pending |
| UX-02 | Phase 5 | Pending |
| UX-03 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-14*
*Last updated: 2026-03-14 after roadmap creation*
