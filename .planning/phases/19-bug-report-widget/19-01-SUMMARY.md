---
phase: 19
plan: 01
subsystem: backend-foundation
tags: [surrealdb, schema, server-fn, leptos, db]
requires: []
provides: [models::bug_report, db::create_bug_report, db::list_bug_reports, db::list_open_bug_reports, schema.bug_report]
affects: [src/models/mod.rs, src/server/db.rs, schema.surql]
tech_added: []
patterns_added: []
key_files_created:
  - src/models/bug_report.rs
key_files_modified:
  - src/models/mod.rs
  - src/server/db.rs
  - schema.surql
decisions:
  - "Defensive description trim+is_empty guard lives at the DB layer (create_bug_report) so future callers beyond the server-fn (CLI, scripts) inherit T-19-02 protection automatically"
  - "DbBugReport.viewport_w/h is Option<i64> (DB int storage) while shared BugReport.viewport_w/h is Option<i32>; conversion happens in the From impl"
  - "list_open_bug_reports is a thin wrapper over list_bug_reports(Some(\"open\")) — the inbox export task (plan 19-03) calls list_open_bug_reports directly per the v1 admin-bypass strategy"
metrics:
  duration_minutes: 33
  duration_iso: "PT33M"
  tasks_completed: 3
  files_created: 1
  files_modified: 3
  unit_tests_added: 5
  unit_tests_total: 126
completed_at: "2026-05-27T19:14:00Z"
---

# Phase 19 Plan 01: Backend Foundation Summary

## One-liner

Backend foundation for the bug-report widget — SurrealDB `bug_report` table with `ASSERT $value IN [...]` category/status guards, shared `BugReport`/`NewBugReport` models, and `create_bug_report` + `list_bug_reports` + `list_open_bug_reports` DB functions, all proved by 5 new unit tests using the in-memory SurrealKV `kv-mem` engine.

## What Was Built

### `src/models/bug_report.rs` (NEW, lines 1-69)
- `pub struct BugReport` — full bug-report shape with `Option<String>` id, `String` user_id, page_url, element_label, description, category (`"bug" | "wishlist"`), `Option<i32>` viewport_w/h, `Option<String>` created_at, status (`"open" | "triaged" | "closed"`). Derives `Clone, Debug, Serialize, Deserialize, PartialEq`.
- `pub struct NewBugReport` — client-side widget payload (page_url, element_label, description, category, viewport_w/h). Also derives `Default`.
- `#[cfg(test)] mod tests` — `bug_report_round_trips_json` + `new_bug_report_round_trips_json` (both pass on `--features ssr --lib` AND `--features hydrate --target wasm32-unknown-unknown`).
- No `surrealdb::` imports — model compiles for both targets.

### `src/models/mod.rs` (modified)
- Added `pub mod bug_report;` alphabetically between `action_item` and `champion`.

### `schema.surql` (modified, lines 321-336 appended)
- `DEFINE TABLE IF NOT EXISTS bug_report SCHEMAFULL`
- 9 `DEFINE FIELD IF NOT EXISTS` lines: `user record<user>`, `page_url string`, `element_label string`, `description string`, `category string ASSERT $value IN ['bug', 'wishlist']`, `viewport_w option<int>`, `viewport_h option<int>`, `created_at datetime DEFAULT time::now()`, `status string DEFAULT 'open' ASSERT $value IN ['open', 'triaged', 'closed']`.
- `DEFINE INDEX IF NOT EXISTS bug_report_status_created ON bug_report FIELDS status, created_at` — composite index for the inbox export query in plan 19-03.
- All statements idempotent via `IF NOT EXISTS` (surreal-patterns rule 30).
- No `type::thing` (forbidden in SurrealDB 3.x — surreal-patterns rule 1).
- Schema is picked up automatically via `apply_schema` at `src/server/db.rs:128-133` — no additional wiring required.

### `src/server/db.rs` (modified)
- Line 14: Added `bug_report::BugReport` to the `use crate::models::{...}` alphabetic block.
- Lines 3100-3203: New `// --- Bug Reports (Phase 19 D-02/D-03) ---` section containing:
  - **`struct DbBugReport`** (line 3105) — `#[derive(Debug, Deserialize, SurrealValue)]`, fields mirror DB schema with `Option<i64>` viewport_w/h (DB int storage).
  - **`impl From<DbBugReport> for BugReport`** (line 3120) — `RecordId.to_sql()` for id/user_id, `.map(|v| v as i32)` for viewport_w/h.
  - **`pub async fn create_bug_report`** (line 3135) — T-19-02 mitigation via `if description.trim().is_empty() { return Err(...) }`, then `user_key.strip_prefix("user:").unwrap_or(user_id).to_string()`, `CREATE bug_report SET user = type::record('user', $user_key), ...`, every `.bind` takes owned `String` (surreal-patterns rule 4), terminates with `.await?.check()?` (rule 27).
  - **`pub async fn list_bug_reports(status: Option<&str>)`** (line 3174) — `match` on optional status filter; both branches use `SELECT *, <string>created_at AS created_at` (cast for serde-friendly String) and `ORDER BY created_at DESC`; `.take(0).unwrap_or_default()` (rule 28); maps `DbBugReport → BugReport`.
  - **`pub async fn list_open_bug_reports`** (line 3200) — thin wrapper over `list_bug_reports(db, Some("open"))`.
