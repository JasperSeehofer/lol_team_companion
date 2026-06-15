---
phase: 30-art-production
plan: "02"
subsystem: rooms/growth-overlays
tags: [rooms, growth, css, e2e, art-production, ROOM-03, D-10]
dependency_graph:
  requires: [30-01]
  provides: [per-tier-overlay-markup, per-tier-css, placeholder-webps, seed-and-assert-e2e]
  affects: [30-05, 30-06]
tech_stack:
  added: []
  patterns:
    - "axis=\"{axis}--tier-{N}\" encodes tier in GrowthOverlay prop to produce distinct CSS class room-overlay--{axis}--tier-{N}"
    - "36 discrete CSS opacity:1 rules replace 36 opacity-ramp rules (base opacity:0 covers tier-0)"
    - "seedSoloTier1Growth: personal_learning >= 1 drives lib axis to tier 1 (threshold [1,8,25])"
key_files:
  created:
    - path: public/img/rooms/*/demacia/overlay-*-{1,2,3}.webp
      note: "72 transparent placeholder WebPs (4 rooms × 2 regions × 3 axes × 3 tiers)"
    - path: .planning/phases/30-art-production/30-02-SUMMARY.md
  modified:
    - src/pages/rooms/solo_journey.rs
    - src/pages/rooms/war_room.rs
    - src/pages/rooms/library.rs
    - src/pages/rooms/throne.rs
    - input.css
    - e2e/tests/fixtures.ts
    - e2e/tests/room-solo-journey.spec.ts
decisions:
  - "[30-02] CSS class naming: room-overlay--{axis}--tier-{N} (e.g. room-overlay--lib--tier-1). Plans 05/06 generate art at overlay-{axis}-{N}.webp paths."
  - "[30-02] seedSoloTier1Growth seeds lib axis via one personal_learning record at /personal-learnings/new (threshold [1,8,25])."
  - "[30-02] Per-tier e2e guard: reads data-lib-tier from .room-atmosphere, asserts active tier opacity > 0 and other two tiers = 0."
metrics:
  duration_minutes: 12
  completed_date: "2026-06-15"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 7
  files_created: 73
---

# Phase 30 Plan 02: Per-Tier Overlay Markup + CSS + E2E Summary

**One-liner:** Discrete per-tier overlay system using `axis="lib--tier-1"` GrowthOverlay calls + CSS data-attr toggling + 72 placeholder WebPs + seed-and-assert e2e test closing the Phase 27 silent-fail gap.

## What Was Built

### Task 1 — Per-tier overlay markup in all 4 rooms + per-tier CSS

All four room components (`solo_journey.rs`, `war_room.rs`, `library.rs`, `throne.rs`) had their 3 single-axis `GrowthOverlay` calls replaced with 9 calls (3 per axis). The `axis` prop encodes the tier number using double-dash notation: `axis="lib--tier-1"` produces CSS class `room-overlay--lib--tier-1`. This makes each tier's image a distinct, unconditionally-rendered DOM node — ROOM-03 compliance verified.

`input.css` had its 36 opacity-ramp rules (`opacity: 0.34/0.67/1`) replaced with 36 discrete per-tier rules (`opacity: 1` for the matching tier only). The base `.room-overlay { opacity: 0 }` covers tier-0 (all hidden) without any explicit zero rules.

Both compile targets verified clean:
- `cargo check --features ssr` — 0 errors
- `cargo check --features hydrate --target wasm32-unknown-unknown` — 0 errors

Acceptance criteria verified:
- 9 per-tier overlay srcs per room (all 4 rooms)
- 12 `room-overlay--*--tier-1` rules in input.css (one per axis)
- `data-lib-tier` present, no `data-lib-tier="0"` rule
- Zero reactive tier branching (`if.*tier` / `match.*tier`) across all 4 room files

### Task 2 — Placeholder WebPs

72 transparent placeholder WebPs generated via ImageMagick `convert` (`xc:none`, 256x144, WebP q68, each <1 KB). All 4 rooms × 2 regions × 3 axes × 3 tiers covered at `overlay-{axis}-{N}.webp` paths. The 48 old dead single-overlay files (`overlay-{axis}.webp`) deleted. Each region directory now holds exactly 12 WebPs (3 planes + 9 per-tier overlays). No file exceeds 120 KB.

### Task 3 — Seed-and-assert per-tier overlay e2e test

`seedSoloTier1Growth()` fixture added to `fixtures.ts`: navigates to `/personal-learnings/new`, fills the three required textareas ("What happened", "What I learned", "Next time I will"), clicks "Save learning". One `personal_learning` row drives the `lib` axis to tier 1 (threshold `[1, 8, 25]`).

Test `"per-tier overlay: correct tier visible, others hidden"` added to `room-solo-journey.spec.ts`: seeds, navigates `/journey`, reads `data-lib-tier` from `.room-atmosphere`, asserts the active tier's overlay has computed `opacity > 0` and the other two tiers of the same axis have `opacity === 0`. Guarded on `libTier >= 1` so tier-0 environments don't false-fail.

Test result: **1/1 green**. Hydration-no-panic suite: all rooms including `/journey` pass in both demacia and pandemonium regions.

## CSS Class Naming Reference for Plans 05/06

Art generation in Plans 05/06 must produce images at these exact paths. The CSS class link is:

| Axis | CSS selector | Asset path |
|------|-------------|------------|
| `lib--tier-1` | `[data-lib-tier="1"] .room-overlay--lib--tier-1` | `overlay-lib-1.webp` |
| `lib--tier-2` | `[data-lib-tier="2"] .room-overlay--lib--tier-2` | `overlay-lib-2.webp` |
| `lib--tier-3` | `[data-lib-tier="3"] .room-overlay--lib--tier-3` | `overlay-lib-3.webp` |
| `banner--tier-N` | `[data-banner-tier="N"] .room-overlay--banner--tier-N` | `overlay-banner-N.webp` |
| `armory--tier-N` | `[data-armory-tier="N"] .room-overlay--armory--tier-N` | `overlay-armory-N.webp` |
| (war-room) `tactics--tier-N` | `[data-tactics-tier="N"] .room-overlay--tactics--tier-N` | `overlay-tactics-N.webp` |
| (war-room) `intel--tier-N` | `[data-intel-tier="N"] .room-overlay--intel--tier-N` | `overlay-intel-N.webp` |
| (war-room) `command--tier-N` | `[data-command-tier="N"] .room-overlay--command--tier-N` | `overlay-command-N.webp` |
| (library) `records--tier-N` | `[data-records-tier="N"] .room-overlay--records--tier-N` | `overlay-records-N.webp` |
| (library) `tomes--tier-N` | `[data-tomes-tier="N"] .room-overlay--tomes--tier-N` | `overlay-tomes-N.webp` |
| (library) `analysis--tier-N` | `[data-analysis-tier="N"] .room-overlay--analysis--tier-N` | `overlay-analysis-N.webp` |
| (throne) `roster--tier-N` | `[data-roster-tier="N"] .room-overlay--roster--tier-N` | `overlay-roster-N.webp` |
| (throne) `progression--tier-N` | `[data-progression-tier="N"] .room-overlay--progression--tier-N` | `overlay-progression-N.webp` |
| (throne) `crown--tier-N` | `[data-crown-tier="N"] .room-overlay--crown--tier-N` | `overlay-crown-N.webp` |

Seed axis for e2e test: **lib** (driven by `personal_learning`, lowest-friction UI path).

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None. The `region` source (server-set `InitialTheme` cookie, not URL params) was not changed. ROOM-03 unconditional DOM structure verified. No new network endpoints introduced.

## Self-Check: PASSED

Files verified:
- `src/pages/rooms/solo_journey.rs` — FOUND (9 per-tier overlays)
- `src/pages/rooms/war_room.rs` — FOUND (9 per-tier overlays)
- `src/pages/rooms/library.rs` — FOUND (9 per-tier overlays)
- `src/pages/rooms/throne.rs` — FOUND (9 per-tier overlays)
- `input.css` — FOUND (36 discrete per-tier rules, no opacity-ramp)
- `e2e/tests/fixtures.ts` — FOUND (seedSoloTier1Growth)
- `e2e/tests/room-solo-journey.spec.ts` — FOUND (per-tier overlay test)
- 72 placeholder WebPs — FOUND (`find public/img/rooms -name "overlay-*-[123].webp" | wc -l` = 72)

Commits verified:
- `db4cb2f` feat(30-02): per-tier overlay markup
- `1e537a7` chore(30-02): placeholder WebPs
- `fbbd87c` test(30-02): seed-and-assert e2e test
