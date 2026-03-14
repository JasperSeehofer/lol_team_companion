---
phase: 2
slug: aggregation-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]` / `#[tokio::test]`) |
| **Config file** | `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr --lib` |
| **Estimated runtime** | ~30 seconds |

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
| 02-01-01 | 01 | 1 | SC-1 | unit/integration | `cargo test --features ssr --lib -- db::tests::test_dashboard_summary` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | SC-1 | unit/integration | `cargo test --features ssr --lib -- db::tests::test_dashboard_summary_empty` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | SC-2 | unit/integration | `cargo test --features ssr --lib -- db::tests::test_champion_perf_summary` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | SC-3 | unit | `cargo test --features ssr --lib -- data_dragon::tests::test_normalize` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | SC-4 | unit/integration | Covered in SC-1 empty test | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | SC-5 | integration | `cargo test --features ssr --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Integration test helpers in `db.rs` tests mod — `create_test_db()` using `surrealdb::engine::local::Mem`
- [ ] `tests::test_dashboard_summary` — covers SC-1
- [ ] `tests::test_dashboard_summary_empty` — covers SC-1 empty case
- [ ] `tests::test_champion_perf_summary` — covers SC-2
- [ ] `data_dragon::tests::test_normalize` — covers SC-3

*Existing infrastructure covers framework installation; only test stubs are needed.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