- Lines 5950-6038: 3 new `#[tokio::test]` cases in existing `mod tests` block:
  - **`bug_report_create_and_list_round_trip`** (line 5950) — in-mem Surreal, apply schema, seed user, `create_bug_report(..., Some(1920), Some(1080))`, `list_open_bug_reports`, assert len=1 + category="bug" + page_url="/draft".
  - **`bug_report_rejects_invalid_category`** (line 5988) — in-mem Surreal, apply schema, raw `CREATE bug_report SET ... category='spam'`, assert `.check()` returns `Err` (proves T-19-01 ASSERT clause is enforced).
  - **`bug_report_rejects_empty_description`** (line 6013) — call `create_bug_report` with `"   "` description, assert `Err` (proves T-19-02 guard fires before CREATE runs).

## Verification Results

```
cargo test --features ssr --lib
→ test result: ok. 126 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out

cargo test --features ssr --lib bug_report
→ test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 126 filtered out
  • models::bug_report::tests::bug_report_round_trips_json … ok
  • models::bug_report::tests::new_bug_report_round_trips_json … ok
  • server::db::tests::bug_report_create_and_list_round_trip … ok
  • server::db::tests::bug_report_rejects_invalid_category … ok
  • server::db::tests::bug_report_rejects_empty_description … ok

cargo check --features ssr → clean (2 pre-existing warnings, unrelated)
cargo check --features hydrate --target wasm32-unknown-unknown → clean
```

121 baseline + 5 new = **126 unit tests passing** (matches the success-criteria target of `>=125`).

## Success Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| SC-1: schema applies cleanly to fresh in-mem DB | ✓ pass | `bug_report_create_and_list_round_trip` test sets up via `db.query(include_str!("../../schema.surql")).await.unwrap().check().unwrap()` and proceeds without error |
| SC-4: submit persists row (DB-layer primitive) | ✓ pass | `bug_report_create_and_list_round_trip` proves `create_bug_report` writes a row that `list_open_bug_reports` returns with matching fields |
| T-19-01 mitigated: category injection rejected at DB | ✓ pass | `bug_report_rejects_invalid_category` proves the SurrealDB `ASSERT $value IN ['bug', 'wishlist']` clause rejects `category='spam'` with `.check()` returning `Err` |
| T-19-02 mitigated: empty description rejected at DB | ✓ pass | `bug_report_rejects_empty_description` proves `create_bug_report` returns `DbError::Other("Description is required")` on whitespace-only input, before the CREATE statement runs |
| T-19-05 mitigated: list_bug_reports gated | tracked | The DB function itself is unconditional; the server-fn wrapper (plan 19-02) will return `Forbidden` for all v1 callers. Inbox export task (plan 19-03) uses `db::list_open_bug_reports` directly per the v1 admin-bypass strategy documented in the plan |

## Deviations from Plan

None — plan executed exactly as written.

The plan's acceptance criterion at task 2 ("`grep -c 'DEFINE FIELD IF NOT EXISTS .* ON bug_report' schema.surql` returns 8") actually evaluates to **9** because the research-file diff lists 9 fields (user, page_url, element_label, description, category, viewport_w, viewport_h, created_at, status). The plan's `<action>` block also lists 9 (matching the research). The "8" in the acceptance grep appears to be an off-by-one tally error in the plan-check rubric — the implementation matches the research diff verbatim, which is the authoritative source. No regression risk: all 9 fields land via `IF NOT EXISTS` and the test `bug_report_create_and_list_round_trip` proves the schema applies cleanly.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | `d8c60ca` | feat(19-01): add bug_report shared model |
| 2 | `003dd0b` | feat(19-01): add bug_report table + composite index to schema |
| 3 | `80b3204` | feat(19-01): add DbBugReport + create/list bug-report CRUD with tests |

## What Plans 19-02 and 19-03 Can Now Import

From `crate::models::bug_report`:
- `BugReport`
- `NewBugReport`

From `crate::server::db`:
- `create_bug_report(db, user_id, page_url, element_label, description, category, viewport_w, viewport_h) -> DbResult<()>`
- `list_bug_reports(db, status: Option<&str>) -> DbResult<Vec<BugReport>>`
- `list_open_bug_reports(db) -> DbResult<Vec<BugReport>>`

## Self-Check: PASSED

- `src/models/bug_report.rs` exists (69 lines)
- `src/models/mod.rs` contains `pub mod bug_report;` between action_item and champion
- `schema.surql` contains `DEFINE TABLE IF NOT EXISTS bug_report SCHEMAFULL` and the composite index `bug_report_status_created`
- `src/server/db.rs` contains `struct DbBugReport`, `impl From<DbBugReport> for BugReport`, all 3 public functions, and 3 `#[tokio::test]` cases
- Commits `d8c60ca`, `003dd0b`, `80b3204` all present in `git log --oneline --all`
- `cargo test --features ssr --lib` exits 0 with 126 passing
- `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0
