---
phase: 26-room-system-foundations
plan: 04
subsystem: ui
tags: [leptos, rust, wasm, aria, tabs, routing, room-system]

# Dependency graph
requires:
  - phase: solo-journey (pre-26)
    provides: quick_nav.rs <Show>+ARIA analog; opponents.rs use_query_map pattern
provides:
  - RoomSurface — reusable intra-room tab component (ARIA tablist, ?tab= sync, signal+<Show>)
  - SurfaceTab — tab descriptor struct
  - resolve_active_tab — pure, unit-tested active-tab resolver
affects: [war-room, 27, library, throne, 28, 29]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Intra-room navigation = signal + <Show> only, never sub-routes (D-02 — keeps AtmosphereRoot mounted)"
    - "?tab= deep-link via use_query_map (read) + use_navigate (write)"
    - "Active surface id exposed to children via provide_context(Signal<&'static str>)"

key-files:
  created:
    - src/components/room/room_surface.rs
  modified:
    - src/components/room/mod.rs

key-decisions:
  - "Unknown/empty ?tab= falls back to the first tab (D-01 discretion)"
  - "Active tab id provided to children via context so panels can branch without re-reading the URL"

patterns-established:
  - "RoomSurface is the locked intra-room nav contract every room consumes (War Room is first real consumer, Phase 27)"

requirements-completed: [ROOM-02]

# Metrics
duration: 12min
completed: 2026-06-04
---

# Phase 26 / Plan 04: RoomSurface Tab Component Summary

**RoomSurface — a standalone, reusable, ARIA-complete intra-room tab bar with `?tab=` deep-linking and signal+`<Show>` switching (never sub-routes) — the navigation contract every room will consume, shipped with a pure unit-tested tab resolver.**

## Performance

- **Duration:** ~12 min (inline execution; salvaged a draft from the aborted worktree run, then fixed and verified it)
- **Completed:** 2026-06-04
- **Tasks:** 2
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments
- `SurfaceTab { id, label }` descriptor + `pub(crate) fn resolve_active_tab` (pure, framework-free, fallback-to-first)
- `RoomSurface` component: `role="tablist"`/`tab`/`tabpanel`, `aria-selected`/`aria-controls`/`aria-labelledby`; `?tab=` read via `use_query_map`, written via `use_navigate`; one `<Show>`-gated panel per surface (no `<Route>`)
- Active surface id exposed to panel children via `provide_context(Signal<&'static str>)`
- Four resolver unit tests (dummy surfaces): known→index, empty→first, unknown→first, explicit-first

## Task Commits

1. **Task 1: RoomSurface + SurfaceTab + resolve_active_tab + unit tests** - `4ba3f3d` (feat)
2. **Task 2: Register & re-export in mod.rs** - `f78d936` (feat)

## Files Created/Modified
- `src/components/room/room_surface.rs` - The component, descriptor, pure resolver, and tests
- `src/components/room/mod.rs` - `pub mod room_surface;` + `pub use room_surface::{RoomSurface, SurfaceTab};`

## Decisions Made
- Added `provide_context(active_id)` so panel children can branch on the active surface (the plan left this to planner's discretion; the draft's doc comment already promised it).

## Deviations from Plan

### Auto-fixed Issues

**1. [Correctness — compile] ChildrenFn moved across multiple panels (E0507)**
- **Found during:** Task 1 verification (`cargo test --features ssr --lib room_surface`)
- **Issue:** The salvaged draft rendered `{children()}` inside a per-panel `.map` closure, moving the `ChildrenFn` out across iterations → `cannot move out of captured variable in an FnMut closure`. The original worktree agent was killed before it ever compiled this.
- **Fix:** Clone the `Rc`-backed `ChildrenFn` per panel (`let children = children.clone();`) so each `<Show>` body owns a callable handle.
- **Files modified:** src/components/room/room_surface.rs
- **Verification:** SSR test compile + 4 resolver tests pass; WASM check exits 0.
- **Committed in:** `4ba3f3d` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 compile correctness)
**Impact on plan:** Necessary for the component to compile. No scope creep.

## Issues Encountered
- This plan was first attempted under parallel-worktree execution; that executor was stopped (socket-instability across the wave) with the component draft uncommitted. The draft was salvaged, corrected (E0507), verified, and committed inline.

## User Setup Required
None.

## Next Phase Readiness
- War Room (Phase 27) can consume `crate::components::room::RoomSurface` with real surfaces.
- Verification: `cargo test --features ssr --lib room_surface` → 4 passed; `grep -c Route src/components/room/room_surface.rs` → 0; both compile targets exit 0.

## Self-Check: PASSED
- ARIA roles/attributes present (7 grep hits); 0 `Route` constructs; `use_query_map` + `use_navigate` present.
- Resolver tests pass with dummy surfaces; both targets compile.

---
*Phase: 26-room-system-foundations*
*Completed: 2026-06-04*
