# Phase 16: Phase 15 Close-out - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-06
**Phase:** 16-phase15-closeout
**Areas discussed:** Signal hoisting strategy, get_personal_goals fate, Review/verify ordering, 15-REVIEW.md annotation format

---

## Signal hoisting strategy

### Q1 — Where should lp_history_resource be defined?

| Option | Description | Selected |
|--------|-------------|----------|
| Hoist into SoloDashboardPage (Recommended) | Define alongside dashboard_resource and goal_progress_resource at lines ~224-226. Pass as Resource prop into LpHistoryGraph. Sync handlers call .refetch() on all three. Pure refetch idiom, matches the SPEC constraint, mirrors goal_progress_resource. | ✓ |
| Keep in LpHistoryGraph + lift via context | Use provide_context::<Resource<...>>() so sync handlers can use_context() it. Avoids prop-drilling but introduces a context type for one resource — inconsistent with goal_progress_resource. | |
| Use a sync_tick RwSignal source | Parent owns sync_tick; each child Resource depends on (lp_window.get(), sync_tick.get()). Violates the SPEC constraint ("use Resource::refetch() rather than introducing a new invalidation primitive"). | |

**User's choice:** Hoist into SoloDashboardPage.

### Q2 — Where should lp_window live after hoisting?

| Option | Description | Selected |
|--------|-------------|----------|
| Hoist alongside the resource (Recommended) | Move lp_window: RwSignal<&'static str> into SoloDashboardPage. Pass both lp_window and lp_history_resource into LpHistoryGraph as props. Resource closure keeps using lp_window.get() unchanged. | ✓ |
| Keep lp_window in child, expose source signal as prop into Resource | Awkward; resource needs lp_window in scope at definition time. Forces context handoff or lazy resource construction. | |
| Replace pill state with parent-owned signal + callback | Parent owns lp_window; child receives ReadSignal + Callback<&'static str>. More moving parts than option 1. | |

**User's choice:** Hoist alongside the resource.

### Q3 — How should LpHistoryGraph receive the hoisted resource?

| Option | Description | Selected |
|--------|-------------|----------|
| Pass full Resource<Result<...>> as prop (Recommended) | Child stays Suspense-aware. Smallest diff, clear ownership. Skeleton fallback stays inside child. | ✓ |
| Parent reads, passes Vec<RankedSnapshot> down | Pre-resolve Suspense in parent, pass plain data. More refactor surface; loses per-section Suspense fallback. | |
| Inline LpHistoryGraph back into SoloDashboardPage | Eliminate the child component. Touches a lot of working code; bigger blast radius than WR-01 justifies. | |

**User's choice:** Pass full Resource<Result<...>> as prop.

### Q4 — Auto-sync Effect at line 232 must refetch all three resources. Anything else to guard?

| Option | Description | Selected |
|--------|-------------|----------|
| Just add the two extra refetch() calls (Recommended) | After successful sync, call refetch() on all three. The auto_synced one-shot guard already prevents loops. No new state. | ✓ |
| Add a syncing-completed signal that fires refetches via Effect | Indirection without benefit — sync handlers already know when to refetch. | |
| Wrap the three refetches in a helper closure | refetch_all = move \|\| { ... }. Mild DRY win; fine to do but not load-bearing. | |

**User's choice:** Just add the two extra refetch() calls.

**Notes:** Helper-closure variant left as Claude's discretion in CONTEXT.md.

---

## get_personal_goals fate

### Q1 — What should happen to get_personal_goals?

| Option | Description | Selected |
|--------|-------------|----------|
| Remove function + delete tests/db_personal_goal.rs (Recommended) | Delete the function from db.rs:4699 and the entire 5-test file. Coverage of upsert/overwrite/cross-user isolation already exists in tests/db_goal_progress.rs. Smallest surface; satisfies SPEC "0 hits" branch. | ✓ |
| Keep function, port asserts to use compute_goal_progress | Function stays unused — contradicts SPEC's "no dead public API" rule. | |
| Wire into a new server fn for goal-edit form pre-fill | Adds redundant round-trip when compute_goal_progress already returns goals in payload. | |

**User's choice:** Remove function + delete tests.

### Q2 — Backfill assertions into db_goal_progress.rs?

