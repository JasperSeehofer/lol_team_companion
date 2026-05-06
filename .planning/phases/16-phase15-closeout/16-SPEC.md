# Phase 16: Phase 15 Close-out — Specification

**Created:** 2026-05-06 (expanded from seed produced by v1.2 → v1.3 pivot)
**Milestone:** v1.3 Launch Readiness
**Ambiguity score:** 0.19 (gate: ≤ 0.20)
**Requirements:** 6 locked

## Goal

Resolve every open `15-REVIEW.md` finding (or explicitly defer it with a written rationale) so v1.2 Solo Mode & Match Intelligence is recorded as shipped before any v1.3 launch-readiness work begins.

## Background

Phase 15 (Goals & LP History) shipped UAT 11/11 at commit `4856140` on 2026-05-06. The phase code review (`.planning/phases/15-goals-lp-history/15-REVIEW.md`, dated 2026-05-05) flagged 2 critical and 2 warning findings. Commit `5902a81` already fixed both criticals (CR-01 WASM hover variable rename, CR-02 Rule 40 violation in `get_latest_ranked_snapshot`) — verified by reading the current source:

- `src/pages/solo_dashboard.rs:564-567` declares `snaps_for_hover` / `points_for_hover` without the leading underscore the original review flagged; the `#[cfg(feature = "hydrate")]` closure compiles.
- `src/server/db.rs:4582` has `<string>snapshotted_at AS snapshotted_at` in the partial SELECT, satisfying SurrealDB Rule 40.

What is **not** yet done:

- **WR-01 open** — `src/pages/solo_dashboard.rs:249, 276` only call `dashboard_resource.refetch()` after sync. `goal_progress_resource` (declared at line 226 in `SoloDashboardPage`) and `lp_history_resource` (declared at line 476 inside `LpHistoryGraph`) are not refetched. Result: after sync, the LP graph and goal cards stay stale until a full page reload.
- **WR-02 open** — `pub async fn get_personal_goals` at `src/server/db.rs:4699` has zero production callers (`grep -rn get_personal_goals src/` returns only the definition). The page uses `compute_goal_progress` instead. The function is dead public API with risk of inconsistency.
- **15-REVIEW.md not reconciled** — still lists CR-01 and CR-02 as open critical findings.
- **No second-pass review** — no `/gsd-code-review 15` run since the close-out fixes will land.
- **No re-verification** — `/gsd-verify-work 15` has not been run against ROADMAP Phase 15 success criteria after the close-out fixes.
- **Milestone records lag** — STATE.md still shows v1.2 as "Closing"; MILESTONES.md has v1.0 and v1.1 entries but no v1.2 entry.

This phase clears the four items above, then marks v1.2 as shipped. It is intentionally tiny — no DB schema changes, no new pages or routes; touches `solo_dashboard.rs` and `db.rs` only, plus planning files.

## Requirements

1. **Sync refreshes all dashboard resources**: after a successful auto-sync or manual sync, the LP history graph and goal progress cards reflect new data without a page reload.
   - Current: `solo_dashboard.rs:249` (auto-sync Effect) and `solo_dashboard.rs:276` (`do_sync` handler) call only `dashboard_resource.refetch()`. `lp_history_resource` is defined inside the child `LpHistoryGraph` component (line 476) and `goal_progress_resource` inside `SoloDashboardPage` (line 226); neither is refetched on sync.
   - Target: both sync paths trigger refetch of `dashboard_resource`, `goal_progress_resource`, and `lp_history_resource`. `lp_history_resource` is hoisted into `SoloDashboardPage` (or a shared invalidation signal is introduced — implementation detail belongs to discuss-phase) so it is reachable from the sync handlers.
   - Acceptance: starting from a Phase 15 dashboard with stale matches, calling `sync_solo_matches()` (manual button or auto Effect) updates the LP graph data points and goal progress card values within the same page render — verified by an e2e test or agent-browser snapshot that does not reload the page between sync and check.

2. **`get_personal_goals` has a caller or is removed**: no public DB function exists without a documented production purpose.
   - Current: `pub async fn get_personal_goals` at `src/server/db.rs:4699`; `grep -rn get_personal_goals src/` returns only the definition. Tests in `tests/db_personal_goal.rs` reference it.
   - Target: either (a) the function is wired to a real caller (server fn or page) with a comment explaining the role distinct from `compute_goal_progress`, or (b) the function is deleted along with its dedicated tests.
   - Acceptance: `grep -rn get_personal_goals src/` returns either 0 hits (full removal, including tests that exclusively cover it) or ≥ 2 hits (definition + at least one production caller in `src/`). `cargo check --features ssr` and `cargo test --features ssr --lib` both pass.

