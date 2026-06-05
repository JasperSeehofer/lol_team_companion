---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Immersive Companion
status: executing
stopped_at: Completed 27-01-PLAN.md (RoomSurface keep-mounted PanelFn refactor)
last_updated: "2026-06-05T17:50:09.196Z"
last_activity: 2026-06-05
progress:
  total_phases: 19
  completed_phases: 11
  total_plans: 59
  completed_plans: 58
  percent: 58
---

## Current Position

Phase: 27 (war-room) — EXECUTING
Plan: 8 of 8
Status: Ready to execute
Last activity: 2026-06-05

Progress: [██████████] 98%

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-31)

**Core value:** Features talk to each other — and now the app is a *place* worth spending time in: an explorable environment that visibly grows as you put in the work.
**Current focus:** Phase 27 — war-room

## Decisions

- [Phase 18.2]: Anti-structural-branching hydration rule — `InitialTheme` (region) must be read ONCE at room top level and passed as `region: String` prop; never inside reactive closures or child components. Reading context inside reactive primitive caused Phase 18.2 tachys `Option::unwrap()` panic. Rule now required in CLAUDE.md + leptos-patterns.md BEFORE any new room is built (ROOM-03).
- [Phase 19]: Bug-report inbox auto-exports on server start; CLAUDE.md instructs future sessions to read it if `total_open > 0`.
- [Roadmap v1.4]: Old v1.3 Phases 20-25 (never built) superseded by v1.4 Phases 31-34. Phase numbers 20, 20.1, 21, 22, 23, 24, 25 are retired.
- [Roadmap v1.4]: Build order — Phase 26 gates all rooms; Phases 27-29 rooms-first; Phase 30 art calendar-parallel with 27-29 (art swaps in at existing file paths, no code change); Phases 31-34 launch after rooms complete.
- [Roadmap v1.4]: Room system uses extract-then-compose pattern (WarSurfaces, LibSurfaces, ThroneSurfaces over AtmosphereRoot); intra-room tab switching via RwSignal<ActiveSurface> + Show, NOT sub-routes (sub-routes remount AtmosphereRoot destroying parallax continuity).
- [Roadmap v1.4]: Art identity — Riot IP go/no-go must be logged in PROJECT.md Key Decisions before first FLUX generation run; `/consult legal` gates first real run. Decision: original-world (Path A) or accepted-risk LoL-names (Path B).
- [Phase 27-01]: Leptos 0.8 native element attribute syntax — `hidden=move || Option<&str>` without `attr:` prefix on native HTML elements; `attr:` is only valid as a component prop pass-through and is rejected by `view!` macro on native elements.
- [Phase 27-01]: RoomSurface contract changed to `panels: Vec<(SurfaceTab, PanelFn)>` (D-06/D-07/D-09); all tabpanels mount eagerly and toggle via `hidden` attribute; `PanelFn` re-exported from `components/room/mod.rs`.
- [Phase ?]: DraftSurface receives region as plain String prop (rule #58); BanPrioritySurface has no region prop
- [Phase ?]: banned_champions live-draft closure stays in DraftSurface (D-05); persistent ban_priority editor moves to BanPrioritySurface
- [Phase ?]: TreeDrafterSurface(region: String) extracted chrome-free; TreeDrafterPage thin shell; rule #58 enforced

## Blockers/Concerns

- [Phase 30 gate]: Riot IP go/no-go decision must be made before any FLUX production run — project owner decision only. `/consult legal` required before committing generated assets.
- [Phase 26]: Confirm `ban_priority` and `team_note` exact table names in `schema.surql` before writing `get_team_growth_counts` — research flagged these as unconfirmed.
- [Phase 26]: `prefers-reduced-motion` runtime listener is missing from `AtmosphereRoot` (only a mount-time check exists) — must be added in Phase 26 (A11Y-01) before rooms ship.
- [Phase 29]: QuickNav focus trap not implemented — WCAG 2.1 SC 2.1.2 failure. Fix required in Phase 29 (ROOM-08).

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Riot IP | go/no-go for FLUX art generation | Pending project owner decision | v1.4 roadmap |
| Growth tiers | Calibrate tier thresholds against real beta user data | Post-beta | v1.4 research |
| Mousemove guard | AtomicBool "listener active" guard vs. Phase 18.2 accumulation pattern | Phase 27 scope | v1.4 research |

## Session Continuity

Last session: 2026-06-05T17:50:09.183Z
Stopped at: Completed 27-01-PLAN.md (RoomSurface keep-mounted PanelFn refactor)
Resume file: None
Next command: `/gsd:execute-phase 27-war-room plan 02`
