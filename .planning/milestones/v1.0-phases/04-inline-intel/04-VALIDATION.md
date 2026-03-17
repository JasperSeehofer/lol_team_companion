---
phase: 4
slug: inline-intel
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + cargo test --features ssr --lib (unit) |
| **Config file** | `playwright.config.ts` (root), `e2e/tests/fixtures.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `just e2e` |
| **Estimated runtime** | ~30 seconds (unit), ~60 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `just e2e`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 0 | PIPE-03 | unit | `cargo test --features ssr --lib -- compute_slot_warnings` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | PIPE-03 | unit | `cargo test --features ssr --lib -- compute_slot_warnings` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | PIPE-03 | smoke | `just e2e -- --grep "draft"` | ✅ | ⬜ pending |
| 04-02-01 | 02 | 0 | PIPE-04 | unit | `cargo test --features ssr --lib -- opponent_intel_no_key` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | PIPE-04 | smoke | `just e2e -- --grep "draft"` | ✅ | ⬜ pending |
| 04-03-01 | 03 | 1 | INTL-02 | unit | `cargo test --features ssr --lib -- win_condition_stats` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 1 | INTL-02 | e2e | `just e2e -- --grep "game-plan"` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/server/db.rs` test: `test_compute_slot_warnings` — unit test for the pure function mapping draft slots to pool warnings (covers PIPE-03)
- [ ] `src/server/db.rs` test: `test_win_condition_stats_empty` — edge case: no post-games linked to plans (covers INTL-02)
- [ ] `src/server/riot.rs` test: cannot unit test without live API key — acceptance via pages.spec.ts smoke + graceful degradation check

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Opponent tendency sidebar displays correctly | PIPE-04 | Requires Riot API data + visual layout check | Navigate to draft page with opponent set, verify sidebar shows frequency counts |
| Intel panels don't cause reactive refetches | SC-4 | Performance behavior, not functional output | Open draft page, pick champions, verify network tab shows no extra API calls |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
