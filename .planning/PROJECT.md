# LoL Team Companion

## What This Is

A League of Legends team management web app for coordinating drafts, tracking stats, and planning games. Built with Rust/Leptos/SurrealDB. Features are deeply connected — drafts flow into game plans with autofill, post-game reviews track win/loss and rating per strategy, analytics surface plan effectiveness, and opponent scouting feeds intel into the draft sidebar.

## Core Value

Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game. The app feels like one connected tool, not a collection of separate pages.

## Requirements

### Validated

- ✓ User authentication (register, login, logout, sessions) — pre-v1.0
- ✓ Team management (create, join, roster, join requests) — pre-v1.0
- ✓ Draft planning (linear drafts with blue/red side) — pre-v1.0
- ✓ Tree-based draft planning (graph view, node editing) — pre-v1.0
- ✓ Champion pool management (tier-based) — pre-v1.0
- ✓ Match history and stats (via Riot API) — pre-v1.0
- ✓ Game plan creation (pre-game strategy) — pre-v1.0
- ✓ Post-game review with pattern analysis — pre-v1.0
- ✓ Opponent scouting profiles — pre-v1.0
- ✓ Cross-feature action items — pre-v1.0
- ✓ Team notebook (shared notes) — pre-v1.0
- ✓ Theme system (dark/light + accent colors) — pre-v1.0
- ✓ Game plan from draft with picks/bans/side prefill (PIPE-01) — v1.0
- ✓ Post-game auto-generates action items from patterns (PIPE-02) — v1.0
- ✓ Draft warns on champion pool gaps (PIPE-03) — v1.0
- ✓ Opponent tendency sidebar on draft page (PIPE-04) — v1.0
- ✓ Smart dashboard with prep priorities (INTL-01) — v1.0
- ✓ Win condition tracker with historical stats (INTL-02) — v1.0
- ✓ Meaningful empty states with CTAs on all pages (UX-01) — v1.0
- ✓ Skeleton loading screens on all data pages (UX-02) — v1.0
- ✓ Consistent mutation feedback via toast system (UX-03) — v1.0
- ✓ Fix tree drafter, game plan view, roster, hydration, drag-and-drop bugs (BUG-01–05) — v1.1
- ✓ UX polish: toasts, timestamps, search, champion pool overhaul, role icons (UX-04–10) — v1.1
- ✓ Draft: opponent dropdown, intel improvements, pool notes sidebar (DRFT-01–05) — v1.1
- ✓ Game plan: autofill from draft, fix empty view (PLAN-01–03) — v1.1
- ✓ Opponents redesign: 5-role form, auto-fetch, OTP detection, mastery, pool depth (OPP-01–04) — v1.1
- ✓ Plan effectiveness dashboard and draft-integrated champion notes (XFEAT-01–02) — v1.1
- ✓ Testing infrastructure: seed binary and Playwright CLI workflow (TEST-01–02) — v1.1

### Active

## Current Milestone: v1.2 Solo Mode & Match Intelligence

**Goal:** Open the app to individual players with solo queue tracking, personal goals, and learnings — plus richer match detail from the Riot API that connects back into champion pool and draft decisions.

**Target features:**
- [ ] Solo/team toggle with personal dashboard
- [ ] Solo queue goal setting (rank targets, CS/min, death count)
- [ ] Ranked climb tracker (LP gains/losses, streaks, rank history)
- [ ] Personal learnings journal (post-game reflections, tagged by champion/matchup)
- [ ] Match detail view — full participant data (10 players), items, damage, gold
- [ ] Match timeline — objectives (dragons, baron, towers), key events
- [ ] Personal performance breakdown (damage share, vision score, CS graphs)
- [ ] Match → learning one-click flow
- [ ] Champion performance trends over time (win rate, KDA, CS/min per champion)
- [ ] Personal matchup notes (surfaced when picking that champion in draft)
- [ ] Practice list with auto-tracked progress from match history

