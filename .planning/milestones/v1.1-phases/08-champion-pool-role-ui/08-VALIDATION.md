---
phase: 8
slug: champion-pool-role-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 8 — Validation Strategy

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
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | UX-08 | e2e | `cd e2e && npx playwright test -g "champion pool"` | ❌ W0 | ⬜ pending |
| 08-01-02 | 01 | 1 | UX-08 | e2e | `cd e2e && npx playwright test -g "drag"` | ❌ W0 | ⬜ pending |
| 08-01-03 | 01 | 1 | UX-08 | e2e | `cd e2e && npx playwright test -g "matchup"` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 1 | UX-10 | unit+e2e | `cargo test --features ssr --lib role` | ❌ W0 | ⬜ pending |
| 08-02-02 | 02 | 1 | UX-10 | e2e | `cd e2e && npx playwright test -g "draft role"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/champion-pool.spec.ts` — stubs for UX-08 (larger icons, drag-and-drop, matchup rework)
- [ ] `e2e/tests/draft-roles.spec.ts` — stubs for UX-10 (role assignment, auto-guess)

*Existing unit test infrastructure covers model-level tests. E2e stubs needed for UI verification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Drag-and-drop visual feedback | UX-08 | CSS opacity/hover effects hard to assert | Drag champion between tiers, verify visual cue appears |
| Role icon popover positioning | UX-10 | Popover layout depends on viewport | Click role icon on pick slot, verify popover appears near slot |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
