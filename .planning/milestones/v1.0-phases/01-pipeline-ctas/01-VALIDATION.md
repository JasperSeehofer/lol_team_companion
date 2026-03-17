---
phase: 1
slug: pipeline-ctas
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (tokio) + Playwright |
| **Config file** | `Cargo.toml` (features: ssr), `e2e/playwright.config.ts` |
| **Quick run command** | `cargo test --features ssr --lib` |
| **Full suite command** | `cargo test --features ssr` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features ssr --lib`
- **After every plan wave:** Run `cargo test --features ssr`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | PIPE-01 | integration | `cargo test --features ssr` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/db_game_plan_pipeline.rs` — integration tests for draft→game plan FK queries
- [ ] Verify `schema.surql` `draft` field type on `game_plan` table

*Existing test infrastructure (tests/common/mod.rs) covers shared fixtures.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CTA buttons render on draft list/detail | PIPE-01 | UI rendering requires browser | Navigate to /draft, verify "Prep for This Draft" button visible |
| Prefill populates game plan fields | PIPE-01 | Requires server + browser interaction | Click CTA, verify champions/side/opponent filled |
| Back-reference badges link correctly | PIPE-01 | Navigation requires browser | Click badge on game plan, verify it opens source draft |
| Direct URL `/game-plan?draft_id=X` works | PIPE-01 | URL param requires browser | Navigate directly, verify prefill |

*These can be verified via Playwright MCP during development and added to e2e suite.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