| Option | Description | Selected |
|--------|-------------|----------|
| No backfill needed (Recommended) | tests/db_goal_progress.rs already covers upsert, overwrite, cross-user isolation, empty-state. db_personal_goal.rs tests are largely redundant. | ✓ |
| Backfill the cross-user isolation test | Add equivalent isolation assertion to db_goal_progress.rs. | |
| Backfill the empty-vec edge case | Add empty-state assertion to db_goal_progress.rs. | |

**User's choice:** No backfill.

### Q3 — Land removal as its own commit, or batched with WR-01?

| Option | Description | Selected |
|--------|-------------|----------|
| Separate commit per finding (Recommended) | fix(phase-16): WR-01 ... then chore(phase-16): WR-02 .... Atomic commits, clean review trail, easy to revert independently. | ✓ |
| Single combined commit | fix(phase-16): close-out WR-01 + WR-02. Loses per-finding traceability for 15-REVIEW.md hash annotations. | |

**User's choice:** Separate commit per finding.

### Q4 — Touch schema.surql as part of WR-02?

| Option | Description | Selected |
|--------|-------------|----------|
| Leave personal_goal table intact (Recommended) | Still used by upsert_personal_goal, delete_personal_goal, compute_goal_progress. Code-only change; no schema migration. | ✓ |
| Audit other db.rs goal functions for similar dead code | Out of scope per SPEC Boundaries (IN-02/IN-03 ad-hoc). | |

**User's choice:** Leave personal_goal table intact.

---

## Review/verify ordering

### Q1 — High-level execution order?

| Option | Description | Selected |
|--------|-------------|----------|
| Fix → Review → Verify → Annotate → Ship (Recommended) | Strict serial: WR-01 commit → WR-02 commit → /gsd-code-review 15 → /gsd-verify-work 15 → annotate 15-REVIEW.md → STATE.md + MILESTONES.md. Quality gates fully gate the milestone close. | ✓ |
| Fix → Annotate → Review → Verify → Ship | Annotate immediately after fixes. If review surfaces a regression, annotation must be amended. | |
| Fix → Review → Annotate → Verify → Ship | Review first, annotate (capturing fix-commit + review-pass), verify, ship. Slight reordering; no real win. | |

**User's choice:** Fix → Review → Verify → Annotate → Ship.

### Q2 — New HIGH/Critical from Phase 16 commits in /gsd-code-review 15?

| Option | Description | Selected |
|--------|-------------|----------|
| Fix in this phase, re-review (Recommended) | Loop: fix → commit → re-run /gsd-code-review 15 → must reach 0 HIGH/Critical before proceeding to verify. Phase 16's purpose is delivering a clean Phase 15. | ✓ |
| Defer to a follow-up phase if low-risk | Violates SPEC R4 acceptance ("zero new HIGH/Critical findings"). | |
| Block the milestone close until external review | Heavyweight; only for genuinely ambiguous findings. | |

**User's choice:** Fix in this phase, re-review.

### Q3 — Phase 15 success-criterion regression in /gsd-verify-work 15?

| Option | Description | Selected |
|--------|-------------|----------|
| Treat as in-scope; fix and re-verify (Recommended) | Loop: fix → commit → re-run /gsd-verify-work 15 → must PASS criteria 1-4. Same handling as a regression-surfacing review. | ✓ |
| Diagnose via /gsd-debug 16 first | Reasonable for non-obvious regressions but adds ceremony for what's likely an obvious WR-01/WR-02 side effect. | |
| Roll back the Phase 16 commits and reopen | Escape hatch if a regression is structurally unfixable in scope. Unlikely given small surface. | |

**User's choice:** Treat as in-scope; fix and re-verify.

**Notes:** Roll-back option preserved as escape hatch in CONTEXT.md (D-11).

### Q4 — Where does v1.2 'Shipped' status check live?

| Option | Description | Selected |
|--------|-------------|----------|
| Inside Phase 16 as R6, after R4+R5 pass (Recommended) | STATE.md prior_milestone.status = "Shipped 2026-05-DD" + MILESTONES.md v1.2 entry. Last commit of Phase 16. Active milestone stays v1.3. | ✓ |
| Defer mark-shipped to /gsd-complete-milestone or /gsd-audit-milestone | v1.2 is prior_milestone (already historically closed); GSD lifecycle skills target the active milestone. | |
| Split: STATE.md in Phase 16, MILESTONES.md in Phase 17 prelude | SPEC R6 requires both files in this phase. | |

