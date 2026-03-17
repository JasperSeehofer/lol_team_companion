# LoL Team Companion

## What This Is

A League of Legends team management web app for coordinating drafts, tracking stats, and planning games. Built with Rust/Leptos/SurrealDB. Features are connected — drafts flow into game plans, post-game reviews auto-generate action items, and the dashboard surfaces prep priorities, pool gaps, and recent patterns at a glance.

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

### Active

- [ ] Ban recommendations based on team champion win rate data (INTL-03)
- [ ] Stats-informed draft pick scoring using match history (INTL-04)
- [ ] Post-game lesson recall when facing previously-seen opponent (INTL-05)
- [ ] Draft outcome correlation ("your team wins 70% with engage comps") (INTL-06)
- [ ] Game day guided wizard flow (FLOW-01)

### Out of Scope

- Real-time collaborative editing — WebSocket complexity without proportional value for async team prep
- AI/LLM-generated draft picks — no comparable data corpus; surfaces own-data win rates instead
- Video analysis / VOD review — completely different product surface
- Mobile responsive redesign — desktop-first; mobile comes after intelligence features
- Public leaderboards / social — shifts product from private team tool to public platform
- Riot API live in-game overlay — requires desktop client
- Auto opponent stats via Riot API — rate limits make per-game scouting expensive
- Offline mode — real-time data is core value

## Context

Shipped v1.0 with 22,986 lines of Rust across 13 route components + server/DB layer.
Tech stack: Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x / Tailwind CSS v4.
Features are now connected end-to-end: draft → game plan → post-game → action items.
Dashboard surfaces prep priorities with independently-loading panels.
All pages have consistent UX (skeletons, empty states, toast feedback).

Known tech debt from v1.0:
- Missing integration tests for DB aggregation functions
- `post_game_champ_outcomes` returns empty (schema lacks win/loss field)
- Dashboard incomplete workflow counts not surfaced (deferred)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Intelligence before polish | Disconnected data was the bigger gap; polish followed naturally | ✓ Good — v1.0 shipped both |
| No real-time sync | WebSocket adds complexity without proportional value for async team prep | ✓ Good — not needed |
| Desktop-first | Competitive teams primarily prep on desktop | ✓ Good — appropriate for v1.0 |
| Champion name normalization in Rust | SurrealDB lacks good string normalization; 3-pass lookup (exact ID, case-insensitive, fuzzy) | ✓ Good — enabled cross-feature joins |
| In-memory filter for opponent stats | Unit-testable over SurrealQL join approach | ✓ Good — clean separation |
| Toast system via context provider | AtomicU64 for ID gen (Cell not Sync); Callback::new for Copy closures | ✓ Good — works across all pages |
| Gap closure phases (4, 5) | Milestone audit found requirements unsatisfied → created targeted phases | ✓ Good — systematic coverage |

## Constraints

- **Tech stack**: Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x — no changes
- **Single crate**: Must maintain dual SSR/WASM compile target architecture
- **Riot API**: Rate limited per key — cross-feature queries must minimize API calls
- **DB monolith**: `db.rs` is ~4,000 lines — splitting is a future refactor
- **Demo quality**: Should be polished enough to show a friend without caveats

---
*Last updated: 2026-03-18 after v1.0 milestone*
