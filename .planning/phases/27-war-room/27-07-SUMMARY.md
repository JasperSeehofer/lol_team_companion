---
phase: 27-war-room
plan: "07"
subsystem: rooms/war-room
tags: [leptos, rooms, parallax, atmosphere, tdd, integration, region-threading]
dependency_graph:
  requires: [27-01, 27-02, 27-04, 27-05, 27-06]
  provides: [WarRoomRoom, WarSurfaces, get_team_growth_state, war-room-route, war-room-assets]
  affects:
    - src/pages/rooms/war_room.rs
    - src/pages/rooms/mod.rs
    - src/app.rs
    - public/img/rooms/war-room/
tech_stack:
  added: []
  patterns:
    - "extract-then-compose integration: WarSurfaces composes all 5 *Surface components via RoomSurface PanelFn contract (D-01)"
    - "region-read-once at WarRoomRoom top level, threaded as plain String prop to WarSurfaces → DraftSurface/TreeDrafterSurface (rule #58)"
    - "get_team_growth_state: no-team → Ok(TeamGrowthState::default()) per rule #44; never Err on absent optional data"
    - "ImageMagick gradient WebP placeholders: planes 1600x900, overlays 1200x800, real art Phase 30 at same paths"
key_files:
  created:
    - src/pages/rooms/war_room.rs
    - public/img/rooms/war-room/demacia/plane-sky.webp
    - public/img/rooms/war-room/demacia/plane-mid.webp
    - public/img/rooms/war-room/demacia/plane-near.webp
    - public/img/rooms/war-room/demacia/overlay-tactics.webp
    - public/img/rooms/war-room/demacia/overlay-intel.webp
    - public/img/rooms/war-room/demacia/overlay-command.webp
    - public/img/rooms/war-room/pandemonium/plane-sky.webp
    - public/img/rooms/war-room/pandemonium/plane-mid.webp
    - public/img/rooms/war-room/pandemonium/plane-near.webp
    - public/img/rooms/war-room/pandemonium/overlay-tactics.webp
    - public/img/rooms/war-room/pandemonium/overlay-intel.webp
    - public/img/rooms/war-room/pandemonium/overlay-command.webp
  modified:
    - src/pages/rooms/mod.rs
    - src/app.rs
decisions:
  - "region threaded via plain String prop (never reactive closure) per CLAUDE.md rule #58 — exactly one use_context::<InitialTheme> read in war_room.rs"
  - "WarSurfaces only clones region twice (for DraftSurface + TreeDrafterSurface); GamePlanSurface/OpponentsSurface/BanPrioritySurface take no region"
  - "Checkpoint auto-approved per pre-authorization: SSR exit 0, WASM exit 0, 148 unit tests pass, 12 WebP assets non-empty"
  - "Definitive browser canary (parallax render + ARIA tabs + hydration-no-panic in both regions) deferred to Wave 4 (plan 27-08)"
metrics:
  duration_minutes: 12
  completed_date: "2026-06-05"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 15
---

# Phase 27 Plan 07: War Room Integration Summary

One-liner: War Room keystone — `WarRoomRoom` (parallax `AtmosphereRoot` + `get_team_growth_state`) + `WarSurfaces` (5-tab `RoomSurface` over all extracted surfaces) + `/war-room` route + 12 placeholder gradient WebP assets for both regions.

## Tasks

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Generate 12 placeholder gradient WebP assets (both regions) | 4da67d3 | 12 WebP files |
| 2 | Create war_room.rs + wire /war-room route | e269a08 | war_room.rs, mod.rs, app.rs |
| 3 | Checkpoint: auto-verified per pre-authorization | — | — |

## Checkpoint Auto-Verification Evidence

Pre-authorized auto-verification per `checkpoint_preauthorization`. All checks passed:

- `cargo check --features ssr`: exit 0 (4 pre-existing warnings, unrelated to this plan)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0 (3 pre-existing warnings)
- `cargo test --features ssr --lib`: **148 passed, 0 failed**, 5 ignored
- 12 WebP assets: present and non-empty (39,446 bytes total)

The definitive browser canary (parallax render + ARIA tabs + hydration-no-panic in both regions) runs in Wave 4 (plan 27-08 e2e suite).

## Structural Assertions

- `pub fn WarRoomRoom()`: present in war_room.rs
- `pub async fn get_team_growth_state()`: present; no-team branch returns `Ok(TeamGrowthState::default())`
- `fn WarSurfaces(region: String)`: present, 5 panels in order Draft / Tree-Drafter / Game Plan / Opponents / Ban Priority
- `use_context::<InitialTheme>`: exactly **1** read in war_room.rs (at WarRoomRoom top level, rule #58)
- `pub mod war_room`: present in src/pages/rooms/mod.rs
- `/war-room` route: registered in src/app.rs after `/journey`
- 12 WebP assets: 6 per region, correct dims (planes 1600x900, overlays 1200x800)

## Security (Threat Model)

| Threat | Disposition | Evidence |
|--------|-------------|---------|
| T-27-07-01: get_team_growth_state auth + team-scoping | mitigated | leptos_axum::extract::<AuthSession>() rejects unauthenticated; team resolved server-side; no-team → Ok(default) never leaks |
| T-27-07-02: asset path region interpolation | mitigated | region from server-set InitialTheme context (fixed enum-like string), never a URL param |

## Deviations from Plan

None — plan executed exactly as written. The composition follows the solo_journey.rs template precisely with War Room axes (tactics/intel/command) substituted for Solo axes (library/armory/banner).

## Known Stubs

- **Placeholder WebP assets** — all 12 are gradient-fill placeholders. Real FLUX art swaps in at the same paths in Phase 30 (no code change required). This is intentional and documented in the plan (D-11).

## Threat Flags

None — no new attack surface beyond what is documented and mitigated in the threat model above.

## Self-Check: PASSED

- [x] src/pages/rooms/war_room.rs exists with all 3 items (WarRoomRoom, WarSurfaces, get_team_growth_state)
- [x] src/pages/rooms/mod.rs contains `pub mod war_room`
- [x] src/app.rs contains `/war-room` route
- [x] 12 WebP assets present and non-empty
- [x] Commit 4da67d3 exists (WebP assets)
- [x] Commit e269a08 exists (war_room.rs + route wiring)
- [x] `use_context::<InitialTheme>` count = 1 (rule #58 enforced)
- [x] SSR: exit 0; hydrate/wasm: exit 0; 148 unit tests pass
