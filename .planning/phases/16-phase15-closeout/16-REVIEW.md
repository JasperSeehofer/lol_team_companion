---
phase: 16-phase15-closeout
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - src/pages/solo_dashboard.rs
  - src/server/db.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 16: Code Review Report

**Reviewed:** 2026-05-07T00:00:00Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found (2 info items; no critical or warning findings)

## Summary

Two changes were reviewed, both scoped tightly to their stated goals:

**WR-01** (b5930bc) hoists `lp_window` and `lp_history_resource` from `LpHistoryGraph` up to `SoloDashboardPage` and adds `.refetch()` calls on all three resources in both the auto-sync Effect and the `do_sync` handler. The reactive model is correct: `lp_history_resource` is only read via `.get()` inside `LpHistoryGraph`'s own inner `Suspense` (lines 515-530), so the outer `Suspense` (lines 318-342) does not block on it. The `auto_synced` guard uses `.get_untracked()` correctly to prevent re-entrant auto-syncs after `dashboard_resource.refetch()` triggers a second Effect run. `Resource<T>` is `Copy` in Leptos 0.8, so captures of `lp_history_resource` and `goal_progress_resource` into `move` closures and `spawn_local` are sound.

**WR-02** (c1b6753) removes the dead `get_personal_goals` function from `src/server/db.rs` and deletes `tests/db_personal_goal.rs`. No remaining callsites exist (`grep -rn get_personal_goals src/` returns zero hits). `compute_goal_progress` already inlines the equivalent `SELECT` as part of its batched 3-statement query (Rule 29). The removal is clean.

Two info-level observations are noted below; neither is a bug.

## Info

### IN-01: `compute_goal_progress` second query fetches all qualifying matches without a LIMIT

**File:** `src/server/db.rs:4818-4821`
**Issue:** The second statement in the batched query fetches all `player_match` rows for the user in queue 420 with no `LIMIT` clause, then caps to 20 in Rust (line 4835). For most users this is harmless, but as match history grows the DB may return large result sets that are immediately discarded after the first 20.

The comment on line 4812-4813 explains the reason: `ORDER BY match.game_end` is not supported for link-traversal fields in SurrealDB 3.x (Assumption A2). This tradeoff is documented and intentional.

**Fix:** No action required for correctness. If this becomes a performance concern in a future pass, one approach is to add a generous but bounded `LIMIT` (e.g. `LIMIT 100`) to reduce data transfer while still capturing the most recent 20 games with high probability. This is deferred to a performance review phase.

---

### IN-02: `do_sync` closure does not guard against re-entry via double-click after the `syncing` signal resets

**File:** `src/pages/solo_dashboard.rs:267-292`
**Issue:** The `do_sync` handler checks `syncing.get_untracked()` before spawning the async task, and `syncing` remains `true` for the duration of the `sync_solo_matches()` call, preventing re-entry. However, the button's `disabled` prop reads `syncing.get()` reactively (line 307), which is correct. There is no gap here — the guard and the disabled state are consistent. This is a non-issue; listed only to confirm the pattern was checked.

**Fix:** No action required.

---

_Reviewed: 2026-05-07T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