**Future (deferred beyond v1.2):**
- [ ] Match as launchpad: create draft → game plan → review chain
- [ ] Ban recommendations based on team champion win rate data (INTL-03)
- [ ] Stats-informed draft pick scoring using match history (INTL-04)
- [ ] Post-game lesson recall when facing previously-seen opponent (INTL-05)
- [ ] Draft outcome correlation ("your team wins 70% with engage comps") (INTL-06)
- [ ] Game day guided wizard flow (FLOW-01)
- [ ] Series view, champion pool health check, action item completion rate

### Out of Scope

- Real-time collaborative editing — WebSocket complexity without proportional value for async team prep
- AI/LLM-generated draft picks — no comparable data corpus; surfaces own-data win rates instead
- Video analysis / VOD review — completely different product surface
- Mobile responsive redesign — desktop-first; mobile comes after intelligence features
- Public leaderboards / social — shifts product from private team tool to public platform
- Riot API live in-game overlay — requires desktop client
- Auto opponent stats via Riot API — rate limits make per-game scouting expensive (v1.1 adds targeted fetch on save, not background sync)
- Offline mode — real-time data is core value

## Context

Shipped v1.1 with 26,686 lines of Rust across 15 route components + server/DB layer.
Tech stack: Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x / Tailwind CSS v4.
Features are deeply connected end-to-end: draft → game plan → post-game → analytics → action items.
Opponent scouting feeds intel into draft sidebar with OTP badges and pool analysis.
Dashboard surfaces prep priorities with independently-loading panels.
All pages have consistent UX (skeletons, empty states, toast feedback).
Seed binary creates realistic demo data for all features.

v1.0 tech debt resolved in v1.1:
- ✓ Integration tests for DB aggregation functions (added)
- ✓ `post_game_champ_outcomes` empty results (schema now has win/loss + rating fields)

Remaining tech debt:
- `db.rs` is ~4,000 lines — splitting is a future refactor
- Dashboard incomplete workflow counts not surfaced (deferred)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Intelligence before polish | Disconnected data was the bigger gap; polish followed naturally | ✓ Good — v1.0 shipped both |
| No real-time sync | WebSocket adds complexity without proportional value for async team prep | ✓ Good — not needed |
| Desktop-first | Competitive teams primarily prep on desktop | ✓ Good — appropriate for v1.x |
| Champion name normalization in Rust | SurrealDB lacks good string normalization; 3-pass lookup | ✓ Good — enabled cross-feature joins |
| In-memory filter for opponent stats | Unit-testable over SurrealQL join approach | ✓ Good — clean separation |
| Toast system via context provider | AtomicU64 for ID gen; Callback::new for Copy closures | ✓ Good — works across all pages |
| Gap closure phases (4, 5) | Milestone audit found requirements unsatisfied → created targeted phases | ✓ Good — systematic coverage |
| agent-browser over Playwright MCP | npm package (Vercel Labs) simpler to integrate as Claude Code skill | ✓ Good — reliable browser verification |
| Champion pool card grid with drag-and-drop | DataTransfer API for tier management; aspect-square portraits | ✓ Good — substantial UX improvement |
| Opponent 5-role form with auto-fetch | Two-step create then fetch avoids transaction ambiguity | ✓ Good — reliable Riot API integration |
| Rust-side HashMap aggregation for analytics | Avoids SurrealQL JOIN ambiguity; unit-testable | ✓ Good — clean separation |
| serde(default) for backward compat | PostGameLearning and DraftAction fields default gracefully | ✓ Good — no migration needed |

## Constraints

- **Tech stack**: Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x — no changes
- **Single crate**: Must maintain dual SSR/WASM compile target architecture
- **Riot API**: Rate limited per key — cross-feature queries must minimize API calls
- **DB monolith**: `db.rs` is ~4,000 lines — splitting is a future refactor
- **Demo quality**: Should be polished enough to show a friend without caveats

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-25 after v1.2 milestone started*
