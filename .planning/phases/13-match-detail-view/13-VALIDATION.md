---
phase: 13
slug: match-detail-view
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust, built-in) |
| **Config file** | Cargo.toml `[features]` — `ssr` feature gates server tests |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr --lib && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | MATCH-01 | unit | `cargo test --features ssr --lib match_detail` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | MATCH-02 | unit | `cargo test --features ssr --lib timeline` | ❌ W0 | ⬜ pending |
| 13-01-03 | 01 | 1 | MATCH-04 | unit | `cargo test --features ssr --lib match_detail` | ❌ W0 | ⬜ pending |
| 13-02-01 | 02 | 2 | MATCH-01 | e2e | `cd e2e && npx playwright test match-detail` | ❌ W0 | ⬜ pending |
| 13-02-02 | 02 | 2 | MATCH-02 | e2e | `cd e2e && npx playwright test match-detail` | ❌ W0 | ⬜ pending |
| 13-02-03 | 02 | 2 | MATCH-03 | e2e | `cd e2e && npx playwright test match-detail` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/server/db.rs` — unit tests for match_detail CRUD and timeline event queries
- [ ] `e2e/tests/match-detail.spec.ts` — e2e stubs for scoreboard, timeline, performance breakdown
- [ ] Existing test infrastructure covers framework — no new installs needed

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Item icons render correctly from Data Dragon CDN | MATCH-01 | Visual verification of correct icon mapping | Navigate to match detail, verify 6 item slots show correct icons |
| Timeline events display in chronological order with correct icons | MATCH-02 | Visual layout verification | Open match detail, scroll timeline, verify event ordering and icon display |
| Performance breakdown percentages are visually clear | MATCH-03 | Visual/UX verification | Open match detail, check damage share %, vision score, CS comparison bars |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
