---
phase: 15
slug: goals-lp-history
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-05
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test --features ssr --lib` (Rust unit + integration tests against SurrealDB in-memory; Playwright for e2e) |
| **Config file** | `Cargo.toml`, `e2e/playwright.config.ts`, `tests/common/mod.rs` (test_db helper) |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `just verify` (cargo check ssr+wasm + test + clippy + fmt) |
| **Estimated runtime** | ~60 seconds (lib tests) / ~3 minutes (full verify) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib` (lib unit + integration tests)
- **After every plan wave:** Run `just verify`
- **Before `/gsd-verify-work`:** Full suite + e2e smoke (`just smoke`) must be green
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

> Filled in after planner emits PLAN.md files. Until then, this section enumerates the validation requirements derived from RESEARCH.md and CONTEXT.md decisions.

### Validation Requirements (from RESEARCH.md `## Validation Architecture`)

| Validation Area | Requirement | Test Type | Notes |
|-----------------|-------------|-----------|-------|
| LP rank-score formula | Iron 4 0LP=0, Diamond 1 99LP=2799, Master 0LP=2800 | unit (model) | `tests/db_ranked_snapshot.rs` or unit in models |
| LP rank-score boundary | Diamond I 99 LP → Master 0 LP transitions monotonically | unit | Edge case: tier upgrade with no LP reset signal |
| LP rank-score Master+ | Master 250 LP and Grandmaster 250 LP both = 3050 — verify no double-counting on tier escalation | unit | Cumulative scale must NOT add tier offset for Master/GM/Challenger |
| Goal progress: zero games | User with 0 solo/duo games → returns "insufficient data, 0 of 5 needed" | integration (db) | Empty-state flow |
| Goal progress: < 5 games | User with 3 solo/duo games → returns "insufficient data, 3 of 5 needed" | integration (db) | Below-threshold flow |
| Goal progress: ≥ 5 games | User with 7 solo/duo games → returns aggregate (avg CS/min, avg deaths) | integration (db) | At-threshold flow |
| Goal progress: ≥ 20 games | User with 35 solo/duo games → aggregate uses last 20 only (not all 35) | integration (db) | Window cap |
| Goal progress: queue filter | User with 10 solo/duo + 10 flex matches → only solo/duo (queue_id=420) feed goal averages | integration (db) | Queue isolation |
| CS/min computation | `cs / (game_duration / 60.0)` — game_duration is seconds, divide by 60 | unit | RESEARCH.md flagged as the most likely silent bug |
| Champion trends: time window | 7d/30d/90d/all-time filters return correctly bounded subsets | integration (db) | Window correctness |
| Champion trends: min-3-games filter | Champions with 1-2 games hidden by default; "Show all" reveals them | unit (Memo derivation) | Client-side filter |
| Champion trends: queue source | solo/duo + flex (queue_id IN [420, 440]) — ARAM/normals excluded | integration (db) | Queue scope correctness |
| Champion trends: KDA formula | `(K+A)/max(D,1)` handles zero-deaths gracefully (no divide-by-zero) | unit | Edge case |
| personal_goal upsert | Saving twice with same (user, goal_type) updates row, does not create duplicate | integration (db) | Unique-index correctness |
| personal_goal: cross-user isolation | User A's goals do not appear in User B's progress query | integration (db) | Auth isolation |
| Time-window toggle: client-side | LP graph + champion trends both filter already-fetched data on toggle change (no requery) | unit (Memo) | Performance contract |
| LP graph: empty state | User with 0 ranked snapshots → empty-state UI with Sync CTA | e2e or component | UX contract |
| Tier emblem text labels | Y-axis renders text-only labels at tier boundaries (no images) | component / visual review | Per UI-SPEC |

> Per-task IDs (e.g., `15-01-01`) will be filled by the planner after PLAN.md files exist.

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/db_personal_goal.rs` — integration test stubs for personal_goal CRUD + upsert + cross-user isolation
- [ ] `tests/db_goal_progress.rs` — integration test stubs for goal-progress aggregation across the 5 game-count flows
- [ ] `tests/db_champion_trends.rs` — integration test stubs for champion trends time-window and queue-source filtering
- [ ] `tests/db_lp_history.rs` — integration test stubs for LP history time-window queries
- [ ] Unit tests for `rank_score()` helper — boundary cases (Iron 4, Diamond 1 99 LP, Master 0/250 LP, Grandmaster 250 LP)
- [ ] Unit tests for `kda_ratio()` helper — zero-deaths, all-zeros, normal cases
- [ ] Unit tests for CS/min computation helper — verify division by 60, not /60.0 truncation

*Existing infrastructure (cargo test --features ssr, tests/common/mod.rs) already covers the framework. New test files are stubs that the executor fills in.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| LP graph hover tooltip | RANK-02 / D-04 | Mouse-coordinate hit testing in inline SVG is hard to assert via unit test; visual confirmation faster | Open `/solo/dashboard` → hover over LP graph data points → verify tooltip shows tier/division/LP/timestamp |
| LP graph cumulative-LP scale visual continuity | RANK-02 / D-03 | Visual continuity across promo/relegation requires eyeballing the line | Open `/solo/dashboard` with seeded data spanning Diamond → Master → verify no visual jumps in line |
| Goal card inline edit affordance | LEARN-04 / D-11 | Animation/expand UX is best confirmed visually | Click "Edit Goal" on a goal card → verify form swap-in (no modal) → save/discard flow |
| Champion trends sortable column visual feedback | LEARN-06 / D-16 | Sort indicator (arrow/highlight) is a visual contract | Click table headers → verify active-sort indicator and stable row identity |

*Most behaviors are automated. Manual checks supplement visual UX validation that's expensive to encode.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (filled when planner emits tasks)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (filled when planner emits tasks)
- [ ] Wave 0 covers all MISSING references (4 new test files + 3 unit-test helpers)
- [ ] No watch-mode flags (use `cargo test`, not `cargo watch test`)
- [ ] Feedback latency < 60s for `cargo test --features ssr --lib`
- [ ] `nyquist_compliant: true` set in frontmatter (after planner emits per-task verify map)

**Approval:** pending
