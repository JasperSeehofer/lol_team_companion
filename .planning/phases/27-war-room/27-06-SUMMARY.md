---
phase: 27-war-room
plan: "06"
subsystem: pages/game_plan, pages/opponents
tags: [surface-extraction, refactor, war-room, leptos]
dependency_graph:
  requires: [27-01, 27-02, 27-03]
  provides: [GamePlanSurface, OpponentsSurface]
  affects: [src/pages/game_plan.rs, src/pages/opponents.rs]
tech_stack:
  added: []
  patterns: [extract-then-compose, chrome-free-surface, thin-page-shell]
key_files:
  modified:
    - src/pages/game_plan.rs
    - src/pages/opponents.rs
decisions:
  - "OpponentsPage is a zero-chrome pass-through (view! { <OpponentsSurface /> }) because the surface's gate_view renders its own canvas-grain div; wrapping in another canvas-grain would create nested chrome and change the DOM structure (D-02 violation)"
  - "Neither GamePlanSurface nor OpponentsSurface takes a region prop — confirmed zero InitialTheme reads in both files"
metrics:
  duration: "8 minutes"
  completed: "2026-06-05T17:41:01Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 27 Plan 06: GamePlanSurface + OpponentsSurface Extraction Summary

One-liner: Chrome-free `GamePlanSurface` and `OpponentsSurface` extracted from ~3,300 lines across two pages; both flat routes preserved verbatim; neither surface requires a region prop.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Extract GamePlanSurface; reduce GamePlanPage to thin shell | 50c60f7 | src/pages/game_plan.rs |
| 2 | Extract OpponentsSurface; reduce OpponentsPage to thin shell | 8b4eadb | src/pages/opponents.rs |
| 3 | Checkpoint: verify /game-plan and /opponents behavior (auto-approved) | — | — |

## What Was Built

**GamePlanSurface** (`src/pages/game_plan.rs:523`): Contains the full game-plan UI chrome-free — auth-redirect Effect, `?draft_id` prefill resource, all editor signals (plan name, champion picks, win conditions, role strats, checklist), `strategy_win_rates`/`opponent_win_rates` resources, the `clear_editor`/`load_plan`/`do_delete`/`do_save` callbacks, and the complete `view!` content. No region prop.

**GamePlanPage** (`src/pages/game_plan.rs:1670`): Thin shell — renders `<GamePlanSurface />` inside `<div class="canvas-grain bg-base min-h-screen" data-feedback-label="Game plan">`.

**OpponentsSurface** (`src/pages/opponents.rs:367`): Contains the full opponents UI chrome-free — auth-redirect + mode-detection Effect, `is_solo_mode` signal, `ToastContext`, `return_to`/`return_draft_id` query params, `has_team` resource, `opponents`/`detail` resources, `on_delete` callback, `gate_view` (solo-mode canvas-grain), and the main content div with `style:display` solo-mode toggle. No region prop.

**OpponentsPage** (`src/pages/opponents.rs:619`): Zero-chrome pass-through — `view! { <OpponentsSurface /> }`. No extra canvas-grain wrapper added because the surface's `gate_view` and main content div already provide their own chrome; wrapping would nest canvas-grain divs and change the DOM structure.

## Verification Evidence (Checkpoint Auto-Approved)

- `cargo check --features ssr`: exit 0 (0 errors, 4 warnings — pre-existing)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0 (0 errors, 3 warnings — pre-existing)
- `cargo test --features ssr --lib`: 148 passed, 0 failed, 5 ignored
- Definitive browser regression runs deferred to Wave 4 (plan 27-08) per pre-authorization

## Deviations from Plan

**1. [Rule 1 - Design] OpponentsPage is a zero-chrome pass-through instead of a canvas-grain-wrapping shell**

- **Found during:** Task 2 analysis
- **Issue:** The plan suggested wrapping `<OpponentsSurface />` in `<div class="canvas-grain bg-base min-h-screen" data-feedback-label="Opponents">`. However, `OpponentsSurface`'s `gate_view` (solo mode) renders its own full `canvas-grain bg-base min-h-screen` div as a top-level sibling to the main content div. Adding an outer canvas-grain wrapper would nest these, changing the DOM structure and violating D-02 (behavior preserved).
- **Fix:** `OpponentsPage` is `view! { <OpponentsSurface /> }` with no extra wrapper.
- **Files modified:** src/pages/opponents.rs
- **Commit:** 8b4eadb

## Known Stubs

None. Both surfaces render live data from existing server functions unchanged.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. This is a pure code-move refactor — existing server functions and auth/team-scoping logic carried verbatim into the surfaces (T-27-06-01: accepted, existing access control preserved).

## Self-Check

- [x] `src/pages/game_plan.rs` exists and contains `pub fn GamePlanSurface`
- [x] `src/pages/opponents.rs` exists and contains `pub fn OpponentsSurface`
- [x] Commit 50c60f7 exists (GamePlanSurface)
- [x] Commit 8b4eadb exists (OpponentsSurface)
- [x] Both compile targets: exit 0
- [x] 148 unit tests: passed

## Self-Check: PASSED
