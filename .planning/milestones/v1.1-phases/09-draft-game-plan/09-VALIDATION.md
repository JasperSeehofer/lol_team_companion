---
phase: 9
slug: draft-game-plan
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + cargo test (unit) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Full suite command** | `cd e2e && npx playwright test` |
| **Estimated runtime** | ~45 seconds (checks) / ~90 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cargo test --features ssr --lib`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | DRFT-01 | e2e | `npx playwright test audit-draft.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-01-02 | 01 | 1 | DRFT-02 | e2e | `npx playwright test audit-draft.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-01-03 | 01 | 1 | DRFT-05 | unit | `cargo test --features ssr --lib` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 1 | DRFT-03 | e2e | `npx playwright test audit-draft.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-02-02 | 02 | 1 | DRFT-04 | e2e | `npx playwright test audit-draft.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-02-03 | 02 | 1 | DRFT-05 | e2e | `npx playwright test audit-draft.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-03-01 | 03 | 2 | PLAN-01 | e2e | `npx playwright test audit-game-plan.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-03-02 | 03 | 2 | PLAN-03 | e2e | `npx playwright test audit-game-plan.spec.ts` | ✅ (needs new case) | ⬜ pending |
| 09-03-03 | 03 | 2 | PLAN-01 | unit | `cargo test --features ssr --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/audit-draft.spec.ts` — add test cases for DRFT-01 through DRFT-05
- [ ] `e2e/tests/audit-game-plan.spec.ts` — add test cases for PLAN-01, PLAN-03
- [ ] `src/models/draft.rs` or `src/pages/game_plan.rs` — unit test for `most_common_tag` helper
- [ ] `src/server/db.rs` — unit test for `get_pool_notes_for_champions` in `#[cfg(test)]` block

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Auto-open intel sidebar on opponent select | DRFT-01 (D-04) | Reactive signal side-effect timing | 1. Select opponent from dropdown 2. Verify intel sidebar opens automatically |
| Notes tab auto-switch on champion pick | DRFT-05 (D-13) | Async Resource refetch timing | 1. Pick a pooled champion 2. Verify Notes tab activates with correct sub-tab |
| Debounced auto-save before opponent nav | DRFT-02 (D-03) | Timer-based JS behavior | 1. Make draft changes 2. Click "Add New Opponent" 3. Verify draft saved before navigation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 45s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