**User's choice:** Inside Phase 16 as R6.

---

## 15-REVIEW.md annotation format

### Q1 — Where in each CR/WR finding does the Status: line go?

| Option | Description | Selected |
|--------|-------------|----------|
| Inline at top of each finding (Recommended) | Status: line as the first content line under the finding's header. Reader sees status before re-reading the original analysis. Single grep matches all four IDs. | ✓ |
| Trailing 'Resolution Log' section at end of file | Append a single section listing each finding with status. Worse for readers scanning a single finding. | |
| Both — inline status line + summary log | Belt-and-braces; double the maintenance for small win. | |

**User's choice:** Inline at top of each finding.

### Q2 — What exactly does each Status: line contain?

| Option | Description | Selected |
|--------|-------------|----------|
| Status + commit hash + 1-line rationale (Recommended) | CR-01: Status: FIXED in 5902a81 — ... CR-02: Status: FIXED in 5902a81 — ... WR-01/WR-02: Status: RESOLVED in Phase 16 commit <hash> — ... | ✓ |
| Status + hash only, no rationale | Cleaner but loses the 'why' that makes 15-REVIEW.md useful for audit. | |
| Status + rationale only, no hash | Loses traceability; SPEC explicitly mentions hash. | |

**User's choice:** Status + commit hash + 1-line rationale.

### Q3 — How should IN-01 / IN-02 / IN-03 be annotated?

| Option | Description | Selected |
|--------|-------------|----------|
| Add explicit deferred status lines (Recommended) | IN-01: DEFERRED to Phase 19. IN-02/IN-03: DEFERRED — ad-hoc when surrounding code is touched. Doc fully self-describing. | ✓ |
| Leave IN-* unannotated since SPEC only requires CR/WR | Strict SPEC reading. But SPEC Boundaries already defines IN-* targets — keep source of truth in one place. | |
| Annotate IN-01 only (the one with a target phase) | Half-measure; IN-02/IN-03 stay dangling as 'open info-only'. | |

**User's choice:** Add explicit deferred status lines.

### Q4 — Where does the second-pass /gsd-code-review 15 output land?

| Option | Description | Selected |
|--------|-------------|----------|
| Appended as a 'Second Pass (Phase 16 close-out)' section at the bottom of 15-REVIEW.md (Recommended) | One file per phase's review history; reader sees first-pass + second-pass in chronological order. | ✓ |
| Separate file: 15-REVIEW-PASS2.md | Cleanly separated timestamps; SPEC permits this branch. | |
| /gsd-code-review 15 default output location | Defer to the skill's convention. | |

**User's choice:** Appended as a section at bottom of 15-REVIEW.md.

---

## Claude's Discretion

- Exact commit message wording (`fix(phase-16):` / `chore(phase-16):` / `docs(phase-16):` prefixes per existing project convention).
- Whether to introduce a small helper closure `let refetch_all = move || { ... }` for the three sync-handler refetch calls (mild DRY win; not required).
- Whether the second-pass code-review section in `15-REVIEW.md` is added in the same commit as the inline Status: annotations or a separate `docs(phase-16):` commit.
- Exact wording of the v1.2 MILESTONES.md entry (mirror the existing v1.0 / v1.1 entries' format).
- The exact ISO date written into `STATE.md` `prior_milestone.status` ("Shipped 2026-05-DD") — fill at commit time.

## Deferred Ideas

- IN-01 hardcoded Data Dragon patch (`src/pages/stats.rs`) — Phase 19 (Production Hardening).
- IN-02 `unwrap_or(None)` anti-pattern at `db.rs:4832` — info-only; ad-hoc.
- IN-03 spurious `#[allow(dead_code)]` on `trends_window_to_cutoff` — info-only; ad-hoc.
- Old Phase 16 (Draft Integration) — renumbered Phase 24 in v1.4 milestone.
- `refetch_all` helper closure — planner/executor's call.
- Audit other `db.rs` functions for similar dead-code patterns — out of scope; ad-hoc.
- `/gsd-debug 16` for non-obvious regressions — escape hatch only.
- Phase 17+ v1.3 launch work — owned by their own phases.
