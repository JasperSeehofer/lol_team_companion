# Phase 16: Phase 15 Close-out - Pattern Map

**Mapped:** 2026-05-06
**Files analyzed:** 5 (2 source files modified, 1 source file deleted, 2 planning files updated)
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/pages/solo_dashboard.rs` | page/component | request-response (resource hoist + refetch) | `src/pages/action_items.rs` | exact (same refetch-in-handler shape) |
| `src/server/db.rs` | service | CRUD (delete dead function) | `src/server/db.rs:4733+` (surrounding functions) | exact (same function shape to delete) |
| `tests/db_personal_goal.rs` | test | — (delete entire file) | `tests/db_goal_progress.rs` | role-match (absorbs coverage) |
| `.planning/phases/15-goals-lp-history/15-REVIEW.md` | planning doc | — (add Status: lines) | `.planning/phases/15-goals-lp-history/15-REVIEW.md` existing entries | self (inline edit) |
| `.planning/STATE.md` + `.planning/MILESTONES.md` | planning doc | — (update shipped status) | `.planning/MILESTONES.md` v1.0 / v1.1 entries | self (format mirror) |

---

## Pattern Assignments

### `src/pages/solo_dashboard.rs` — WR-01: hoist `lp_history_resource` + `lp_window`

**Change type:** resource hoist + prop-down + refetch wiring

---

#### Pattern 1: Resource definition at parent level (model to replicate)

**Analog:** `src/pages/solo_dashboard.rs` lines 224–226 (existing `dashboard_resource` + `goal_progress_resource` definitions, already in `SoloDashboardPage`)

```rust
// Lines 224–226 — the two resources that already live in SoloDashboardPage
let dashboard_resource = Resource::new(move || queue_filter.get(), |qf| get_solo_dashboard(qf));
let goal_progress_resource = Resource::new(|| (), |_| async move { compute_goal_progress().await });
```

**New resource to add immediately after line 226 (hoisted from LpHistoryGraph lines 475–479):**

```rust
let lp_window: RwSignal<&'static str> = RwSignal::new("30d");
let lp_history_resource = Resource::new(
    move || lp_window.get(),
    |w| async move { get_lp_history(w.to_string()).await },
);
```

---

#### Pattern 2: Refetch in both sync paths (the missing wiring)

**Analog A:** `src/pages/action_items.rs` lines 215–225 — `spawn_local` + `Ok(_) => resource.refetch()` shape

```rust
// action_items.rs lines 215–225
leptos::task::spawn_local(async move {
    match create_action_item_fn(text, assigned).await {
        Ok(_) => {
            // ... state resets ...
            items.refetch();          // <-- single resource refetch
        }
        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
    }
});
```

**Analog B:** `src/pages/champion_pool.rs` lines 374, 427, 440, 583 — multiple `.refetch()` calls after mutation in same handler

**Apply to `solo_dashboard.rs` line 249 (auto-sync Effect `Ok` branch):**

```rust
// BEFORE (line 249):
dashboard_resource.refetch();

// AFTER — add two lines immediately below:
dashboard_resource.refetch();
goal_progress_resource.refetch();
lp_history_resource.refetch();
```

**Apply to `solo_dashboard.rs` line 276 (`do_sync` handler `Ok` branch):**

```rust
// BEFORE (line 276):
dashboard_resource.refetch();

