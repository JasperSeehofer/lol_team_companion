---
phase: 15-goals-lp-history
reviewed: 2026-05-05T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - Cargo.toml
  - schema.surql
  - src/models/match_data.rs
  - src/pages/solo_dashboard.rs
  - src/pages/stats.rs
  - src/server/db.rs
  - tests/db_champion_trends.rs
  - tests/db_goal_progress.rs
  - tests/db_lp_history.rs
  - tests/db_personal_goal.rs
findings:
  critical: 2
  warning: 2
  info: 3
  total: 7
status: issues_found
---

# Phase 15: Code Review Report

**Reviewed:** 2026-05-05
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 15 introduced ranked snapshot storage, LP history graph, personal goals with progress cards, and a champion trends table. The DB logic and model layer are clean and well-structured. The SSR target compiles cleanly. However there are two blockers: a variable-naming mistake causes the WASM/hydrate target to fail to compile (the LP graph hover is completely broken in the browser), and a SurrealDB Rule 40 violation in `get_latest_ranked_snapshot` will silently return wrong data in production. Two additional warnings cover a stale LP graph after sync and an unused public function.

---

## Critical Issues

### CR-01: WASM compile failure — LP graph hover uses undeclared variable names

Status: FIXED in 5902a81 — underscore-prefix issue resolved by renaming snaps_for_hover/points_for_hover.

**File:** `src/pages/solo_dashboard.rs:564-565`
**Issue:** `_snaps_for_hover` and `_points_for_hover` are declared with a leading underscore (intending to suppress SSR unused-variable warnings), but the `on:mousemove` closure inside `#[cfg(feature = "hydrate")]` references the names **without** the underscore prefix (`snaps_for_hover`, `points_for_hover`). The compiler cannot find these names in the hydrate target and raises `E0425`. The SSR target compiles fine because the `#[cfg(feature = "hydrate")]` block is excluded, but `cargo check --features hydrate --target wasm32-unknown-unknown` produces 3 errors, and the LP graph hover is completely broken in the browser.

This was confirmed by running `cargo check --features hydrate --target wasm32-unknown-unknown`.

**Fix:**

```rust
// Line 564-565: remove the leading underscores
let snaps_for_hover = snapshots.clone();
let points_for_hover = points.clone();
```

To suppress the "unused variable" warning on the SSR build, add `#[allow(unused_variables)]` above the declarations, matching the established project pattern (WASM rule 43):

```rust
#[allow(unused_variables)]
let snaps_for_hover = snapshots.clone();
#[allow(unused_variables)]
let points_for_hover = points.clone();
```

---

### CR-02: Rule 40 violation in `get_latest_ranked_snapshot` — `ORDER BY snapshotted_at` on a field not in the SELECT list

Status: FIXED in 5902a81 — '<string>snapshotted_at AS snapshotted_at' added to partial SELECT (Surreal Rule 40).

**File:** `src/server/db.rs:4581`
**Issue:** The query uses a partial SELECT (`SELECT queue_type, tier, division, lp, wins, losses`) but orders by `snapshotted_at`, which is not included in that SELECT list. Per project rule 40 (and the fix applied to `list_pending_join_requests` in a prior phase), SurrealDB 3.x rejects or silently misbehaves when `ORDER BY` references a field absent from a partial SELECT. This means the "latest" snapshot could be returned in arbitrary order, causing the ranked badge to display a stale or incorrect rank.

The `compute_goal_progress` batch query at line 4818 correctly includes `snapshotted_at` in its SELECT — the fix is the same pattern.

**Fix:**

```rust
// db.rs line 4581 — add snapshotted_at to the SELECT list
let mut result = db
    .query(
        "SELECT queue_type, tier, division, lp, wins, losses, snapshotted_at \
         FROM ranked_snapshot \
         WHERE user = type::record('user', $user_key) \
         AND queue_type = $queue_type \
         ORDER BY snapshotted_at DESC LIMIT 1",
    )
    // ...
```

Also update `DbRankedSnapshot` at line 4571 to include the field (it can be `Option<String>` since it is only needed for ordering, not returned to the caller):

