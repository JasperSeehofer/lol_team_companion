---
phase: 28-library-ia-healing
plan: "05"
subsystem: rooms
tags: [leptos, rust, rooms, library, parallax, growth-tiers, css, webp-assets]

# Dependency graph
requires:
  - phase: 28-01
    provides: LibraryGrowthCounts/LibraryGrowthState + get_library_growth_counts
  - phase: 28-03
    provides: StatsSurface, MatchDetailSurface (with shared selected_match_id consumer)
  - phase: 28-04
    provides: AnalyticsSurface, PostGameSurface, PersonalLearningsSurface
provides:
  - get_library_growth_state server fn (auth-gated, no-team → Ok(default))
  - LibraryRoom page component (parallax AtmosphereRoot, region read once)
  - LibrarySurfaces composer (5-tab RoomSurface + selected_match_id context)
  - /library route in app.rs
  - pub mod library in rooms/mod.rs
  - 9 Library tier→opacity CSS selectors in input.css (records/tomes/analysis)
  - 12 placeholder gradient WebP assets (6 per region)
affects:
  - 28-06 (IA redirect shells — /stats, /analytics, etc. → /library)
  - 28-07 (e2e smoke coverage for /library)
  - Phase 30 (real FLUX art swaps in at same asset paths, no code change)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Library room follows war_room.rs line-for-line: region-read-once, Resource growth fetch, per-axis Signal::derive, AtmosphereRoot + 3 Plane + 3 GrowthOverlay + scrim + z-10 surface layer"
    - "provide_context(selected_match_id) BEFORE view! in LibrarySurfaces — Pitfall 4 prevention for Stats↔Match Detail bridge"
    - "9 tier→opacity CSS selectors: [data-{axis}-tier=\"N\"] .room-overlay--{axis} { opacity: ... } — closes Phase 27 SC3 gap"
    - "ImageMagick convert gradient: for placeholder WebP assets at correct dims (planes 1600x900, overlays 1200x800)"

key-files:
  created:
    - src/pages/rooms/library.rs
    - public/img/rooms/library/demacia/plane-sky.webp
    - public/img/rooms/library/demacia/plane-mid.webp
    - public/img/rooms/library/demacia/plane-near.webp
    - public/img/rooms/library/demacia/overlay-records.webp
    - public/img/rooms/library/demacia/overlay-tomes.webp
    - public/img/rooms/library/demacia/overlay-analysis.webp
    - public/img/rooms/library/pandemonium/plane-sky.webp
    - public/img/rooms/library/pandemonium/plane-mid.webp
    - public/img/rooms/library/pandemonium/plane-near.webp
    - public/img/rooms/library/pandemonium/overlay-records.webp
    - public/img/rooms/library/pandemonium/overlay-tomes.webp
    - public/img/rooms/library/pandemonium/overlay-analysis.webp
  modified:
    - src/pages/rooms/mod.rs
    - src/app.rs
    - input.css

key-decisions:
  - "region prop accepted by LibrarySurfaces but prefixed _region (none of the 5 surfaces need it); keeps parity with WarSurfaces without triggering unused-variable warning"
  - "provide_context(selected_match_id) placed before panels vec and view! in LibrarySurfaces — both keep-mounted Stats and Match Detail panels find signal in context on mount"
  - "9 CSS selectors mirror War Room ramp exactly (0.34 / 0.67 / 1) — closing the identical omission that broke Phase 27 SC3"
  - "Tab IDs: stats/match/analytics/post-game/personal — match Plan 06 redirect targets (?tab=...)"

# Metrics
duration: 25min
completed: 2026-06-06T01:36:48Z
---

# Phase 28 Plan 05: Library Room Composition Summary

**LibraryRoom page over a 5-tab RoomSurface (parallax AtmosphereRoot + get_library_growth_state + 9 input.css tier→opacity selectors + 12 placeholder WebP assets) — the keystone that makes growth visible and turns the growth model into a working room**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-06T01:11:00Z
- **Completed:** 2026-06-06T01:36:48Z
- **Tasks completed (pre-checkpoint):** 3 of 4
- **Files created/modified:** 16 (13 assets + 3 code files + input.css)

## Accomplishments

