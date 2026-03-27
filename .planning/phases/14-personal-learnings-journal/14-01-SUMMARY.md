---
phase: 14-personal-learnings-journal
plan: "01"
subsystem: data-layer
tags: [personal-learning, surrealdb, model, crud, e2e]
dependency_graph:
  requires: []
  provides: [personal_learning-table, PersonalLearning-model, personal-learning-crud]
  affects: [14-02, 14-03]
tech_stack:
  added: []
  patterns: [DbStruct-to-model-conversion, strip-prefix-key-pattern, serde-default-optional-fields]
key_files:
  created:
    - src/models/personal_learning.rs
  modified:
    - schema.surql
    - src/models/mod.rs
    - src/server/db.rs
    - e2e/tests/pages.spec.ts
decisions:
  - personal_learning is user-scoped (not team-scoped) — matches D-01 decision in STATE.md
  - created_at field on PersonalLearning is Option<String> (not DateTime) to match DB serialization pattern used by other models
metrics:
  duration_minutes: 8
  completed_date: "2026-03-27"
  tasks_completed: 2
  files_changed: 5
---

# Phase 14 Plan 01: Data Layer for Personal Learnings Journal Summary

PersonalLearning SurrealDB table, shared Rust model, 5 CRUD functions, and e2e smoke test scaffolding — all compiling for both SSR and WASM targets.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Schema, model, and DB CRUD functions | d3a6126 | schema.surql, src/models/personal_learning.rs, src/models/mod.rs, src/server/db.rs |
| 2 | Add new routes to e2e smoke test array | eb68ebb | e2e/tests/pages.spec.ts |

## What Was Built

**schema.surql** — Added `personal_learning` table with 15 fields and a `personal_learning_user` index. Table is SCHEMAFULL and user-scoped (not team-scoped, matching the Phase 14 design decision).

**src/models/personal_learning.rs** — `PersonalLearning` struct with all required fields from D-01 through D-14. `LEARNING_TAGS` constant contains 8 predefined category tags (Laning, Teamfighting, Macro/Rotations, Vision, Trading, Wave Management, Objective Control, Mental/Tilt). Two unit tests: round-trip serialization and missing-optional-fields deserialization.

**src/models/mod.rs** — Registered `pub mod personal_learning`.

**src/server/db.rs** — Added `DbPersonalLearning` struct with `SurrealValue` derive, `From<DbPersonalLearning> for PersonalLearning` conversion, and 5 CRUD functions:
- `create_personal_learning` — CREATE with `.check()`, returns `id.to_sql()`
- `get_personal_learning` — SELECT by record ID, returns `Option<PersonalLearning>`
- `list_personal_learnings` — SELECT WHERE user, ORDER BY created_at DESC, `unwrap_or_default()`
- `update_personal_learning` — UPDATE with `.check()`, strips `personal_learning:` prefix
- `delete_personal_learning` — DELETE with `.check()`, strips `personal_learning:` prefix

**e2e/tests/pages.spec.ts** — Added `/personal-learnings` and `/personal-learnings/new` to `AUTHED_PAGES` array. These will fail until Plan 02 creates the page components.

## Verification

- `cargo check --features ssr` — passes (0 errors, 3 pre-existing unused import warnings)
- `cargo check --features hydrate --target wasm32-unknown-unknown` — passes (0 errors)
- `cargo test --features ssr --lib personal_learning` — 2 tests pass

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — this is a pure data layer. No UI components were created.

## Self-Check: PASSED

- schema.surql contains "DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL" — FOUND
- schema.surql contains "DEFINE INDEX IF NOT EXISTS personal_learning_user ON personal_learning FIELDS user" — FOUND
- src/models/personal_learning.rs created — FOUND
- src/models/mod.rs contains "pub mod personal_learning" — FOUND
- src/server/db.rs contains "pub async fn create_personal_learning" — FOUND
- src/server/db.rs contains "pub async fn list_personal_learnings" — FOUND
- e2e/tests/pages.spec.ts contains 2 "personal-learnings" entries — FOUND
- Commits d3a6126, eb68ebb — FOUND