```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct DbRankedSnapshot {
    queue_type: String,
    tier: String,
    division: String,
    lp: i32,
    wins: i32,
    losses: i32,
    snapshotted_at: Option<String>, // included for ORDER BY compliance (Rule 40)
}
```

---

## Warnings

### WR-01: LP history graph and goal progress stale after manual sync

Status: RESOLVED in Phase 16 commit b5930bc — hoisted lp_history_resource into SoloDashboardPage; both sync paths refetch all three resources.

**File:** `src/pages/solo_dashboard.rs:249, 276`
**Issue:** After a successful sync (both auto-sync and manual `do_sync`), only `dashboard_resource.refetch()` is called. The `lp_history_resource` and `goal_progress_resource` are independent resources and are not refetched. A user syncing their matches will see new games in the match list but the LP history graph and goal progress cards will remain stale until the page is reloaded.

**Fix:** Capture the resources in the closures and call `.refetch()` on all three after a successful sync:

```rust
// In SoloDashboardPage, pass lp_history_resource and goal_progress_resource into the closures.
// Auto-sync Effect (line 249):
dashboard_resource.refetch();
lp_history_resource.refetch();
goal_progress_resource.refetch();

// do_sync handler (line 276):
dashboard_resource.refetch();
lp_history_resource.refetch();
goal_progress_resource.refetch();
```

Note: `lp_history_resource` is defined inside `LpHistoryGraph` and not accessible from `SoloDashboardPage`. The cleanest fix is to hoist `lp_history_resource` up to `SoloDashboardPage` and pass it as a prop, or use a shared invalidation signal that `LpHistoryGraph` watches.

---

### WR-02: `get_personal_goals` is a public DB function with no production caller

Status: RESOLVED in Phase 16 commit c1b6753 — dead get_personal_goals removed; tests/db_personal_goal.rs deleted.

**File:** `src/server/db.rs:4698`
**Issue:** `pub async fn get_personal_goals` is exported and tested in `tests/db_personal_goal.rs`, but it is not called from any server function or page. The page instead calls `compute_goal_progress` (which inlines the goals query). This is dead public API — any future caller may expect it to return the same data shape as `compute_goal_progress`'s internal goals query, but the two queries differ slightly (one batches with match data, the other is standalone). This creates a risk of inconsistency if goals are fetched from two different sources.

**Fix:** Either add a `#[allow(dead_code)]` annotation if intentionally reserved for future use, or remove the function. If kept, add a doc comment noting that it is a standalone utility not used by the main progress flow.

```rust
/// List all personal goals for a user without computing progress.
/// Used in integration tests and future admin/export features.
/// For in-app goal progress, use `compute_goal_progress` instead.
#[allow(dead_code)]
pub async fn get_personal_goals(
```

---

## Info

### IN-01: Hardcoded Data Dragon patch version in `champion_icon_url`

Status: DEFERRED to Phase 19 — dynamic Data Dragon version loading is the natural home.

**File:** `src/pages/stats.rs:267`
**Issue:** Champion icon URLs use a hardcoded patch version `15.6.1`. When the game updates to a new patch, champion icons will 404 for newly added or renamed champions. The `on:error` fallback in `ChampionTrendRow` handles missing icons gracefully, but the `StatsPage` match rows (line 759, 866) have no such fallback — a broken icon would show a broken-image placeholder.

**Fix:** Extract the version to a constant or load it dynamically from Data Dragon's version endpoint. At minimum, centralise the string:

```rust
// In stats.rs (or a shared constants module)
const DDRAGON_VERSION: &str = "15.6.1";

fn champion_icon_url(champion_name: &str) -> String {
    format!("https://ddragon.leagueoflegends.com/cdn/{DDRAGON_VERSION}/img/champion/{champion_name}.png")
}
```

---

### IN-02: `unwrap_or(None)` anti-pattern

Status: DEFERRED — info-only finding; address ad-hoc when surrounding code at db.rs:4832 is touched.

**File:** `src/server/db.rs:4832`
**Issue:** `r.take(2).unwrap_or(None)` is equivalent to `r.take(2).ok().flatten()` or simply `r.take(2).unwrap_or_default()` for `Option<T>` where `Default` is `None`. The current form compiles but is idiomatically surprising — `unwrap_or(None)` on a `Result<Option<T>>` hides the error silently.