### Task 1 — Placeholder gradient WebP assets
- Generated 12 WebP assets via ImageMagick `convert gradient:` at the declared paths
- Demacia: blue/gold library tones; pandemonium: red/dark tones; per-axis gradient variation
- All planes 1600x900, all overlays 1200x800 (mirror war-room dims exactly)
- Real FLUX art swaps in at same paths in Phase 30 — zero code change required

### Task 2 — 9 Library tier→opacity CSS selectors (input.css)
- Added records/tomes/analysis × tiers 1/2/3 block immediately after the War Room selectors
- Ramp: tier 1 → 0.34, tier 2 → 0.67, tier 3 → 1.0 (identical to War Room)
- Closes the Phase 27 SC3 gap: `data-*-tier` attrs were wired but CSS was missing → overlays were invisible; now tier≥1 renders a visible overlay, tier-0 stays hidden
- War Room selectors, base `.room-overlay` rule, and reduced-motion block unchanged

### Task 3 — library.rs + mod.rs + app.rs
- `get_library_growth_state` server fn: auth-gated, team-scoped, no-team → `Ok(LibraryGrowthState::default())` (never Err — rule #44)
- `LibraryRoom`: region read ONCE at top (rule #58); Resource growth fetch; 3 `Signal::derive` tier signals; `AtmosphereRoot` + 3 Plane + 3 GrowthOverlay (axis strings `"records"` / `"tomes"` / `"analysis"` — match CSS class targets exactly) + contrast scrim + z-10 surface layer; 172 lines (≤200 target)
- `LibrarySurfaces`: `provide_context(selected_match_id)` BEFORE `view!` (Pitfall 4 prevention — D-07 Stats↔Match Detail bridge); 5-tab `RoomSurface` vec in order Stats/Match Detail/Analytics/Post-Game/Personal Learnings; tab IDs stats/match/analytics/post-game/personal
- `pub mod library` added to `rooms/mod.rs`; `LibraryRoom` imported + `/library` route registered after `/war-room` in `app.rs`
- Both SSR and hydrate compile targets exit 0 with pre-existing warnings only

## Task Commits

1. **Task 1: Placeholder gradient WebP assets** — `3df637b` (chore)
2. **Task 2: 9 Library tier→opacity CSS selectors** — `976d523` (feat)
3. **Task 3: library.rs + mod.rs + app.rs** — `bcb416f` (feat)

## Files Created/Modified

- `src/pages/rooms/library.rs` — get_library_growth_state, LibraryRoom, LibrarySurfaces (created)
- `src/pages/rooms/mod.rs` — added `pub mod library;`
- `src/app.rs` — added LibraryRoom import + `/library` route
- `input.css` — added 9 Library tier→opacity selectors (lines 455-464)
- `public/img/rooms/library/demacia/*.webp` — 6 placeholder assets (created)
- `public/img/rooms/library/pandemonium/*.webp` — 6 placeholder assets (created)

## Checkpoint Reached (Task 4)

Plan 28-05 has a `type="checkpoint:human-verify"` task — execution paused here. The implementation is complete; human verification is pending for:
- `/library` renders five tabs over a stable parallax atmosphere in both themes
- Stats→Match Detail bridge (click match row → switches to Match Detail tab, no full-page nav)
- Growth overlays visibly tier up (confirming Task 2 CSS selectors are wired)
- No tachys hydration panic in pandemonium theme

## Deviations from Plan

None — plan executed exactly as written. All three auto tasks completed before the checkpoint.

## Known Stubs

The 12 WebP assets are intentional placeholder gradients. They render as colored gradient rectangles, not thematic art. Real FLUX environment art replaces them at the same paths in Phase 30 — no code change required. This is documented in the plan as the intended Phase 30 deliverable.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes beyond what the plan's threat model covers. `get_library_growth_state` follows the exact T-28-05-01 mitigation (auth + team-scoping, no raw client team id accepted, aggregate counts only).

## Self-Check: PASSED

- `src/pages/rooms/library.rs` — FOUND
- `input.css` (9 Library selectors) — FOUND
- `public/img/rooms/library/demacia/plane-sky.webp` — FOUND
- `public/img/rooms/library/pandemonium/overlay-analysis.webp` — FOUND
- Commit `3df637b` (assets) — FOUND
- Commit `976d523` (CSS selectors) — FOUND
- Commit `bcb416f` (library.rs) — FOUND
