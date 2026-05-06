# Phase 16 — Phase 15 Close-out (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-spec-phase 16` (or `/gsd-discuss-phase 16` if skipping SPEC) to expand into a full phase context.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

Resolve all open issues from the Phase 15 code review so v1.2 closes cleanly before launch-readiness work begins.

## Why this phase exists

Phase 15 (Goals & LP History) shipped UAT-passed at commit `4856140`. The code review (`15-REVIEW.md`) initially flagged 2 critical bugs and 2 warnings; commit `5902a81` already fixed the two criticals (variable rename revert + `snapshotted_at` in SELECT), but `15-REVIEW.md` was not updated to mark them resolved. UAT 11/11 confirms the criticals are gone in the running app.

What's left for v1.2 to close cleanly are the two warnings, plus the second-pass review/verify.

## State verification (2026-05-06, against main `4856140`)

| ID | Severity | Status | Evidence |
|---|---|---|---|
| CR-01 | Critical | ✅ **FIXED** in `5902a81` | `src/pages/solo_dashboard.rs:565,567` declare `snaps_for_hover` / `points_for_hover` (no underscore); hydrate closure compiles cleanly; UAT passed |
| CR-02 | Critical | ✅ **FIXED** in `5902a81` | `src/server/db.rs:4582` has `<string>snapshotted_at AS snapshotted_at` in SELECT list; comment at line 4578 confirms Rule 40 awareness |
| WR-01 | Warning | ❌ **OPEN** | `src/pages/solo_dashboard.rs:249,276` only call `dashboard_resource.refetch()`; `lp_history_resource` and `goal_progress_resource` not refetched after sync |
| WR-02 | Warning | ❌ **OPEN** | `src/server/db.rs:4699` `pub async fn get_personal_goals` defined but `grep -rn get_personal_goals src/` finds no production caller — only the definition itself |

## In-scope (must fix)

| ID | Severity | Summary | Likely fix location |
|---|---|---|---|
| WR-01 | Warning | LP history + goal progress resources not refetched after sync. Stale until reload. | `src/pages/solo_dashboard.rs:249,276` — also call `lp_history_resource.refetch()` and `goal_progress_resource.refetch()` after sync completes |
| WR-02 | Warning | `get_personal_goals` orphan public DB function with no production caller. Either wire it in (if needed) or remove it. | `src/server/db.rs` |
| 2nd review | — | `/gsd-code-review 15` second pass on commits since the original review (`5902a81`, `4856140`) | — |
| Verify | — | `/gsd-verify-work 15` against ROADMAP success criteria (1-4) | — |

## Out of scope (deferred to other v1.3 phases)

- Hardcoded Data Dragon patch `15.6.1` → **Phase 19 (Production Hardening)**
- `unwrap_or(None)` anti-pattern (info-only) — handle ad-hoc as encountered
- Spurious `#[allow(dead_code)]` on `trends_window_to_cutoff` — handle ad-hoc as encountered
- Old Phase 16 (Draft Integration) — renumbered Phase 24, deferred to v1.4

## Success criteria (verify with `/gsd-verify-work 16`)

1. After a sync, LP graph and goal cards refetch automatically without page reload (WR-01)
2. `get_personal_goals` either has a production caller or is removed (WR-02)
3. `/gsd-code-review 15` second pass produces no new HIGH/Critical findings
4. `/gsd-verify-work 15` confirms ROADMAP success criteria 1-4 still pass after the warning fixes
5. `15-REVIEW.md` updated to reflect that all flagged issues are resolved (or note explicit waivers with rationale)
6. STATE.md and MILESTONES.md mark v1.2 fully closed

## Required reading before discuss-phase

1. `.planning/phases/15-goals-lp-history/15-REVIEW.md` — full review notes (severity, recommended fixes)
2. `.planning/phases/15-goals-lp-history/.continue-here.md` — issues moved to this phase, historical context
3. `.planning/phases/15-goals-lp-history/15-02-SUMMARY.md` — LP graph + hover implementation details (SVG hit-testing decisions, web-sys features)
4. `.claude/rules/surreal-patterns.md` Rule 40 specifically (ORDER BY field-in-SELECT requirement)
5. `.claude/rules/wasm-patterns.md` Rule 35 (don't `.unwrap()` in WASM event handlers) — relevant to the hover closure fix

## Estimated scope

Tiny phase (smaller than initially scoped — the criticals are already fixed). Likely 1 plan. No DB schema changes. No new pages/routes. Touches `solo_dashboard.rs` (refetch calls) and `db.rs` (orphan removal/wiring) only.

## Plans

TBD — produced by `/gsd-plan-phase 16`.

## Notes for executor

- The fix for CR-01 may surface a related stale-capture issue (per `.claude/rules/wasm-patterns.md` rule 56 — closures capturing signals lazily). Read carefully before fixing the variable name; the rename alone may not fix the bug if the root cause is timer/closure lifecycle.
- For WR-01, the existing pattern in this codebase uses `Resource::refetch()` after server-fn mutation — see `champion_pool.rs` or `action_items.rs` for examples.
- For WR-02: search for callers with `grep -r "get_personal_goals" src/`. If none exist, remove the function. If a caller exists but is dead, remove the caller too.

---

This SPEC was seeded by the pivot. It is intentionally unopinionated — `/gsd-spec-phase 16` may expand it with ambiguity scoring or `/gsd-discuss-phase 16` will turn it into a full CONTEXT document.