**Fix:**

```rust
// Line 4832
let current: Option<DbCurrentRankRow> = r.take(2).ok().flatten();
```

---

### IN-03: `trends_window_to_cutoff` marked `#[allow(dead_code)]`

Status: DEFERRED — info-only finding; address ad-hoc when surrounding code is touched.

**File:** `src/pages/stats.rs:182`
**Issue:** `trends_window_to_cutoff` is annotated `#[allow(dead_code)]` but is actually called at line 176 via `get_champion_trends`. The annotation is leftover — either from a refactor or added to suppress a spurious warning. Leaving it in place hides future accidental dead code in the same file.

**Fix:** Remove the `#[allow(dead_code)]` attribute — the function is clearly in use.

---

_Reviewed: 2026-05-05_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

---

## Second Pass (Phase 16 close-out)

**Reviewed:** 2026-05-07
**Diff scope:** Phase 16 commits 16-01 (b5930bc) and 16-02 (c1b6753, ba98015)
**Findings:** Critical: 0, HIGH: 0, Warning: 0, Info: 0

### Scope

Second-pass review covering the Phase 16 close-out commits against the Phase 15 codebase. Files reviewed: `src/pages/solo_dashboard.rs` (WR-01 hoist), `src/server/db.rs` (WR-02 removal), `tests/db_personal_goal.rs` (deleted). Bonus fixes also reviewed: `get_solo_matches` RecordId alias fix (922e0f8, 4c9431f, 293e43a).

### Findings

**No new findings introduced by Phase 16 commits.**

Both compile targets pass clean after Phase 16 commits:
- `cargo check --features ssr` — 0 errors, 0 new warnings
- `cargo check --features hydrate --target wasm32-unknown-unknown` — 0 errors; 1 warning (`window_to_cutoff` dead_code) is the pre-existing IN-03 finding from the Phase 15 first-pass review, not introduced by Phase 16

### Commit-by-commit analysis

**b5930bc — WR-01 hoist lp_window + lp_history_resource into SoloDashboardPage**
- `lp_window: RwSignal<&'static str>` and `lp_history_resource` correctly hoisted to `SoloDashboardPage` alongside `dashboard_resource` and `goal_progress_resource`
- `LpHistoryGraph` prop signature updated to accept both; internal `Resource::new` and `RwSignal::new("30d")` removed from child — correct ownership transfer
- Both sync paths (auto-sync Effect + `do_sync` handler) now call `.refetch()` on all three resources — matches D-04 design decision
- No reactive loop risk: `auto_synced` one-shot guard already in place (line 229)
- No Leptos Rule 23 violations; `.refetch()` called directly on the resource handles, not from within reactive closures

**c1b6753 — WR-02 remove dead get_personal_goals**
- `pub async fn get_personal_goals` and inner `DbPersonalGoal` struct deleted cleanly (34 lines removed)
- `compute_goal_progress`, `upsert_personal_goal`, `delete_personal_goal` untouched — correct scope
- `schema.surql` `personal_goal` table intact — correct (D-08)

**ba98015 — WR-02 delete tests/db_personal_goal.rs**
- 83-line test file deleted; 5 tests all asserted via the removed function
- `tests/db_goal_progress.rs` (168 lines) absorbs all coverage per D-06
- `cargo test --features ssr --lib` passes (102 tests)

**922e0f8, 4c9431f, 293e43a — get_solo_matches RecordId alias fix (bonus)**
- `SELECT *` replaced with explicit column aliases; `DbSoloMatch` intermediate struct introduced following the project Db* → model conversion pattern
- Rule 40 compliance: `<string>match.game_end AS game_end` added to SELECT; `ORDER BY game_end` references the alias not the traversal idiom
- Closes a silent deserialization regression present since Phase 12

### Verdict

**PASS — 0 Critical, 0 HIGH, 0 Warning, 0 Info attributable to Phase 16 commits.**

Pre-existing IN-03 finding (`window_to_cutoff` dead_code warning) is unchanged — correctly deferred per D-15.
