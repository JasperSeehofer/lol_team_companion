---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Immersive Companion
status: executing
stopped_at: Phase 30 context gathered
last_updated: "2026-06-15T13:34:11.405Z"
last_activity: 2026-06-15
progress:
  total_phases: 20
  completed_phases: 15
  total_plans: 80
  completed_plans: 77
  percent: 75
---

## Current Position

Phase: 30 (art-production) — EXECUTING (paused at 30-04 human-action gate)
Plan: 4 of 6 (30-01/02/03 complete; 30-04 blocked on credentials + visual halo inspection)
Status: PAUSED — awaiting human action (set BFL_KEY/FAL_KEY, run test-plane generation, make SC3a halo call)
Next: see .planning/phases/30-art-production/.continue-here.md — resume with `/gsd:execute-phase 30` after running the test plane
Last activity: 2026-06-19

Progress: [██████████] 96%

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-31)

**Core value:** Features talk to each other — and now the app is a *place* worth spending time in: an explorable environment that visibly grows as you put in the work.
**Current focus:** Phase 30 — art-production

## Decisions

- [Phase 30-02]: Per-tier overlay CSS naming locked: `room-overlay--{axis}--tier-{N}` (e.g. `room-overlay--lib--tier-1`). Plans 05/06 generate art at `overlay-{axis}-{N}.webp` paths; the CSS class and asset path are the canonical link.
- [Phase 30-02]: seedSoloTier1Growth seeds lib axis via one `personal_learning` record at `/personal-learnings/new` (lib threshold `[1,8,25]`). Seed-and-assert e2e pattern closes Phase 27 silent-fail gap — tier-visibility now has a regression gate before real art lands.
- [Phase 18.2]: Anti-structural-branching hydration rule — `InitialTheme` (region) must be read ONCE at room top level and passed as `region: String` prop; never inside reactive closures or child components. Reading context inside reactive primitive caused Phase 18.2 tachys `Option::unwrap()` panic. Rule now required in CLAUDE.md + leptos-patterns.md BEFORE any new room is built (ROOM-03).
- [Phase 19]: Bug-report inbox auto-exports on server start; CLAUDE.md instructs future sessions to read it if `total_open > 0`.
- [Phase 29 close-out]: All 5 SCs verified green via independent spec re-run. Orchestrator VERIFIED a team-user-only regression: `/throne?tab=development` panics at `tachys hydration.rs:216` (`pageerror: unreachable`), freezing the room, because `ActionItemsPanel`'s own `Resource::new` sits inside `DevelopmentSurface`'s conditional Suspense branch (nested-resource hydration mismatch, Leptos 0.8). SC4's no-team canary doesn't surface it. Closed Phase 29 with the issue documented; inserted Phase 29.1 (Suspend::new refactor of action_items.rs) to fix. User decision: verify-then-close + follow-up.
- [Phase 29.1]: Throne Development-tab hydration panic fixed and VERIFIED (5/5 must-haves). Root cause was NOT the panel's `.get().map` reads but the conditional-Suspense-branch NESTING — a resource-backed sub-component (ActionItemsPanel/TeamNotebookInner) mounted inside the `Ok(Some(_))` branch of DevelopmentSurface's outer `dash` Suspense cannot hydrate in Leptos 0.8 (SSR-resolved vs WASM-pending structural mismatch → tachys panic). Fix: mount the sub-components UNCONDITIONALLY (both already return `Ok(Vec::new())` for no-team, rule #44) and gate the no-team CTA with a reactive `hidden` banner; collapse panel/page to a SINGLE Suspense awaiting all resources once. New generalizable gotcha (candidate for leptos-patterns.md alongside #58/#19): never embed a resource-backed sub-component inside a conditional branch of an outer Suspense — mount unconditionally + reactive `hidden`. Deviation overrode the plan's "do not touch DevelopmentSurface" instruction (Rule 1 — plan instruction contradicted its own binding must_haves.truths); justified and verified.
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
- [Phase 28-03]: Callback<T,()> must be created at component scope (not inside a FnMut reactive closure) when it captures non-Copy values — otherwise the outer closure becomes FnOnce; Resource::new needs move || signal.get() source closure when input is a Signal<T>
- [Phase 28-03]: #[prop(optional)] Option<Callback<T,()>> — pass bare Callback at call site (Leptos macro wraps in Some); StatsPage omits prop (None = href fallback), StatsSurface supplies it (Some = signal+navigate)
- [Phase 28-04]: PostGameSurface carries all CRUD state verbatim (Resources/signals/Effects); PostGamePage becomes canvas-grain+SectionHead+Card shell only — avoids any shared-state coupling between page and surface
- [Phase 28-04]: PersonalLearningsSurface strips page h1 header but preserves + New learning CTA in surface-level header div; both CTAs point at /personal-learnings/new (D-11 mandate)
- [Phase 28-05]: region prop accepted by LibrarySurfaces but prefixed _region; none of the 5 surfaces need it — keeps prop parity with WarSurfaces without unused-variable warning
- [Phase 28-05]: provide_context(selected_match_id) placed before panels vec and view! in LibrarySurfaces — both keep-mounted Stats and Match Detail panels find signal in context on mount (D-07 bridge, Pitfall 4)
- [Phase 28-05]: Tab IDs stats/match/analytics/post-game/personal match Plan 06 redirect targets (?tab=...)
- [Phase ?]: Dual redirect pattern (SSR leptos_axum::redirect + hydrate set_href) used for all six IA-healing redirect shells — same contract as roster.rs/profile.rs
- [Phase ?]: MatchRedirect reads use_params_map id at component top; dispatches RedirectToLibraryMatch(id) server fn so SSR 302 preserves the match id (D-11)
- [Phase Phase 29-01]: ThroneGrowthState has NO counts field — progression derives from user ranked games independently of team membership (D-05/D-07 lock)
- [Phase Phase 29-01]: get_throne_growth_counts: pre-authorized team_id caller contract, parameterized type::record(), 2-statement batch count (roster + crown wins)
- [Phase ?]: [Phase 29-02]: Wave 0 CSS selectors + placeholder WebPs before room render plan; test skip guards keep suite green
- [Phase ?]: TeamNotebookInner: auth resolves server-side, no props needed
- [Phase ?]: Game Day Brief mode dropped from surfaces; DashboardSurface retains prep-priority panel content
- [Phase ?]: DevelopmentSurface: ActionItemsPanel imported from action_items.rs, TeamNotebookInner resolves auth server-side independently of team gate
- [Phase ?]: [Phase 29.1-01]: Team-user /throne?tab=development hydration panic root cause was the conditional-Suspend-branch NESTING in DevelopmentSurface, not ActionItemsPanel .get().map reads. Fix: mount ActionItemsPanel + TeamNotebookInner UNCONDITIONALLY (handle no-team via Ok(Vec::new), rule #44); no-team CTA is a reactive hidden banner; panel/page collapsed to a single Suspense.
- [Phase ?]: [Phase 29.1-01]: Leptos 0.8 gotcha — a resource-backed sub-component mounted inside a conditional branch of an OUTER Suspense panics at hydration; mount unconditionally and gate optional chrome with a reactive hidden attribute, not a Suspend branch.

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

Last session: 2026-06-15T13:34:11.387Z
Stopped at: Phase 30 context gathered
Resume file: None
Next command: Begin Phase 29 (Throne Room)
