---
phase: 5
slug: post-game-loop-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[cfg(test)]` unit tests + Playwright e2e |
| **Config file** | `playwright.config.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `just e2e` (requires running server) |
| **Estimated runtime** | ~30 seconds (unit), ~120 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr --lib && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full `just e2e` must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | PIPE-02 | unit | `cargo test --features ssr --lib batch_create` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | PIPE-02 | unit | `cargo test --features ssr --lib batch_create_dedup` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | PIPE-02 | unit | `cargo test --features ssr --lib create_review_returns_count` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 2 | UX-01 | e2e | `just e2e -- --grep "empty state"` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | UX-02 | e2e smoke | `just e2e -- --grep "skeleton"` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | UX-03 | e2e | `just e2e -- --grep "toast"` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | UX-03 | e2e | `just e2e -- --grep "error toast"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for `batch_create_action_items_from_review` in `src/server/db.rs` test block
- [ ] Playwright tests for empty states and toast behavior in `e2e/tests/polish.spec.ts`

*Existing infrastructure covers test framework and config — only test files are needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Skeleton visual fidelity | UX-02 | Shape-matching layout is subjective | Navigate to each page, verify skeleton approximates final content shape |
| Toast positioning/animation | UX-03 | CSS animation timing is visual | Trigger save, verify toast appears top-right, auto-dismisses after 4s |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
