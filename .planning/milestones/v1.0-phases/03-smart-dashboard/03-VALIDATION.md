---
phase: 03
slug: smart-dashboard
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`) + Playwright (e2e) |
| **Config file** | `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib && npx playwright test` |
| **Estimated runtime** | ~30 seconds (unit) + ~60 seconds (e2e) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr --lib`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | INTL-01 SC-1 | e2e | `npx playwright test audit-team.spec.ts` | Partial | ⬜ pending |
| 03-01-02 | 01 | 1 | INTL-01 SC-2 | e2e | `npx playwright test audit-team.spec.ts` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 2 | INTL-01 SC-3 | e2e | `npx playwright test audit-team.spec.ts` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 2 | INTL-01 SC-4 | code review | N/A (structural) | N/A | ⬜ pending |
| 03-02-03 | 02 | 2 | INTL-01 SC-5 | e2e | `npx playwright test audit-team.spec.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows action items count panel" using `teamPage` fixture
- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows post-game panel (empty CTA)" using `teamPage` fixture
- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows pool gap panel (empty CTA)" using `teamPage` fixture

*The `teamPage` fixture always starts with a new team with no data — perfect for empty-state testing.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Each panel loads independently (Suspense isolation) | INTL-01 SC-4 | Structural code review — separate Resources/Suspense boundaries | Verify each panel has its own `Resource::new` at component top level and own `<Suspense>` boundary |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
