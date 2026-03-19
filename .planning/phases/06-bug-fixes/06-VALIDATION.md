---
phase: 6
slug: bug-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-19
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (existing in `e2e/`) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cd e2e && npx playwright test regression.spec.ts` |
| **Full suite command** | `just e2e` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd e2e && npx playwright test regression.spec.ts`
- **After every plan wave:** Run `just e2e`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | TEST-02 | manual smoke | `just e2e` (full suite uses helpers) | ❌ W0 | ⬜ pending |
| 06-02-01 | 02 | 2 | BUG-03 | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-03"` | ❌ W0 | ⬜ pending |
| 06-02-02 | 02 | 2 | BUG-04 | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-04"` | ❌ W0 | ⬜ pending |
| 06-02-03 | 02 | 2 | BUG-05 | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-05"` | ❌ W0 | ⬜ pending |
| 06-02-04 | 02 | 2 | BUG-01 | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-01"` | ❌ W0 | ⬜ pending |
| 06-02-05 | 02 | 2 | BUG-02, PLAN-02 | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-02"` | ❌ W0 | ⬜ pending |
| 06-03-01 | 03 | 3 | TEST-02 | e2e | `just e2e` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/regression.spec.ts` — covers BUG-01 through BUG-05
- [ ] `e2e/tests/helpers.ts` — navigation, error capture, interaction utilities

*Created during Plan 1 (test infrastructure).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent-browser skill works for interactive browser checks | TEST-02 | Skill requires agent runtime | Install skill, run `agent-browser navigate http://127.0.0.1:3002`, verify screenshot |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
