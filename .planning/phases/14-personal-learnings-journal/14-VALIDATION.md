---
phase: 14
slug: personal-learnings-journal
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + `cargo test --features ssr --lib` (unit) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cd e2e && npx playwright test pages.spec.ts` |
| **Full suite command** | `cd e2e && npx playwright test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cargo test --features ssr --lib && cd e2e && npx playwright test pages.spec.ts`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | LEARN-01 | unit | `cargo test --features ssr --lib personal_learning` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | LEARN-01 | smoke | `npx playwright test pages.spec.ts -g "personal-learnings"` | ❌ W0 | ⬜ pending |
| 14-02-01 | 02 | 1 | LEARN-01 | e2e | `npx playwright test audit-personal-learnings.spec.ts -g "create learning"` | ❌ W0 | ⬜ pending |
| 14-02-02 | 02 | 2 | LEARN-02 | e2e | `npx playwright test match-detail.spec.ts -g "Add Learning"` | ❌ W0 | ⬜ pending |
| 14-03-01 | 03 | 2 | LEARN-03 | e2e | `npx playwright test audit-personal-learnings.spec.ts -g "filter"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/audit-personal-learnings.spec.ts` — covers LEARN-01 (create), LEARN-03 (filter), LEARN-01 (delete)
- [ ] Add `/personal-learnings` and `/personal-learnings/new` to `AUTHED_PAGES` array in `e2e/tests/pages.spec.ts`
- [ ] Add unit test for `PersonalLearning` model round-trip in `src/models/personal_learning.rs`
- [ ] Match detail "Add Learning" test extension in `e2e/tests/match-detail.spec.ts`

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Card grid responsive layout | LEARN-02 | Visual layout verification | Browse page at 1280px and 768px — cards reflow correctly |
| Inline expand/collapse animation | LEARN-02 | Visual smoothness | Click a card, verify expansion is smooth with no layout jank |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
