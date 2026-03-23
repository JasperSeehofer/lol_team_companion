---
phase: 10
slug: opponents-redesign
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + Playwright e2e |
| **Config file** | `Cargo.toml` (features: ssr), `e2e/playwright.config.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib && cd e2e && npx playwright test` |
| **Estimated runtime** | ~30 seconds (unit), ~60 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr --lib && cd e2e && npx playwright test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | OPP-01 | unit | `cargo test --features ssr --lib opponent` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | OPP-01 | unit | `cargo test --features ssr --lib opponent` | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 1 | OPP-02 | unit | `cargo test --features ssr --lib intel` | ❌ W0 | ⬜ pending |
| 10-02-02 | 02 | 1 | OPP-03 | unit | `cargo test --features ssr --lib mastery` | ❌ W0 | ⬜ pending |
| 10-03-01 | 03 | 2 | OPP-01 | e2e | `cd e2e && npx playwright test opponents` | ❌ W0 | ⬜ pending |
| 10-03-02 | 03 | 2 | OPP-04 | unit | `cargo test --features ssr --lib pool_analysis` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/db_opponents.rs` — integration tests for new opponent CRUD (5-role form, batch create)
- [ ] Unit tests in `src/server/db.rs` or `src/models/opponent.rs` for OTP detection, pool analysis computation
- [ ] `e2e/tests/opponents.spec.ts` — e2e test for opponent form submission, player card display

*Existing test infrastructure (cargo test + Playwright) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OTP badge visual styling (orange/yellow) | OPP-02 | CSS visual check | Navigate to opponent detail with OTP player, verify badge color |
| Mastery badge display on champion pills | OPP-03 | Visual layout check | View player card with mastery data, verify M7/M5 badges |
| Data recency stale-orange color at 7+ days | OPP-03 | Time-dependent visual | Mock old `last_fetched`, verify orange color |
| Pool analysis collapsible section animation | OPP-04 | Visual/interaction | Click to expand/collapse, verify smooth transition |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
