---
phase: 16-phase15-closeout
plan: "02"
subsystem: server/db
tags:
  - dead-code-removal
  - surrealdb
  - test-cleanup
dependency_graph:
  requires: []
  provides:
    - "WR-02 closed: get_personal_goals removed from public API surface"
  affects:
    - src/server/db.rs
    - tests/db_personal_goal.rs
tech_stack:
  added: []
  patterns: []
key_files:
  modified:
    - src/server/db.rs
  deleted:
    - tests/db_personal_goal.rs
decisions:
  - "D-05: Remove over wire-to-caller — compute_goal_progress already covers all production goal reads via batched 3-statement query (Rule 29)"
  - "D-06: No assertion backfill — tests/db_goal_progress.rs absorbs all coverage"
  - "D-07: Separate commit per WR item"
  - "D-08: schema.surql personal_goal table untouched — still used by upsert/delete/compute_goal_progress"
metrics:
  duration_minutes: 5
  completed_date: "2026-05-06"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
  files_deleted: 1
---

# Phase 16 Plan 02: WR-02 Remove Dead get_personal_goals Summary

Dead public API surface removed: `pub async fn get_personal_goals` deleted from `src/server/db.rs`, companion test file deleted entirely. `compute_goal_progress` inlines the equivalent SELECT in its batched 3-statement query per Rule 29 — the standalone function was never called in production.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | c1b6753 | chore(phase-16): WR-02 remove dead get_personal_goals |
| 2 | ba98015 | chore(phase-16): WR-02 delete tests/db_personal_goal.rs |

## Tasks Completed

### Task 1: Delete get_personal_goals from src/server/db.rs

Removed the 33-line `pub async fn get_personal_goals` function (lines 4699–4731) and its inner `DbPersonalGoal` struct from `src/server/db.rs`. The function had zero production callers — `compute_goal_progress` (line 4774, now 4740 after deletion) inlines the equivalent `SELECT id, goal_type, target_value FROM personal_goal WHERE user = ...` in its batched 3-statement query per surreal-patterns.md Rule 29.

Surrounding functions (`upsert_personal_goal`, `delete_personal_goal`, `compute_goal_progress`) and `schema.surql` are byte-identical to pre-change state.

**Verification passed:**
- `cargo check --features ssr` exits 0
- `grep -rn "get_personal_goals" src/` returns 0 hits
- `grep -c "DbPersonalGoal" src/server/db.rs` returns 0
- `grep -n "pub async fn upsert_personal_goal"` returns 1 hit
- `grep -n "pub async fn compute_goal_progress"` returns 1 hit
- `DEFINE TABLE IF NOT EXISTS personal_goal` present in schema.surql

### Task 2: Delete tests/db_personal_goal.rs

Deleted the 83-line integration test file containing 5 tests (`upsert_creates_new_goal`, `upsert_overwrites_existing_goal`, `upsert_different_types_creates_separate_goals`, `cross_user_goal_isolation`, `get_personal_goals_empty_returns_empty_vec`) — all asserting via the now-removed function.

Per D-06, no assertion backfill required. `tests/db_goal_progress.rs` (168 lines, untouched) covers create, overwrite, cross-user isolation, and empty-state via `upsert_personal_goal` → `compute_goal_progress` paths.

**Verification passed:**
- `ls tests/db_personal_goal.rs` → No such file or directory
- `ls tests/db_goal_progress.rs` → file present, 168 lines
- `grep -rn "get_personal_goals" tests/` → 0 hits
- `cargo check --features ssr` exits 0
- `cargo test --features ssr --lib` → 102 passed, 0 failed, 5 ignored

## Must-Haves Verification

| Truth | Status |
|-------|--------|
| `pub async fn get_personal_goals` no longer exists in `src/server/db.rs` | CONFIRMED |
| `tests/db_personal_goal.rs` does not exist | CONFIRMED |
| `grep -rn get_personal_goals src/` returns 0 hits | CONFIRMED |
| `compute_goal_progress` and goal-edit forms continue to work unchanged | CONFIRMED (cargo check passes) |
| `tests/db_goal_progress.rs` continues to pass | CONFIRMED (102 tests pass) |
| `schema.surql personal_goal` table definition is untouched | CONFIRMED (git diff schema.surql empty) |

## Threat Flags

None. This plan only contracts the public API surface (removes a function) and deletes a test file targeting the removed function. No new network endpoints, auth paths, file access patterns, or schema changes were introduced.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- `src/server/db.rs` exists and was modified: FOUND
- `tests/db_personal_goal.rs` deleted: CONFIRMED (No such file)
- `tests/db_goal_progress.rs` untouched: FOUND (168 lines)
- Commit c1b6753 exists: CONFIRMED
- Commit ba98015 exists: CONFIRMED
- `cargo test --features ssr --lib`: 102 passed, 0 failed
