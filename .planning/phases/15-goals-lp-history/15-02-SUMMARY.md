---
phase: "15"
plan: "02"
subsystem: solo-dashboard
tags: [lp-history, goals, svg-chart, server-fns, leptos]
dependency_graph:
  requires: [15-01]
  provides: [lp-history-graph, goal-cards]
  affects: [src/pages/solo_dashboard.rs]
tech_stack:
  added: [SvgsvgElement/SvgGraphicsElement/DomRect web-sys features, chrono window_to_cutoff]
  patterns: [SVG polyline chart with hover tooltip, goal CRUD cards with inline editing, Resource refetch after mutation]
key_files:
  modified:
    - src/pages/solo_dashboard.rs
    - Cargo.toml
decisions:
  - Used SvgsvgElement cast (not Element) for get_bounding_client_rect — requires web-sys features DomRect + SvgsvgElement + SvgGraphicsElement
  - SkeletonGrid IS available in ui.rs; used SkeletonCard instead per plan guidance for GoalCards fallback (simpler, sufficient)
  - Single file rewrite meant both Task 1 and Task 2 landed in one commit (deviation documented below)
  - get_untracked() used in on_save handlers per leptos-patterns rule 20
metrics:
  duration: "~12 min"
  completed: "2026-05-05"
  tasks_completed: 2
  files_modified: 2
---

# Phase 15 Plan 02: Goals & LP History — Solo Dashboard Summary

LP history graph and live goal cards wired into the solo dashboard, replacing the `GoalPlaceholders` stub component.

## Server Functions Added

| Function | Location | Purpose |
|---|---|---|
| `get_lp_history(window: String)` | solo_dashboard.rs:130 | Fetches ranked snapshots with optional time window cutoff |
| `compute_goal_progress()` | solo_dashboard.rs:142 | Returns GoalProgressPayload with rank/cs/deaths progress |
| `upsert_personal_goal(goal_type, target_value)` | solo_dashboard.rs:153 | Creates/updates a personal goal with server-side validation |
| `window_to_cutoff(window)` | solo_dashboard.rs:186 | Helper: converts "7d"/"30d"/"90d" to RFC3339 cutoff string |

## Components Added

| Component | Approx. Line | Purpose |
|---|---|---|
| `LpHistoryGraph` | ~417 | Container with 7d/30d/90d/All-time pill selector + Suspense |
| `LpGraphSvg` | ~455 | SVG polyline chart with area fill, tier grid lines, date labels, hover tooltip |
| `GoalCards` | ~700 | Suspense wrapper dispatching to 3 goal card sub-components |
| `RankTargetCard` | ~720 | Rank target goal: tier+division selects, progress bar, LP-to-go |
| `CsGoalCard` | ~820 | CS/min goal: number input, progress bar, avg display |
| `DeathsGoalCard` | ~900 | Deaths/game goal: number input, progress bar, avg display |

## Dashboard Render Order

1. Header + Sync Button
2. `RankedBadgeSection` — current rank badge
3. `LpHistoryGraph` — LP over time SVG chart (NEW)
4. `MatchListSection` — recent match list with queue filter
5. `GoalCards` — rank/CS/deaths goals with inline editing (replaces GoalPlaceholders)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] web_sys::Element does not expose get_bounding_client_rect in WASM**
- **Found during:** WASM hydrate compile check
- **Issue:** `el.closest("svg")` returns `Option<Element>`, but `Element` in web-sys 0.3 doesn't expose `get_bounding_client_rect()` — that method requires `SvgsvgElement` (via `SvgGraphicsElement` trait).
- **Fix:** Added `.and_then(|e| e.dyn_into::<web_sys::SvgsvgElement>().ok())` cast after `closest()`. Added `SvgsvgElement`, `SvgGraphicsElement`, `DomRect` to web-sys features in Cargo.toml.
- **Files modified:** `Cargo.toml`, `src/pages/solo_dashboard.rs`
- **Commit:** e129ec3

**2. [Deviation] Single commit for both tasks**
- **Reason:** Both Task 1 (server fns + LpHistoryGraph) and Task 2 (GoalCards) were written as a complete file rewrite in one pass. Since both were verified compiling before any commit, staging them separately after the fact would produce an empty second commit (no file changes remain). Both components are captured in commit e129ec3.
- **Impact:** None — all code is committed and verified.

### Notes

- `window_to_cutoff` shows a dead_code warning in the hydrate target — expected, since it's called only from `#[server]` fns which compile SSR-only. Not suppressed (not a breaking warning).
- `snaps_for_hover` and `points_for_hover` show unused-variable warnings in SSR — expected, they're used only inside `#[cfg(feature = "hydrate")]` blocks. Per wasm-patterns rule 43, could add `#[allow(unused_variables)]` but not required for compilation.
- `get_untracked()` used in all `on_save` handlers per leptos-patterns rule 20.

## Known Stubs

None. All three goal cards wire live data via `compute_goal_progress` resource. LP chart wires live data via `get_lp_history` resource. Empty states are shown when data is absent (not stubs).

## Checkpoint

Auto-approved (--auto mode). No human verification step in this plan.

## Self-Check: PASSED

- `src/pages/solo_dashboard.rs` — exists and modified
- `Cargo.toml` — exists and modified  
- Commit e129ec3 — confirmed in git log
- Both `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` pass with no errors