3. **`15-REVIEW.md` reconciled**: every finding listed in the Phase 15 review is annotated with its current resolution status.
   - Current: `.planning/phases/15-goals-lp-history/15-REVIEW.md` still presents CR-01, CR-02, WR-01, WR-02 as if all four were open; CR-01/CR-02 were fixed in `5902a81` 8 days before this phase.
   - Target: each of the 4 CR/WR findings carries a status line: "Fixed in `<commit>`", "Resolved in Phase 16 commit `<hash>`", or "Deferred to Phase X" with rationale. The three IN-* findings remain or are explicitly marked deferred per Boundaries below.
   - Acceptance: reading `15-REVIEW.md`, no CR or WR finding is left without an explicit resolution annotation. A `grep -E "Status:|FIXED|RESOLVED|DEFERRED" .planning/phases/15-goals-lp-history/15-REVIEW.md` finds entries for all four CR/WR IDs.

4. **Second-pass code review on close-out fixes**: a fresh code review covers commits introduced by Phase 16 itself.
   - Current: the most recent `15-REVIEW.md` was produced 2026-05-05 — before commit `5902a81` and before any Phase 16 fix lands.
   - Target: `/gsd-code-review 15` is run after Phase 16's WR-01 and WR-02 fixes are committed. The output is captured (either appended to `15-REVIEW.md` or written as a new section) and contains zero new HIGH/Critical findings introduced by the close-out commits.
   - Acceptance: a code-review artifact exists for the post-close-out state; its findings list shows 0 entries in the Critical and HIGH severity buckets attributable to Phase 16 commits.

5. **Phase 15 success criteria re-verified**: the four ROADMAP Phase 15 success criteria still hold after Phase 16 fixes.
   - Current: Phase 15 UAT was 11/11 at `4856140`; no verification has been run after `5902a81` or after the upcoming WR-01/WR-02 fixes.
   - Target: `/gsd-verify-work 15` is executed against ROADMAP `Phase 15 → Success Criteria 1-4` after Phase 16's code commits land. All four criteria pass.
   - Acceptance: a verification artifact (e.g. `15-VERIFICATION.md`) exists and reports PASS for ROADMAP criteria 1-4. The previously-passed UAT 11/11 remains green when re-checked.

6. **v1.2 marked shipped**: project state files reflect the close of milestone v1.2.
   - Current: `.planning/STATE.md` `prior_milestone.status` reads `Closing — Phase 15 functionally delivered (UAT 11/11), close-out moved to Phase 16 in v1.3`. `.planning/MILESTONES.md` ends at the v1.1 entry — no v1.2 record exists.
   - Target: after Phase 16 requirements 1-5 pass, STATE.md `prior_milestone.status` is updated to a "Shipped" wording with the close date; MILESTONES.md gains a v1.2 entry summarising Phases 12-15 plus the Phase 16 close-out.
   - Acceptance: `grep -i "Shipped" .planning/STATE.md` matches the `prior_milestone` block. `grep -i "v1\.2" .planning/MILESTONES.md` returns ≥ 1 hit pointing at a new heading. STATE.md continues to declare the active milestone as v1.3 (this requirement does not bump the active milestone).

## Boundaries

**In scope:**
- WR-01 fix in `src/pages/solo_dashboard.rs` (refetch all three resources after sync; hoist or share `lp_history_resource` so the sync handlers can reach it).
- WR-02 resolution in `src/server/db.rs` (wire `get_personal_goals` to a real caller, or remove the function plus tests that exclusively cover it).
- Reconciliation of `.planning/phases/15-goals-lp-history/15-REVIEW.md` to mark CR-01, CR-02, WR-01, WR-02 with explicit resolution status.
- One run each of `/gsd-code-review 15` (second pass) and `/gsd-verify-work 15` after the fixes land.
- Updates to `.planning/STATE.md` and `.planning/MILESTONES.md` so v1.2 is recorded as shipped.

**Out of scope:**
- IN-01 hardcoded Data Dragon patch `15.6.1` in `stats.rs` — deferred to Phase 19 (Production Hardening), where dynamic Data Dragon version loading is the natural home.
- IN-02 `unwrap_or(None)` anti-pattern at `db.rs:4832` — info-only finding; handle ad-hoc when the surrounding code is touched, otherwise leave as accepted tech debt.
- IN-03 spurious `#[allow(dead_code)]` on `trends_window_to_cutoff` — info-only finding; handle ad-hoc when the surrounding code is touched.
- Old Phase 16 (Draft Integration) — renumbered Phase 24 in v1.4; this phase deliberately does NOT touch personal matchup notes in the draft sidebar or champion-pool win-rate badges.
- New v1.3 launch work — Phase 17+ owns UI consolidation, the bug-report widget, deploy infra, compliance, etc. This phase does not start any of it.
- DB schema changes — none required; the v1.2 schema is the v1.2 ship state.
- New pages or routes — none required.
- Performance optimisation beyond what the WR-01 / WR-02 fixes incidentally require.
- v1.4 prep work (e.g. seeding Phase 24 plans).

## Constraints

