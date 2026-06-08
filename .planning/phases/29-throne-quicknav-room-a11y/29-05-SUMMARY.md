---
phase: 29-throne-quicknav-room-a11y
plan: "05"
subsystem: rooms/throne
tags: [room, throne, growth, blended, redirects, leptos, ssr, wasm]
dependency_graph:
  requires: [29-01, 29-02, 29-03, 29-04]
  provides:
    - ThroneRoom (src/pages/rooms/throne.rs)
    - ThroneSurfaces (src/pages/rooms/throne.rs)
    - get_throne_growth_state (src/pages/rooms/throne.rs)
    - ProfileRedirect (src/pages/rooms/throne.rs)
    - TeamDashboardRedirect (src/pages/rooms/throne.rs)
    - ActionItemsRedirect (src/pages/rooms/throne.rs)
  affects:
    - src/app.rs (/throne route + 3 redirect conversions)
    - src/pages/rooms/mod.rs (pub mod throne)
tech_stack:
  added: []
  patterns:
    - blended-growth-derivation (personal axis BEFORE team check — D-07 divergence)
    - region-read-once (rule #58 — ThroneRoom top level only)
    - dual-redirect-shell (SSR leptos_axum::redirect + hydrate set_href)
    - RoomSurface keep-mounted-tabs (5 panels, no Show wrappers)
    - contrast-scrim-z8 (D-15, mirrors solo_journey.rs:69)
key_files:
  created:
    - src/pages/rooms/throne.rs
  modified:
    - src/pages/rooms/mod.rs
    - src/app.rs
decisions:
  - "get_throne_growth_state computes progression from ranked snapshot BEFORE get_user_team_id match — teamless ranked grinder's room still lights progression overlay (D-05/D-07 lock)"
  - "ThroneSurfaces does NOT use provide_context for shared state — each of the 5 surfaces resolves auth independently; double-fetch of get_team_dashboard is accepted (eager-fetch optimization deferred)"
  - "/throne route added immediately after /analytics to preserve existing route ordering"
requirements_completed: [ROOM-06, A11Y-02]
metrics:
  duration: "15 minutes"
  completed: "2026-06-08"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 2
---

# Phase 29 Plan 05: ThroneRoom Composition Summary

**One-liner:** ThroneRoom assembled — parallax atmosphere with blended team+personal growth tiers (progression-before-team D-07), 5 keep-mounted tabs over IdentitySurface/DashboardSurface/RosterSurface/DevelopmentSurface/SoloSurfaces, and 302-redirect shells for /profile, /team/dashboard, /action-items wired into app.rs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create throne.rs — get_throne_growth_state + ThroneRoom + ThroneSurfaces + 3 redirect shells | 2a54d35 | src/pages/rooms/throne.rs |
| 2 | Wire /throne route + 3 redirects into app.rs; export throne in rooms/mod.rs | 3534c7d | src/app.rs, src/pages/rooms/mod.rs |
| 3 | Checkpoint: human-verify /throne hydration canary + redirects | APPROVED | See verification section |

## What Was Built

### throne.rs (243 lines)

**Three redirect server fns + three redirect shells:**
- `redirect_to_throne_identity` / `ProfileRedirect` → `/throne?tab=identity`
- `redirect_to_throne_dashboard` / `TeamDashboardRedirect` → `/throne?tab=dashboard`
- `redirect_to_throne_development` / `ActionItemsRedirect` → `/throne?tab=development`

All three shells use the dual pattern: SSR `leptos_axum::redirect` + `#[cfg(feature="hydrate")] window.location().set_href(...)`.

**`get_throne_growth_state` — the D-07 blended server fn:**
Computes `ranked = get_latest_ranked_snapshot(...)` FIRST, then the team axes via `match get_user_team_id(...)`. The no-team arm falls to `ThroneGrowthCounts::default()` — it does NOT early-return the whole `ThroneGrowthState`. A teamless ranked grinder's room lights the progression overlay. Auth-gated (rejects unauthenticated with ServerFnError — T-29-05-05).

**`ThroneRoom`:**
- Region read ONCE at top level (`use_context::<InitialTheme>().unwrap_or_default().0`) — never inside reactive primitives (rule #58)
- Three `Signal::derive` tier signals (roster/progression/crown) from the growth Resource
- `AtmosphereRoot` with `roster_tier`/`progression_tier`/`crown_tier` props
- 3 `<Plane>` (sky/mid/near at depth 6/16/32, z 1/2/3)
- 3 `<GrowthOverlay>` (roster/progression/crown at z 4/5/6)
- z-8 contrast scrim (D-15 verbatim from solo_journey.rs:69)
- z-10 surface div with `data-feedback-label="Throne"` wrapping `<ThroneSurfaces region=... />`

**`ThroneSurfaces`:**
- 5-element `Vec<(SurfaceTab, PanelFn)>` in order: identity/dashboard/roster/development/solo
- Only the solo panel clones `region` into `<SoloSurfaces region=r />`; all others are region-agnostic
- No `<Show>` wrappers — `RoomSurface` handles keep-mount + CSS hide

### app.rs + rooms/mod.rs

- `pub mod throne` added to `rooms/mod.rs`
- `rooms::throne::{ActionItemsRedirect, ProfileRedirect, TeamDashboardRedirect, ThroneRoom}` imported
- Removed now-unused imports: `action_items::ActionItemsPage`, `profile::ProfilePage`, `team::dashboard::TeamDashboard`
- `/throne` route added (view=ThroneRoom)
- `/profile` → `ProfileRedirect`, `/team/dashboard` → `TeamDashboardRedirect`, `/action-items` → `ActionItemsRedirect`
- `/team/roster` → `RosterPage` preserved (D-12); `/journey` → `SoloJourneyRoom` preserved (D-13)

## Verification Results

### Automated (pre-checkpoint)

- `cargo check --features ssr`: exit 0
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0
- `grep -c 'use_context::<InitialTheme>' src/pages/rooms/throne.rs`: 1 (rule #58 — single top-level read)
- `grep -c 'Default::default\|ThroneGrowthState::default' src/pages/rooms/throne.rs`: 0 (no early-return Default — D-07)
- `get_latest_ranked_snapshot` at line 118, `get_user_team_id` at line 124 — personal axis BEFORE team check
- All three redirect shells present with correct `?tab=` targets
- `/team/roster` and `/journey` routes unchanged

### Checkpoint: Human-Verify — APPROVED (2026-06-08)

Evidence gathered by orchestrator against the running dev server:

- `/throne` returns HTTP 200
- `e2e/hydration-no-panic.spec.ts` for `/throne`: 2/2 PASS — both [demacia] and [pandemonium] regions hydrate WITHOUT the tachys `html/mod.rs:217` `Option::unwrap()` panic (rule #58 keystone risk cleared; SC4 satisfied)
- All 12 Throne atmosphere WebP assets committed (29-02) and resolve over HTTP 200 (dev-server staleness showing 11/12 as 404 was a stale `target/site/` cache; `public/` dir confirmed correct on disk and in git — not a code defect; production `cargo leptos build` copies all of `public/`)
- `room-contrast.spec.ts` (SC5) and `throne-growth.spec.ts` (SC1/SC1b) remain skip-guarded by design; will be un-skipped and greened in Plan 29-07 (the phase gate)

SC1 through SC5 status:
- SC1/SC1b (growth overlays light): acceptance deferred to 29-07 phase gate; blended server fn correctness confirmed by source assertion
- SC4 (hydration-no-panic canary): PASSED in both regions
- SC5 (contrast scrim WCAG AA): z-8 scrim wired; formal contrast spec un-skipped in 29-07

## Deviations from Plan

None — plan executed exactly as written.

The one implementation judgment call: `ThroneSurfaces` does not use `provide_context` for shared state before the panels vec (unlike `LibrarySurfaces` which provides `selected_match_id`). None of the five Throne surfaces require a cross-surface shared signal at composition time, so the library.rs:299 optimization was correctly omitted.

## Known Stubs

None — `ThroneRoom` renders real data. The parallax planes and growth overlays will show placeholder WebP gradients (from 29-02) until Phase 30 art drops. This is intentional and expected.

## Threat Surface Scan

`get_throne_growth_state` is the one new network endpoint. It:
- Rejects unauthenticated callers (T-29-05-05 mitigated)
- Scopes `get_latest_ranked_snapshot` to the session user's ID only — no cross-user read (T-29-05-02 mitigated)
- Resolves `team_id` server-side via `get_user_team_id` — never client input (T-29-05-01 mitigated)

The three redirect targets are static literals — no open-redirect surface (T-29-05-04 mitigated).

`region` in asset paths comes from `InitialTheme` (server-set cookie), not a URL/query param (T-29-05-03 mitigated).

## Self-Check: PASSED

- FOUND: src/pages/rooms/throne.rs
- FOUND: src/pages/rooms/mod.rs (pub mod throne)
- FOUND: src/app.rs (ThroneRoom route + 3 redirect conversions)
- FOUND: commit 2a54d35
- FOUND: commit 3534c7d
- ThroneRoom, ThroneSurfaces, get_throne_growth_state, ProfileRedirect, TeamDashboardRedirect, ActionItemsRedirect all present in throne.rs
