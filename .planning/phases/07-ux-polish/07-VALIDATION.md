---
phase: 7
slug: ux-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Playwright (e2e) |
| **Config file** | `Cargo.toml` / `e2e/playwright.config.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib && cd e2e && npx playwright test` |
| **Estimated runtime** | ~30 seconds (unit) + ~60 seconds (e2e) |

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
| 07-01-01 | 01 | 1 | UX-04 | e2e | `cd e2e && npx playwright test -g "toast"` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | UX-05 | unit+e2e | `cargo test --features ssr --lib format_timestamp` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 1 | UX-06 | e2e | `cd e2e && npx playwright test -g "profile"` | ❌ W0 | ⬜ pending |
| 07-04-01 | 04 | 1 | UX-07 | e2e | `cd e2e && npx playwright test -g "team search"` | ❌ W0 | ⬜ pending |
| 07-05-01 | 05 | 1 | UX-09 | e2e | `cd e2e && npx playwright test -g "roster"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/ux-polish.spec.ts` — e2e stubs for UX-04 (toast), UX-06 (profile), UX-07 (team search), UX-09 (role icons)
- [ ] Unit test for `format_timestamp()` helper in model utils

*Existing test infrastructure (cargo test + Playwright) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Toast does not overlap nav visually | UX-04 | CSS positioning visual check | Screenshot toast, verify it renders below nav header |
| Role watermark opacity/positioning | UX-09 | Visual design quality | Screenshot roster cards, verify watermark visible but subtle |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
