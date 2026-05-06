# Phase 16: Phase 15 Close-out - Context

**Gathered:** 2026-05-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Resolve every open `15-REVIEW.md` finding (or explicitly defer it with a written rationale) so v1.2 Solo Mode & Match Intelligence is recorded as shipped before any v1.3 launch-readiness work begins.

CR-01 and CR-02 were already fixed in commit `5902a81` (verified live in source). The work that remains:
1. Fix WR-01 — sync paths refetch all dashboard resources, not just `dashboard_resource`.
2. Resolve WR-02 — `get_personal_goals` removed (no production caller; `compute_goal_progress` covers everything in payload).
3. Reconcile `15-REVIEW.md` so every CR/WR/IN finding has an explicit `Status:` line.
4. Run `/gsd-code-review 15` (second pass) on the close-out commits — must show 0 new HIGH/Critical.
5. Run `/gsd-verify-work 15` against ROADMAP Phase 15 success criteria 1-4 — must PASS.
6. Mark v1.2 shipped in `.planning/STATE.md` + `.planning/MILESTONES.md`.

This phase is intentionally tiny. Touches `src/pages/solo_dashboard.rs`, `src/server/db.rs`, and planning files only. No DB schema changes, no new pages or routes.

</domain>

<spec_lock>
## Requirements (locked via SPEC.md)

**6 requirements are locked.** See `16-SPEC.md` for full requirements, boundaries, and acceptance criteria.

Downstream agents MUST read `16-SPEC.md` before planning or implementing. Requirements are not duplicated here.

**In scope (from SPEC.md):**
- WR-01 fix in `src/pages/solo_dashboard.rs` (refetch all three resources after sync; hoist or share `lp_history_resource` so the sync handlers can reach it).
- WR-02 resolution in `src/server/db.rs` (wire `get_personal_goals` to a real caller, or remove the function plus tests that exclusively cover it).
- Reconciliation of `.planning/phases/15-goals-lp-history/15-REVIEW.md` to mark CR-01, CR-02, WR-01, WR-02 with explicit resolution status.
- One run each of `/gsd-code-review 15` (second pass) and `/gsd-verify-work 15` after the fixes land.
- Updates to `.planning/STATE.md` and `.planning/MILESTONES.md` so v1.2 is recorded as shipped.

**Out of scope (from SPEC.md):**
- IN-01 hardcoded Data Dragon patch `15.6.1` in `stats.rs` — deferred to Phase 19 (Production Hardening).
- IN-02 `unwrap_or(None)` anti-pattern at `db.rs:4832` — info-only; ad-hoc when surrounding code is touched.
- IN-03 spurious `#[allow(dead_code)]` on `trends_window_to_cutoff` — info-only; ad-hoc when surrounding code is touched.
- Old Phase 16 (Draft Integration) — renumbered Phase 24 in v1.4; this phase deliberately does NOT touch personal matchup notes in the draft sidebar or champion-pool win-rate badges.
- New v1.3 launch work — Phase 17+ owns UI consolidation, the bug-report widget, deploy infra, compliance, etc.
- DB schema changes — none required.
- New pages or routes — none required.
- Performance optimisation beyond what the WR-01 / WR-02 fixes incidentally require.
- v1.4 prep work (e.g. seeding Phase 24 plans).

</spec_lock>

<decisions>
## Implementation Decisions

### WR-01 — Signal hoisting strategy

