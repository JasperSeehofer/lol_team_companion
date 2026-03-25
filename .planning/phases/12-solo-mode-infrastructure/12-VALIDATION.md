---
phase: 12
slug: solo-mode-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `Cargo.toml` `[features]` section — `ssr` feature enables server tests |
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
| 12-01-01 | 01 | 1 | SOLO-01 | unit | `cargo test --features ssr --lib mode` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | SOLO-02 | unit | `cargo test --features ssr --lib region` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 1 | SOLO-03 | unit | `cargo test --features ssr --lib solo_sync` | ❌ W0 | ⬜ pending |
| 12-02-02 | 02 | 1 | RANK-01 | unit | `cargo test --features ssr --lib ranked` | ❌ W0 | ⬜ pending |
| 12-02-03 | 02 | 1 | RANK-03 | unit | `cargo test --features ssr --lib ranked_snapshot` | ❌ W0 | ⬜ pending |
| 12-03-01 | 03 | 2 | SOLO-04 | unit | `cargo test --features ssr --lib queue_filter` | ❌ W0 | ⬜ pending |
| 12-03-02 | 03 | 2 | SOLO-05 | e2e | `cd e2e && npx playwright test solo` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit test stubs for mode toggle, region selection, solo sync, ranked data in `src/server/db.rs` `#[cfg(test)]` blocks
- [ ] E2e test stub `e2e/tests/solo-dashboard.spec.ts` for SOLO-05

*Existing test infrastructure (cargo test + Playwright) covers framework needs — no new dependencies.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Mode toggle persists across hard navigation | SOLO-01 | Requires browser session + page reload | 1. Login 2. Toggle to solo mode 3. Refresh page 4. Verify mode still shows "solo" |
| Ranked badge renders correct tier icon | RANK-01 | Visual verification of SVG/image | 1. Sync ranked data 2. Navigate to solo dashboard 3. Verify tier badge matches API data |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
