# LoL Team Companion — Cross-Feature Intelligence

## What This Is

A League of Legends team management web app for coordinating drafts, tracking stats, and planning games. Built with Rust/Leptos/SurrealDB, it serves teams who want a single place to prepare for competitive play. Currently feature-complete but siloed — each feature (drafts, stats, game plans, post-game reviews) operates independently without sharing data or context.

## Core Value

Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game. The app feels like one connected tool, not a collection of separate pages.

## Requirements

### Validated

- ✓ User authentication (register, login, logout, sessions) — existing
- ✓ Team management (create, join, roster, join requests) — existing
- ✓ Draft planning (linear drafts with blue/red side) — existing
- ✓ Tree-based draft planning (graph view, node editing) — existing
- ✓ Champion pool management (tier-based) — existing
- ✓ Match history and stats (via Riot API) — existing
- ✓ Game plan creation (pre-game strategy) — existing
- ✓ Post-game review with pattern analysis — existing
- ✓ Opponent scouting profiles — existing
- ✓ Cross-feature action items — existing
- ✓ Team notebook (shared notes) — existing
- ✓ Theme system (dark/light + accent colors) — existing

### Active

- [ ] Smart dashboard that surfaces what matters (upcoming prep, patterns, action items)
- [ ] Stats-informed draft suggestions (win rates, champion performance → draft recommendations)
- [ ] Draft → Game plan pipeline (planned draft flows into pre-game strategy)
- [ ] Post-game → Next game loop (lessons and action items surface in future game prep)
- [ ] Game day guided flow (opponent review → draft plan → game plan → post-game review)
- [ ] Auto-suggestions (ban recommendations from stats, champion pool gap warnings)
- [ ] Meaningful empty states that guide users on what to do next
- [ ] Loading states and skeleton screens for data-fetching pages
- [ ] Clear success/error feedback for all mutations (save, delete, update)

### Out of Scope

- Mobile-specific responsive redesign — not this milestone, desktop-first
- Public launch infrastructure (CI/CD, monitoring, error tracking) — separate effort
- Onboarding wizard for new teams — polish milestone, not intelligence
- Real-time collaboration (WebSocket live updates) — complexity not justified yet
- OAuth / social login — email/password works, not a priority

## Context

This is a brownfield project with ~10,500 lines of page code across 13 route components, plus 3,200 lines of DB queries. The app is functional but each feature is an island — drafts, stats, game plans, and post-game reviews don't share data or context. The codebase has solid patterns (typed DB structs, server functions, reactive signals) that can be extended for cross-feature data flow.

The target user is a competitive League team (5 players + coach) who wants to improve through structured preparation. The app should feel like a coaching assistant that connects the dots between past performance and future strategy.

The app will eventually be public (any team can sign up), so the intelligence features need to work generically — not hardcoded to one team's workflow.

## Constraints

- **Tech stack**: Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x — no changes
- **Single crate**: Must maintain dual SSR/WASM compile target architecture
- **Riot API**: Rate limited per key — cross-feature queries must minimize API calls
- **DB monolith**: `db.rs` is 3,243 lines — new queries add to this, but splitting is out of scope
- **Demo quality**: Should be polished enough to show a friend without caveats

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Intelligence before polish | User feels disconnected data is the bigger gap; polish follows naturally | — Pending |
| No real-time sync | WebSocket adds complexity without proportional value for async team prep | — Pending |
| Desktop-first | Competitive teams primarily prep on desktop; mobile can come later | — Pending |

---
*Last updated: 2026-03-14 after initialization*