- **D-01: Hoist `lp_history_resource` into `SoloDashboardPage`.** Move the `Resource::new` call out of `LpHistoryGraph` (currently `solo_dashboard.rs:476`) up to `SoloDashboardPage` so it sits alongside `dashboard_resource` (line 224) and `goal_progress_resource` (line 226). Sync handlers can then call `.refetch()` on all three.
- **D-02: Hoist `lp_window: RwSignal<&'static str>` alongside the resource.** The pill state is the resource's source signal — co-locate them. The closure `move || lp_window.get()` passed to `Resource::new` keeps working unchanged.
- **D-03: Pass full `Resource<Result<Vec<RankedSnapshot>, ServerFnError>>` and `RwSignal<&'static str>` as props into `LpHistoryGraph`.** Child stays Suspense-aware — its skeleton fallback (`SkeletonCard height="h-48"`), tooltip state, and pill rendering all remain inside the child component. Smallest diff, clearest ownership.
- **D-04: Both sync paths refetch all three resources directly.** Inside `do_sync` (line 259) and the auto-sync `Effect` (line 232), after `Ok(n)` from `sync_solo_matches()`, call:
  ```rust
  dashboard_resource.refetch();
  goal_progress_resource.refetch();
  lp_history_resource.refetch();
  ```
  No `sync_tick: RwSignal<u32>` invalidation primitive (would violate the SPEC constraint). The `auto_synced.get_untracked()` one-shot guard already prevents loops; no additional state needed. A helper closure for the three refetches is optional DRY — fine to add but not required.

### WR-02 — `get_personal_goals` fate

- **D-05: Remove `pub async fn get_personal_goals` from `src/server/db.rs:4699`.** No production caller exists; `compute_goal_progress` (db.rs:4774) inlines an equivalent `SELECT id, goal_type, target_value FROM personal_goal WHERE user = ...` as part of its batched 3-statement query (Rule 29 — keeps a single round-trip), and goal-edit forms read targets from the resulting `GoalProgressPayload` (e.g., `RankTargetCard` line 740-741).
- **D-06: Delete `tests/db_personal_goal.rs` in its entirety** (5 tests). All five assert via `get_personal_goals`; they're redundant with `tests/db_goal_progress.rs`, which already exercises upsert → compute_goal_progress paths. No assertion backfill needed — `db_goal_progress.rs` already covers create, overwrite, cross-user isolation, and empty-state semantics.
- **D-07: WR-02 lands as its own commit, separate from WR-01.** `fix(phase-16): WR-01 hoist resources …` then `chore(phase-16): WR-02 remove dead get_personal_goals`. Per-finding commits give 15-REVIEW.md's `Resolved in Phase 16 commit <hash>` annotations precise hashes.
- **D-08: `schema.surql` `personal_goal` table left intact.** Still used by `upsert_personal_goal`, `delete_personal_goal`, and `compute_goal_progress`. Removal is code-only; no migration, no risk to existing user goal data.

### Review/verify ordering

- **D-09: Strict serial execution order:**
  1. Commit WR-01 hoist fix (refetch on all three resources).
  2. Commit WR-02 removal (drop `get_personal_goals` + delete `tests/db_personal_goal.rs`).
  3. Run `/gsd-code-review 15` on the post-close-out state — gate: 0 new HIGH/Critical findings introduced by Phase 16 commits.
  4. Run `/gsd-verify-work 15` against ROADMAP Phase 15 success criteria 1-4 — gate: PASS for all four.
  5. Annotate `.planning/phases/15-goals-lp-history/15-REVIEW.md` with Status: lines for every CR/WR/IN finding.
  6. As the final commit of Phase 16, update `.planning/STATE.md` (`prior_milestone.status` → `Shipped 2026-05-DD`) and add a v1.2 entry to `.planning/MILESTONES.md` summarising Phases 12-15 + Phase 16 close-out. Active milestone stays v1.3.
- **D-10: New HIGH/Critical from Phase 16 commits in `/gsd-code-review 15` → fix in scope, re-review, do not defer.** SPEC R4 acceptance is "0 new HIGH/Critical" — looping until clean is the only path forward. Phase 16's purpose is delivering a clean Phase 15.
- **D-11: `/gsd-verify-work 15` regression on Phase 15 success criteria → fix in scope, re-verify, do not defer.** Same logic as D-10. The escape hatch (only if a regression is structurally unfixable in scope) is rolling back Phase 16's commits and reopening — but the WR-01/WR-02 surface is small enough that this should not be needed.
- **D-12: v1.2 mark-shipped is inline in this phase, not delegated to `/gsd-complete-milestone`.** v1.2 is `prior_milestone` (already historically closed); the GSD lifecycle skills target the active milestone. Direct edit is the right tool.

