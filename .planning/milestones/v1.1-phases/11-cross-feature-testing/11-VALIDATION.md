---
phase: 11
slug: cross-feature-testing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Playwright (e2e) |
| **Config file** | `Cargo.toml` (test features), `e2e/playwright.config.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib && cd e2e && npx playwright test` |
| **Estimated runtime** | ~30 seconds (unit) + ~60 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr --lib && cd e2e && npx playwright test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | XFEAT-01 | unit | `cargo test --features ssr --lib post_game` | ✅ | ⬜ pending |
| 11-01-02 | 01 | 1 | XFEAT-01 | unit | `cargo test --features ssr --lib analytics` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 2 | XFEAT-01 | e2e | `cd e2e && npx playwright test pages.spec.ts` | ✅ | ⬜ pending |
| 11-03-01 | 03 | 2 | TEST-01 | integration | `cargo run --features ssr --bin seed` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/pages/analytics.rs` — new page stub for analytics route
- [ ] `e2e/tests/pages.spec.ts` — add `/analytics` to AUTHED_PAGES array
- [ ] Schema fields for `win_loss` and `rating` on `post_game_learning` table

*Existing test infrastructure (cargo test + Playwright) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Strategy tag card visual layout | XFEAT-01 | Visual design check | Navigate to /analytics with seeded data, verify cards show win rate + avg rating per tag |
| Accordion expansion animation | XFEAT-01 | Animation timing | Click game plan row, verify smooth accordion with review details |
| Seed data completeness | TEST-01 | Data quality check | Run seed script, browse all pages, verify realistic data across features |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
