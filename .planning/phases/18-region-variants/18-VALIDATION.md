---
phase: 18
slug: region-variants
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright 1.58.2 (e2e); `cargo test --features ssr --lib` (unit) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Full suite command** | `cargo test --features ssr --lib && cd e2e && npx playwright test` |
| **Estimated runtime** | ~180 seconds (full); ~30 seconds (quick compile check) |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cargo test --features ssr --lib`
- **Before `/gsd-verify-work`:** Full suite must be green (`cargo test --features ssr --lib && cd e2e && npx playwright test`)
- **Max feedback latency:** 30 seconds (quick); 180 seconds (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 18-01 | 01 | 1 | REQ-1 | — | N/A | compile | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| 18-02 | 02 | 1 | REQ-2 | — | N/A | compile + visual | `cargo check --features ssr` | ✅ | ⬜ pending |
| 18-03 | 03 | 2 | REQ-3 | — | N/A | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ | ⬜ pending |
| 18-04 | 04 | 2 | REQ-3 | — | N/A | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ | ⬜ pending |
| 18-05 | 05 | 2 | REQ-3 | — | N/A | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ | ⬜ pending |
| 18-06 | 06 | 3 | REQ-3 | — | N/A | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ | ⬜ pending |
| 18-07 | 07 | 3 | REQ-4 | — | N/A | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ | ⬜ pending |
| 18-08 | 08 | 3 | REQ-5 | — | N/A | e2e | `cd e2e && npx playwright test theme.spec.ts` | ✅ | ⬜ pending |
| 18-09 | 09 | 4 | REQ-6 | — | N/A | visual | `cd e2e && npx playwright test region-diff.spec.ts` | ❌ W0 | ⬜ pending |
| 18-10 | 10 | 5 | REQ-8 | — | N/A | manual audit | `ls .planning/phases/18-region-variants/18-UI-REVIEW.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/region-diff.spec.ts` — new Playwright spec using pixelmatch; asserts `pixelDiffRatio(demacia, pandemonium) > 0.40` per scoped route (REQ-6)
- [ ] Install `pixelmatch` + `pngjs` for manual pixel comparison in region-diff spec: `npm install --save-dev pixelmatch pngjs @types/pngjs` inside `e2e/`

*All other test infrastructure exists. Existing `visual-regression.spec.ts`, `theme.spec.ts`, and `cargo test` cover the remaining requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 18-UI-REVIEW.md 6-pillar audit for both regions | REQ-8 | Requires gsd-ui-auditor agent + visual inspection | Spawn `/gsd-ui-review` after 18-09 baselines are committed; both regions must have no FAIL ratings |
| REQ-7: Utility routes have zero new region conditionals | REQ-7 | Grep check at commit time | `grep -rE "is_pandemonium\|theme == \"pandemonium\"" src/pages/auth/ src/pages/profile.rs src/pages/stats.rs` — must return zero matches |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`region-diff.spec.ts`)
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
