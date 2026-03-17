---
phase: 01-pipeline-ctas
plan: 01
subsystem: database
tags: [surrealdb, rust, leptos, server-functions, tdd]

# Dependency graph
requires: []
provides:
  - "get_draft_for_prefill: fetch draft + actions by ID from SurrealDB"
  - "get_game_plans_for_draft: query game plans by draft FK string"
  - "get_draft_for_prefill server fn: auth-gated callable from client/WASM"
  - "check_draft_has_game_plan server fn: returns first plan ID for draft or None"
affects:
  - 01-pipeline-ctas/01-02
  - 01-pipeline-ctas/01-03

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Batch query pattern: SELECT draft + SELECT draft_action in single .query() call for prefill"
    - "FK string match pattern: game_plan.draft is option<string> storing full record ID (e.g. draft:abc123), queried with WHERE draft = $draft_id"

key-files:
  created:
    - tests/db_game_plan_pipeline.rs
  modified:
    - src/server/db.rs
    - src/pages/game_plan.rs

key-decisions:
  - "game_plan.draft field is option<string> (not a record reference) — query with full string match WHERE draft = $draft_id"
  - "get_draft_for_prefill uses batch query (draft + draft_action in one round-trip) consistent with list_drafts pattern"
  - "check_draft_has_game_plan returns Option<String> (first plan ID) rather than bool, so callers can link directly to the existing plan"

patterns-established:
  - "Pipeline query functions follow existing DbStruct->ModelStruct conversion pattern in db.rs"
  - "Server functions wrapping pipeline queries use auth guard (_user) but no team scope — draft_id already came from user's own context"

requirements-completed:
  - PIPE-01

# Metrics
duration: 9min
completed: 2026-03-14
---

# Phase 1 Plan 01: Pipeline DB Queries Summary

**Two SurrealDB query functions powering the draft-to-game-plan pipeline: prefill lookup (draft + actions in one batch) and game plan FK lookup by draft ID string, both covered by 4 passing integration tests and exposed as auth-gated server functions**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-14T01:48:29Z
- **Completed:** 2026-03-14T01:57:10Z
- **Tasks:** 2
- **Files modified:** 3 (+ 1 created)

## Accomplishments
- `get_draft_for_prefill`: batch query fetches a single draft with all its actions in one round-trip, returns `None` if not found
- `get_game_plans_for_draft`: queries game plans whose `draft` field (an `option<string>` FK) matches the given draft ID string
- 4 integration tests (TDD) verified all behavior — both found and not-found cases for both functions
- Two Leptos `#[server]` functions in `game_plan.rs` expose the queries to client code with auth gating

## Task Commits

Each task was committed atomically:

1. **Task 1 (TDD RED): Failing test file** - `d7478d8` (test)
2. **Task 1 (TDD GREEN): DB query implementation** - `fa3e4fa` (feat)
3. **Task 2: Server functions in game_plan.rs** - `0c24657` (feat)

## Files Created/Modified
- `tests/db_game_plan_pipeline.rs` - 4 integration tests for the two new pipeline queries
- `src/server/db.rs` - Added `get_draft_for_prefill` and `get_game_plans_for_draft` functions
- `src/pages/game_plan.rs` - Added `get_draft_for_prefill` and `check_draft_has_game_plan` server functions

## Decisions Made
- `game_plan.draft` is `option<string>` in schema (not a `record<draft>` reference), so the FK is a full record ID string (`"draft:abc123"`). Query uses direct string equality: `WHERE draft = $draft_id` with the full string bound.
- `check_draft_has_game_plan` returns `Option<String>` (the first plan's ID) rather than a boolean, enabling callers to link directly to the existing plan for duplicate detection.
- No team-scope guard on `get_draft_for_prefill` server fn — the draft ID will always come from the user's own team's draft list in the CTA context.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing OOM during linking of the main binary (`cargo test --test db_game_plan_pipeline` triggers main binary link which OOMs). The test binary itself compiled and ran successfully (confirmed by running the pre-built test binary directly: 4/4 tests passed). The OOM issue is pre-existing per MEMORY.md and does not affect the new code.
- Pre-existing `db_drafts.rs` test file has argument count mismatches with the current `save_draft` signature — these are pre-existing failures, not caused by this plan's changes.

## Next Phase Readiness
- `get_draft_for_prefill` and `check_draft_has_game_plan` server functions are ready for use in Plan 02 (Draft page CTA button) and Plan 03 (Game Plan page back-reference badge).
- The FK string match pattern (`WHERE draft = $draft_id`) is confirmed working and documented.

## Self-Check: PASSED

- FOUND: tests/db_game_plan_pipeline.rs
- FOUND: src/server/db.rs
- FOUND: src/pages/game_plan.rs
- FOUND commit d7478d8 (test: failing tests)
- FOUND commit fa3e4fa (feat: db implementation)
- FOUND commit 0c24657 (feat: server functions)

---
*Phase: 01-pipeline-ctas*
*Completed: 2026-03-14*
