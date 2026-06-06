---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Immersive Companion
status: executing
stopped_at: Completed 28-01-PLAN.md (Library growth data layer)
last_updated: "2026-06-06T00:49:09.203Z"
last_activity: 2026-06-06
progress:
  total_phases: 19
  completed_phases: 12
  total_plans: 66
  completed_plans: 61
  percent: 63
---

## Current Position

Phase: 28 (library-ia-healing) — EXECUTING
Plan: 3 of 7
Status: Ready to execute
Last activity: 2026-06-06

Progress: [█████████░] 92%

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-31)

**Core value:** Features talk to each other — and now the app is a *place* worth spending time in: an explorable environment that visibly grows as you put in the work.
**Current focus:** Phase 28 — library-ia-healing

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
- [Phase 28-01]: D-13 thresholds are calibration hypotheses — records [1,10,30], tomes/analysis [1,5,15] — tunable post-beta without touching room UI
- [Phase 28-01]: match.team_id vs post_game_learning/team_note.team field-name distinction (Pitfall 5) documented in get_library_growth_counts doc comment

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

Last session: 2026-06-06T00:49:09.176Z
Stopped at: Completed 28-01-PLAN.md (Library growth data layer)
Resume file: None
Next command: `/gsd:execute-phase 28-library-ia-healing plan 02`
