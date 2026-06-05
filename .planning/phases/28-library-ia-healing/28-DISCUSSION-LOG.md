# Phase 28: Library & IA Healing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 28-library-ia-healing
**Areas discussed:** Match-detail surface, Learnings surface identity, IA-healing routes, Library growth axes

---

## Match-detail surface

### What the Match tab shows on first open (no id selected)

| Option | Description | Selected |
|--------|-------------|----------|
| Most-recent match | Auto-load the user's latest synced match | |
| Empty-state + picker | "Select a match from Stats" until one is chosen | |
| Recent-match list in-tab | Tab lists recent matches; selecting expands scoreboard inline | ✓ |

### How a /match/:id deep link resolves

| Option | Description | Selected |
|--------|-------------|----------|
| Library deep link w/ match param | /match/:id resolves into the Library Match Detail surface | |
| Keep /match/:id standalone | Stays a standalone full-page view | |
| You decide | Planner picks cleanest routing | ✓ |

### Stats ↔ Match relationship (follow-up)

| Option | Description | Selected |
|--------|-------------|----------|
| Stats rows link to Match tab | Clicking a Stats row switches to Match Detail tab with that match expanded | ✓ |
| Keep them fully independent | Each surface has its own list, no interaction | |
| You decide | Planner picks | |

**User's choice:** Recent-match list in-tab; deep-link routing left to planner; Stats rows are the canonical "open a match" path into the Match Detail tab.
**Notes:** With the later 302-redirect decision, /match/:id will redirect into the Library Match Detail tab with the id preserved (D-11). Two keep-mounted surfaces need shared match-selection state (D-07).

---

## Learnings surface identity

| Option | Description | Selected |
|--------|-------------|----------|
| Post-Game + Personal (2 tabs) | 5 surfaces: Stats, Match Detail, Analytics, Post-Game (team reviews), Personal Learnings (journal); each stays its own feature | ✓ |
| One unified Learnings tab | Merge team reviews + personal journal into one toggled surface | |
| You decide | Planner judges separability | |

**User's choice:** Post-Game + Personal as two distinct tabs.
**Notes:** SC1's "team learnings" = the Post-Game tab (not a separate feature). Team Notebook stays in the team dashboard (Phase 29), not a Library surface — it only feeds Library growth.

---

## IA-healing routes

### Route strategy

| Option | Description | Selected |
|--------|-------------|----------|
| 302 redirect to intent room | Legacy routes redirect into the room with the right tab active | ✓ |
| Thin-shell render in place | Legacy routes keep rendering standalone (Phase 27 D-02 style) | |
| You decide | Planner picks per-route | |

### Redirect scope

| Option | Description | Selected |
|--------|-------------|----------|
| Library routes + /opponents only | 5 Library routes + /opponents→War Room; leave /draft, /tree-drafter, /game-plan | ✓ |
| All healed room routes | Also redirect /draft, /tree-drafter, /game-plan | |
| You decide | Planner sets exact set | |

### Sub-routes (/personal-learnings/new, /match/:id)

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve params, heal index | Carry id through; keep /new flow working | ✓ |
| Redirect everything to tab root | Drop id and /new | |
| You decide | Planner handles | |

**User's choice:** 302 redirect; scope = 5 Library routes + /opponents only (War Room routes untouched); preserve params and the /personal-learnings/new flow.
**Notes:** This is a deliberate step beyond Phase 27 D-02 (which preserved standalone routes) — Phase 28 retires the standalone Library pages in favor of one canonical room home each. "Resolves rather than 404ing" (SC2) is satisfied by the redirect resolving.

---

## Library growth axes

### Axis → data mapping

| Option | Description | Selected |
|--------|-------------|----------|
| records=syncs, tomes=learnings, analysis=reviews | Intuitive metaphor; counts defined in CONTEXT | ✓ |
| Sketch intent, planner finalizes counts | Lock metaphor, planner picks counts | |
| You decide | Planner defines entirely | |

### Growth scope

| Option | Description | Selected |
|--------|-------------|----------|
| Team-scoped, mirror War Room | get_library_growth_state wraps team counts; no-team→tier 0, never Err | ✓ |
| Blend team + personal | Personal data counts even with no team | |
| You decide | Planner chooses | |

**User's choice:** records=match syncs, tomes=written learnings, analysis=reviews/analytical depth; team-scoped mirroring War Room.
**Notes:** Environmental-only growth (no badges); verified via data-*-tier attributes (Phase 27 D-11 pattern). Exact count sources/thresholds finalized by planner against db.rs.

---

## Claude's Discretion

- Deep-link routing mechanism for /match/:id → Library (nested route vs query param).
- Shared match-selection state mechanism between Stats and Match Detail surfaces.
- Asset paths, plane/overlay count, parallax depths (mirror war_room.rs).
- Internal module layout for the five *Surface components.
- get_library_growth_state placement; exact records/tomes/analysis count sources + tier thresholds.

## Deferred Ideas

- Redirect /draft, /tree-drafter, /game-plan into War Room (War Room cleanup, out of scope).
- Blended team + personal Library growth for solo users (deferred; stays team-scoped).
- QuickNav (Phase 29); Throne / Team Notebook room (Phase 29).
- Real FLUX art (Phase 30); eager-fetch optimization for mounted surfaces.
- Unified Post-Game + Personal learnings surface (rejected D-02; revisit if redundant to beta users).