- **Established refetch pattern:** the WR-01 fix must use `Resource::refetch()` (the existing project idiom — see `src/pages/champion_pool.rs` and `src/pages/action_items.rs` for examples) rather than introducing a new invalidation primitive.
- **Both targets must compile clean:** `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` both succeed with no errors. Per CLAUDE.md "Debugging reactive bugs without a browser", verify both targets after every reactive change.
- **WASM rules continue to hold:** any new closure in `solo_dashboard.rs` must follow `.claude/rules/wasm-patterns.md` Rule 35 (don't `.unwrap()` in WASM event handlers) and Rule 56 (stale captures in delayed closures) — relevant because the WR-01 fix touches signal lifecycle.
- **Phase 15 UAT must not regress:** Phase 15's previously-passing UAT 11/11 remains 11/11 after the close-out fixes.
- **Test suite must not regress:** `cargo test --features ssr --lib` passes; integration tests in `tests/` may be skipped per CLAUDE.md (BFD linker OOM with `--lib`-less invocation).
- **Scope size:** tiny phase — likely 1 plan. If discuss-phase / plan-phase produces > 2 plans, that is a signal the boundaries above leaked.

## Acceptance Criteria

- [ ] After a manual "Sync" click on `/solo/dashboard`, the LP history graph displays the new match's data point without a page reload
- [ ] After a manual "Sync" click on `/solo/dashboard`, goal progress cards display updated values without a page reload
- [ ] After auto-sync (the Effect at `solo_dashboard.rs:233`) fires, both the LP graph and goal cards refetch
- [ ] `grep -rn get_personal_goals src/` returns either 0 hits OR ≥ 2 hits (definition + at least one production caller)
- [ ] `cargo check --features ssr` exits 0 with no errors
- [ ] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0 with no errors
- [ ] `cargo test --features ssr --lib` exits 0
- [ ] `.planning/phases/15-goals-lp-history/15-REVIEW.md` annotates every CR/WR finding (CR-01, CR-02, WR-01, WR-02) with an explicit resolution status — no finding is left ambiguously "open"
- [ ] A second-pass `/gsd-code-review 15` artifact exists and reports zero new HIGH/Critical findings introduced by Phase 16 commits
- [ ] A `/gsd-verify-work 15` artifact exists and reports PASS for ROADMAP Phase 15 success criteria 1-4
- [ ] `.planning/STATE.md` `prior_milestone.status` no longer reads "Closing" (reads "Shipped" or equivalent terminal state) with a close date
- [ ] `.planning/MILESTONES.md` contains a v1.2 entry summarising Phases 12-15 + Phase 16 close-out

## Ambiguity Report

| Dimension          | Score | Min  | Status | Notes                                                                                          |
|--------------------|-------|------|--------|------------------------------------------------------------------------------------------------|
| Goal Clarity       | 0.85  | 0.75 | ✓      | "Resolve all open Phase 15 review findings" with 4 specific IDs (CR-01/CR-02 already fixed)    |
| Boundary Clarity   | 0.85  | 0.70 | ✓      | Explicit out-of-scope list with rationale per item; IN-01 → Phase 19, old Phase 16 → Phase 24  |
| Constraint Clarity | 0.70  | 0.65 | ✓      | Refetch pattern locked; both compile targets must pass; WASM rules 35/56 cited                 |
| Acceptance Criteria| 0.80  | 0.70 | ✓      | 12 pass/fail checkboxes — most reduce to a grep, a build, or an artifact-exists test           |
| **Ambiguity**      | 0.19  | ≤0.20| ✓      | Gate passed before interview; expanded from a strongly-grounded seed produced by the pivot     |

Status: ✓ = met minimum, ⚠ = below minimum (planner treats as assumption)

## Interview Log

| Round | Perspective    | Question summary                                              | Decision locked                                                                                |
|-------|----------------|---------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| 0     | Auto (seed)    | Seed SPEC.md from pivot scoped the close-out — gate passes?   | Yes — initial ambiguity 0.19, all dimensions ≥ minimum; skip interview, expand seed directly   |
| —     | Researcher     | (auto-derived) What exists today vs target?                   | CR-01/CR-02 fixed in `5902a81`, verified in source; WR-01/WR-02 confirmed open via grep        |
| —     | Boundary Keeper| (auto-derived) What's NOT in this phase?                      | IN-01 → Phase 19; IN-02/IN-03 ad-hoc; old Phase 16 (Draft Integration) → v1.4 Phase 24         |
| —     | Failure Analyst| (auto-derived) What invalidates the requirements?             | WR-01 fix that breaks WASM target; signal-lifecycle regressions per WASM Rule 56               |

Auto-derivation rationale: the seed SPEC produced by the v1.2 → v1.3 pivot already encoded the perspectives a Socratic interview would have surfaced (current state verification, explicit out-of-scope deferrals with target phases, named failure modes). Auto Mode was active. The four dimensions met their minimums on first scoring, so the interview short-circuit defined in the workflow's Step 3 applies.

---

*Phase: 16-phase15-closeout*
*Spec created: 2026-05-06 (expanded from seed dated 2026-05-06)*
*Next step: /gsd-discuss-phase 16 — implementation decisions (signal hoisting strategy for `lp_history_resource`, wire-or-remove decision for `get_personal_goals`, review/verify ordering)*
