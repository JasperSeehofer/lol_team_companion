---
phase: 27-war-room
plan: "01"
subsystem: room-ui
tags: [leptos, components, accessibility, refactor, tdd]
dependency_graph:
  requires: []
  provides: [RoomSurface-PanelFn-contract, keep-mounted-panels]
  affects: [src/components/room/room_surface.rs, src/components/room/mod.rs]
tech_stack:
  added: []
  patterns:
    - "keep-mounted tabpanels with hidden=move || Option<&str> (attr:hidden rejected for native elements in Leptos 0.8 view! macro)"
    - "PanelFn type alias Box<dyn Fn() -> AnyView> as per-tab render contract"
    - "derive internal tabs vec from panels before storing in StoredValue"
key_files:
  created: []
  modified:
    - src/components/room/room_surface.rs
    - src/components/room/mod.rs
decisions:
  - "Used hidden=move || Option<&str> (no attr: prefix) on native HTML div — attr: prefix is only valid on Leptos component props, not native elements"
  - "PanelFn closures in unit test use unreachable!() to avoid requiring Leptos runtime context for AnyView construction"
metrics:
  duration_minutes: 12
  completed_date: "2026-06-05"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 2
---

# Phase 27 Plan 01: RoomSurface Keep-Mounted Refactor Summary

Refactored `RoomSurface` from `<Show>`-gated unmount/remount panels to keep-mounted, CSS-hidden panels with a per-tab `PanelFn` render contract (D-06, D-07, D-09). This is the gating foundation change for Phase 27: `WarSurfaces` (Wave 3) depends on the new contract, and keep-mounted panels guarantee zero atmosphere flash on tab switch.

## Tasks

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Replace RoomSurface ChildrenFn API with keep-mounted per-tab PanelFn contract | e127fe3 | room_surface.rs, mod.rs |
| 2 | Update RoomSurface dummy-surface render test to the new contract | b6878ee | room_surface.rs |

## Verification

- `cargo check --features ssr`: 0 errors (2 pre-existing unrelated warnings)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: 0 errors
- `cargo test --features ssr --lib room_surface`: 5 passed, 0 failed
- `grep -c '<Show' src/components/room/room_surface.rs`: 0 (no Show in panel render)
- `pub type PanelFn`: present in room_surface.rs
- `hidden=move || ...`: present on always-mounted tabpanel divs
- `PanelFn`: re-exported from mod.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] attr:hidden rejected for native HTML elements in Leptos 0.8 view! macro**
- **Found during:** Task 1
- **Issue:** Plan specified `attr:hidden=move || ...` on the native `<div role="tabpanel">` element. The `view!` macro rejected this with "expected one of `(`, `)`, `,`, `.`, `::`, `?`, or an operator, found `:`" — `attr:` prefix is only valid on Leptos *component* props (pass-through mechanism), not on native HTML elements.
- **Fix:** Removed the `attr:` prefix; used `hidden=move || if active_idx.get() != idx { Some("") } else { None }` directly. Native HTML element attributes in `view!` accept closures returning `Option<&str>` without the prefix.
- **Files modified:** src/components/room/room_surface.rs
- **Commit:** e127fe3

**2. [Rule 1 - Bug] AnyView::default() does not exist in Leptos 0.8**
- **Found during:** Task 2
- **Issue:** Initial attempt at the dummy-surface test used `leptos::prelude::AnyView::default()` to construct dummy PanelFn closures. `AnyView` has no `Default` impl and construction requires the Leptos renderer context (which is unavailable in `#[cfg(test)]` unit tests).
- **Fix:** Used `Box::new(|| unreachable!())` as the PanelFn closure bodies — they satisfy the type without constructing an `AnyView`. The test validates the type alias compiles and that `resolve_active_tab` resolves correctly against the derived `tab_ids`, which is the plan's stated intent (compile-and-resolve smoke test, not a DOM-render test).
- **Files modified:** src/components/room/room_surface.rs
- **Commit:** b6878ee

## Known Stubs

None — this plan is a presentational/refactor change with no UI data paths.

## Threat Flags

None — no new attack surface (presentational/refactor only, as documented in plan threat model).

## Self-Check: PASSED

- [x] src/components/room/room_surface.rs exists and modified
- [x] src/components/room/mod.rs exists and modified
- [x] Commit e127fe3 exists: `git log --oneline | grep e127fe3`
- [x] Commit b6878ee exists: `git log --oneline | grep b6878ee`
- [x] No `<Show>` in panel render path (grep count = 0)
- [x] `pub type PanelFn` present
- [x] `hidden=move` present
- [x] `PanelFn` re-exported from mod.rs
- [x] 5 unit tests green