// AFTER — add two lines immediately below:
dashboard_resource.refetch();
goal_progress_resource.refetch();
lp_history_resource.refetch();
```

---

#### Pattern 3: Prop-down of Resource + RwSignal into child component

**Analog:** `src/pages/solo_dashboard.rs` lines 709–710 — `GoalCards` already uses this exact signature shape

```rust
// solo_dashboard.rs lines 709–710 (GoalCards — the model to copy)
#[component]
fn GoalCards(progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>) -> impl IntoView {
```

**New `LpHistoryGraph` signature (replacing the no-arg signature at line 474):**

```rust
#[component]
fn LpHistoryGraph(
    lp_history_resource: Resource<Result<Vec<RankedSnapshot>, ServerFnError>>,
    lp_window: RwSignal<&'static str>,
) -> impl IntoView {
```

Remove the two lines currently at 475–479 (the old `RwSignal::new` + `Resource::new` inside the child). Everything else in `LpHistoryGraph` stays identical — the `render_pill`, `Suspense`, `tooltip` signal, and SVG rendering all remain inside the child.

---

#### Pattern 4: Updated call site at `solo_dashboard.rs` line 323

**Current (line 323):**

```rust
<LpHistoryGraph />
```

**After hoist:**

```rust
<LpHistoryGraph lp_history_resource=lp_history_resource lp_window=lp_window />
```

---

### `src/server/db.rs` — WR-02: delete `get_personal_goals`

**Change type:** delete dead function (lines 4699–4731, inclusive)

**Analog context:** The function immediately following (`upsert_personal_goal` at line 4733) and the function that supersedes it (`compute_goal_progress` at 4774) both stay untouched.

**Function to delete (lines 4699–4731):**

```rust
pub async fn get_personal_goals(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<crate::models::match_data::PersonalGoal>> {
    // ... 33 lines total, ending at line 4731 ...
}
```

Delete the entire function body including the trailing `}` at line 4731. The blank line at 4732 (before `/// Upsert a personal goal`) may be retained or trimmed — no functional difference.

**Verify after deletion:** `cargo check --features ssr` must compile clean. `cargo test --features ssr --lib` must pass.

---

### `tests/db_personal_goal.rs` — WR-02: delete entire file

**Change type:** file deletion

**Surviving coverage:** `tests/db_goal_progress.rs` already covers upsert + compute_goal_progress + cross-user isolation + empty-state semantics. No assertion backfill needed.

**Verify after deletion:** `cargo test --features ssr --lib` must pass (5 tests fewer, all from `db_personal_goal.rs`).

---

### `.planning/phases/15-goals-lp-history/15-REVIEW.md` — add Status: lines

**Change type:** documentation annotation

**Format (D-14):** `Status: <STATE> in <hash> — <one-line rationale>` as the first content line under each finding's header.

**Concrete annotations per D-14 and D-15:**

```
### CR-01 header
Status: FIXED in 5902a81 — underscore-prefix issue resolved by renaming snaps_for_hover/points_for_hover.

### CR-02 header
Status: FIXED in 5902a81 — '<string>snapshotted_at AS snapshotted_at' added to partial SELECT (Surreal Rule 40).

### WR-01 header
Status: RESOLVED in Phase 16 commit <hash> — hoisted lp_history_resource into SoloDashboardPage; both sync paths refetch all three resources.

### WR-02 header
Status: RESOLVED in Phase 16 commit <hash> — dead get_personal_goals removed; tests/db_personal_goal.rs deleted.

### IN-01 header
Status: DEFERRED to Phase 19 — dynamic Data Dragon version loading is the natural home.

### IN-02 header
Status: DEFERRED — info-only finding; address ad-hoc when surrounding code at db.rs:4832 is touched.

### IN-03 header
Status: DEFERRED — info-only finding; address ad-hoc when surrounding code is touched.
```

**Second-pass section (D-16):** Append `## Second Pass (Phase 16 close-out)` at the end of `15-REVIEW.md` with the `/gsd-code-review 15` output after the WR-01/WR-02 commits land.

---

### `.planning/STATE.md` — flip `prior_milestone.status` to Shipped

**Change type:** planning file update

**Current value (line 5):**
```
status: Pivoted from v1.2 → v1.3 on 2026-05-06; entering Phase 16 (Phase 15 close-out)
```

**Target pattern:** Add a `prior_milestone` block (not present yet) OR update the status narrative to record v1.2 as Shipped. The exact field is the `status:` line in the YAML frontmatter — update to record v1.2 shipped.

**Analog:** No structured `prior_milestone` key exists yet in STATE.md; write as a new narrative entry at the bottom of `## Decisions` or as a note in `## Current Position`. Exact location is executor's discretion (D-12: direct edit, not delegated to skill).

---

### `.planning/MILESTONES.md` — add v1.2 entry

**Change type:** planning file update (prepend new entry)

**Analog format — mirror v1.1 entry (lines 3–26):**

```markdown
## v1.1 Polish, Draft & Opponents Rework (Shipped: 2026-03-24)

**Phases completed:** 6 phases, 17 plans, 20 tasks

**Key accomplishments:**

- [bullet list of shipped features]

---
```

**v1.2 entry content guidance:**
- Title: `v1.2 Solo Mode & Match Intelligence (Shipped: 2026-05-DD)`
- Phases completed: Phases 12–15 + Phase 16 close-out (5 phases)
- Key accomplishments: solo ranked dashboard, LP history graph, champion trends table, personal goals system, post-Phase-15 close-out review (0 critical findings)
- Prepend above the existing v1.1 entry so newest is first (matches the file's descending order)

---

## Shared Patterns

### Resource::refetch() after async mutation
**Source:** `src/pages/action_items.rs` lines 215–248 and `src/pages/champion_pool.rs` lines 370–380
**Apply to:** Both sync paths in `SoloDashboardPage` (`do_sync` handler and auto-sync `Effect`)

```rust
// Canonical shape (action_items.rs lines 215–225):
leptos::task::spawn_local(async move {
    match some_server_fn(args).await {
        Ok(_) => {
            resource_a.refetch();      // call .refetch() on every resource
            resource_b.refetch();      // that reads data changed by the mutation
            resource_c.refetch();
        }
        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
    }
});
```

### Resource prop-down to child component
**Source:** `src/pages/solo_dashboard.rs` lines 709–710 (`GoalCards`)
**Apply to:** `LpHistoryGraph` new signature

```rust
// GoalCards (canonical prop-down shape):
#[component]
fn GoalCards(progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>) -> impl IntoView {
    // child owns Suspense; parent owns Resource::new
```

### Suppressing unused-variable warnings on hydrate-only signals
**Source:** `src/pages/solo_dashboard.rs` lines 564–567 (post-CR-01 fix in commit 5902a81)
**Apply to:** Any signal/variable only read inside `#[cfg(feature = "hydrate")]`

```rust
#[allow(unused_variables)]
let snaps_for_hover = snapshots.clone();
#[allow(unused_variables)]
let points_for_hover = points.clone();
```

---

## No Analog Found

All files have close analogs. No entries.

---

## Metadata

**Analog search scope:** `src/pages/`, `src/server/db.rs`, `tests/`
**Files scanned:** 6 source files read, 3 grep passes
**Pattern extraction date:** 2026-05-06