### 15-REVIEW.md annotation format

- **D-13: Inline `Status:` line at the top of each finding** (first content line under the finding's header), not a trailing resolution log. Reader sees status before re-reading the original analysis. `grep -E "Status:|FIXED|RESOLVED|DEFERRED" .planning/phases/15-goals-lp-history/15-REVIEW.md` matches all entries.
- **D-14: Status format = `Status: <STATE> in <hash> — <one-line rationale>`.** Concrete annotations:
  - **CR-01:** `Status: FIXED in 5902a81 — underscore-prefix issue resolved by renaming snaps_for_hover/points_for_hover.`
  - **CR-02:** `Status: FIXED in 5902a81 — '<string>snapshotted_at AS snapshotted_at' added to partial SELECT (Surreal Rule 40).`
  - **WR-01:** `Status: RESOLVED in Phase 16 commit <hash> — hoisted lp_history_resource into SoloDashboardPage; both sync paths refetch all three resources.`
  - **WR-02:** `Status: RESOLVED in Phase 16 commit <hash> — dead get_personal_goals removed; tests/db_personal_goal.rs deleted.`
- **D-15: IN-01/IN-02/IN-03 also receive explicit `Status: DEFERRED` lines** so the doc is fully self-describing for future readers:
  - **IN-01:** `Status: DEFERRED to Phase 19 — dynamic Data Dragon version loading is the natural home.`
  - **IN-02:** `Status: DEFERRED — info-only finding; address ad-hoc when surrounding code at db.rs:4832 is touched.`
  - **IN-03:** `Status: DEFERRED — info-only finding; address ad-hoc when surrounding code is touched.`
- **D-16: Second-pass `/gsd-code-review 15` output appended as a `## Second Pass (Phase 16 close-out)` section at the bottom of `15-REVIEW.md`.** Keeps the full review trail in one file, in chronological order.

### Claude's Discretion
- Exact commit message wording (the `fix(phase-16):` / `chore(phase-16):` / `docs(phase-16):` prefixes follow the project's existing convention shown in `git log`).
- Whether to introduce a small helper closure `let refetch_all = move || { ... }` for the three refetch calls (mild DRY win; not required).
- Whether the second-pass code-review section in `15-REVIEW.md` is added in the same commit as the inline Status: annotations or a separate `docs(phase-16):` commit.
- Exact wording of the v1.2 MILESTONES.md entry (should summarise Phases 12-15 + Phase 16 close-out; format mirrors the existing v1.0 / v1.1 entries).
- The exact ISO date written into `STATE.md` `prior_milestone.status` ("Shipped 2026-05-DD") — fill at commit time.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Locked Requirements
- `.planning/phases/16-phase15-closeout/16-SPEC.md` — Locked requirements, boundaries, acceptance criteria. **MUST read before planning.**

### Requirements & Roadmap
- `.planning/ROADMAP.md` — Phase 16 entry under v1.3 milestone; Phase 15 success criteria 1-4 (the targets `/gsd-verify-work 15` checks).
- `.planning/REQUIREMENTS.md` — Phase 15 RANK-02 / LEARN-04/05/06 definitions (already shipped; verify-work re-runs against them).

### Source of the WR-01 / WR-02 Findings
- `.planning/phases/15-goals-lp-history/15-REVIEW.md` — The doc this phase reconciles. Contains the original CR-01, CR-02, WR-01, WR-02, IN-01, IN-02, IN-03 analyses.
- `.planning/phases/15-goals-lp-history/15-CONTEXT.md` — Phase 15 implementation decisions (D-01..D-20) — context for what the LP graph and goals system are doing.

### Project State Files
- `.planning/STATE.md` — `prior_milestone.status` block to update (currently reads "Closing — Phase 15 functionally delivered…").
- `.planning/MILESTONES.md` — Receives a new v1.2 entry; format mirrors existing v1.0 / v1.1 entries.

### Source — WR-01 Touch Points
- `src/pages/solo_dashboard.rs:209` — `SoloDashboardPage` component (target for `lp_history_resource` + `lp_window` hoisting).
- `src/pages/solo_dashboard.rs:224` — `dashboard_resource` definition (model for hoisting placement).
- `src/pages/solo_dashboard.rs:226` — `goal_progress_resource` definition (already at parent level).
- `src/pages/solo_dashboard.rs:232-257` — Auto-sync Effect (refetches `dashboard_resource` on line 249; needs the other two added).
- `src/pages/solo_dashboard.rs:259-282` — `do_sync` handler (refetches `dashboard_resource` on line 276; needs the other two added).
- `src/pages/solo_dashboard.rs:323` — `<LpHistoryGraph />` site (becomes `<LpHistoryGraph lp_history_resource=… lp_window=… />`).
- `src/pages/solo_dashboard.rs:473-558` — `LpHistoryGraph` component body (currently owns `lp_window` line 475, `lp_history_resource` line 476; both move to props).

### Reference Implementations of `Resource::refetch()` (the locked idiom)
- `src/pages/champion_pool.rs` — Existing project use of `.refetch()` after mutations.
- `src/pages/action_items.rs` — Same pattern; reference for handler shape.

### Source — WR-02 Touch Points
- `src/server/db.rs:4699-4731` — `pub async fn get_personal_goals` body (delete entire function).
- `src/server/db.rs:4774-4822` — `compute_goal_progress` (note the inline goal SELECT inside the batched query — this is what makes `get_personal_goals` redundant). Do NOT touch.
- `tests/db_personal_goal.rs` — Delete entire file (5 tests: lines 19, 31, 47, 64, 78).
- `tests/db_goal_progress.rs` — Existing coverage of upsert → compute_goal_progress paths; remains the test home for goal behaviour.

### Patterns / Rules (auto-loaded path-specific)
- `.claude/rules/leptos-patterns.md` — Rules 11/12 (server fn DB+auth context), 23 (`resource.refetch()` after mutations — the WR-01 pattern), 18 (clone before multiple closures — relevant if a `refetch_all` helper is added), 24 (`spawn_local` for async event handlers — already used in `do_sync`).
- `.claude/rules/wasm-patterns.md` — Rule 35 (no `.unwrap()` in event handlers; the WR-01 fix touches WASM closures), Rule 56 (E2e WASM Effect settle delay — relevant if WR-01 verification uses agent-browser).
- `.claude/rules/surreal-patterns.md` — Rule 29 (batched queries; explains why `compute_goal_progress` inlines the personal_goal SELECT instead of calling `get_personal_goals`).

### Verification Targets
- `.planning/ROADMAP.md` Phase 15 success criteria 1-4 — what `/gsd-verify-work 15` re-runs against.
- `tests/db_goal_progress.rs` — must pass after WR-02 removal.
- `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` — both targets must compile clean (Phase 16 SPEC Constraint).
- `cargo test --features ssr --lib` — must pass.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Resource::refetch()` idiom** — already wired in `src/pages/champion_pool.rs` and `src/pages/action_items.rs`; same shape extends to the WR-01 fix.
- **`compute_goal_progress` payload** (`src/server/db.rs:4774`) — already returns `GoalProgressPayload` with goals embedded, eliminating any need for a separate `get_personal_goals` server fn. Goal-edit forms (`RankTargetCard`, `CsGoalCard`, `DeathsGoalCard`) read targets directly from the payload.
- **`tests/db_goal_progress.rs`** — already covers upsert + cross-user isolation + empty-state semantics; absorbs the coverage moved out of `tests/db_personal_goal.rs`.
- **`auto_synced` one-shot guard** (`solo_dashboard.rs:229`) — already prevents the auto-sync Effect from firing more than once per page load; the WR-01 fix doesn't need any new guards.

### Established Patterns
- **Resource hoisting + prop-down** — `goal_progress_resource` is already defined in the parent and passed into `GoalCards` (line 328); WR-01 mirrors this exact pattern for `lp_history_resource`.
- **Atomic per-finding commits** — project convention (visible in `git log`); WR-01 and WR-02 land as separate commits so 15-REVIEW.md's per-finding hash annotations are precise.
- **Batched SurrealDB queries via Rule 29** — `compute_goal_progress`'s 3-statement batched query is the project's preferred shape; argues against extracting `get_personal_goals` as a helper that would force two round-trips.
- **`Resource<Result<T, ServerFnError>>` as a prop type** — pattern reference: `GoalCards(progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>)` at `solo_dashboard.rs:710`.

### Integration Points
- **`SoloDashboardPage`** (`src/pages/solo_dashboard.rs:209`): gains `lp_history_resource` + `lp_window` definitions; both sync handlers gain two more `.refetch()` calls.
- **`LpHistoryGraph`** (`src/pages/solo_dashboard.rs:473`): signature changes to accept the hoisted resource + signal as props; internal `Resource::new` and `RwSignal::new("30d")` are removed.
- **`src/server/db.rs`**: `get_personal_goals` deleted; `compute_goal_progress` and `upsert_personal_goal` / `delete_personal_goal` untouched.
- **`tests/db_personal_goal.rs`**: file deleted.
- **`schema.surql`**: untouched.
- **`.planning/phases/15-goals-lp-history/15-REVIEW.md`**: gains 7 inline `Status:` lines (4 CR/WR + 3 IN) and a trailing `## Second Pass (Phase 16 close-out)` section.
- **`.planning/STATE.md`**: `prior_milestone.status` flips from "Closing" to "Shipped 2026-05-DD".
- **`.planning/MILESTONES.md`**: gains a v1.2 entry.

</code_context>

<specifics>
## Specific Ideas

- The Phase 15 carryover decisions (Phase 12 D-07 single-column dashboard, Phase 15 D-01..D-06 LP graph) are NOT being changed — WR-01 is a refetch-wiring fix only. The visual layout, the SVG graph implementation, and the cumulative LP scale remain exactly as Phase 15 shipped them.
- The "auto-sync fires once per page load" semantics from Phase 15 are preserved. After the WR-01 fix, the auto-sync Effect still fires once per mount when `data.should_auto_sync && data.has_puuid && !auto_synced.get_untracked()` — but now refetches all three resources instead of just one.
- `compute_goal_progress`'s inline goal SELECT is the reason `get_personal_goals` is dead. This is intentional architecture (Rule 29 batching), not an oversight — the right move is to delete the dead function, not to refactor the production query to call it.
- Per-finding atomic commits give the 15-REVIEW.md `Resolved in Phase 16 commit <hash>` annotations clean per-finding traceability — both for the live audit and for any future revert/forensics.

</specifics>

<deferred>
## Deferred Ideas

- **IN-01 hardcoded Data Dragon patch** (`src/pages/stats.rs`) — explicitly deferred to Phase 19 (Production Hardening) per SPEC Boundaries.
- **IN-02 `unwrap_or(None)` anti-pattern at `db.rs:4832`** — info-only; ad-hoc when surrounding code is touched.
- **IN-03 spurious `#[allow(dead_code)]` on `trends_window_to_cutoff`** — info-only; ad-hoc when surrounding code is touched.
- **Old Phase 16 (Draft Integration)** — renumbered Phase 24 in v1.4 milestone; this phase deliberately does NOT touch personal matchup notes in the draft sidebar or champion-pool win-rate badges.
- **`refetch_all` helper closure** — mild DRY win for the three sync-handler refetches; planner/executor's call whether to introduce it or inline the three calls.
- **Audit other db.rs functions for similar dead-code patterns** — out of scope per SPEC Boundaries (this phase is intentionally tiny). Ad-hoc when those areas are touched.
- **`/gsd-debug 16` for non-obvious regressions** — escape hatch if `/gsd-verify-work 15` surfaces something structurally hard to fix; not the default path.
- **Phase 17+ v1.3 launch work** — UI consolidation, bug-report widget, deploy infra, compliance, full pre-launch review, closed-beta soft launch. Owned by their own phases.

</deferred>

---

*Phase: 16-phase15-closeout*
*Context gathered: 2026-05-06*
