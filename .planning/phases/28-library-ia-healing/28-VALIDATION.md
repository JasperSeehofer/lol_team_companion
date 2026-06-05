---
phase: 28
slug: library-ia-healing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-06
---

# Phase 28 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + Rust `#[cfg(test)]` unit tests |
| **Config file** | `e2e/playwright.config.ts` (existing) |
| **Quick run command** | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Full suite command** | `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts` |
| **Estimated runtime** | ~60–120 seconds (e2e subset); ~30s checks |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts`
- **Before `/gsd:verify-work`:** Full e2e suite green (`cd e2e && npx playwright test`); `cargo test --features ssr --lib` green
- **Max feedback latency:** ~120 seconds

---

## SC → Test Map

| SC | Behavior | Test Type | Automated Command | File Exists |
|----|----------|-----------|-------------------|-------------|
| SC1 | `/library` loads with tab strip visible, no JS errors; tabs switch without atmosphere remount | smoke | `cd e2e && npx playwright test pages.spec.ts -g "/library"` | ❌ W0 — add `/library` to `AUTHED_PAGES` |
| SC2 | `/personal-learnings`, `/stats`, `/analytics`, `/post-game` resolve via redirect (not 404) | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ W0 — update those entries to expect redirect destination |
| SC3 | `/opponents` redirects into War Room (`/war-room?tab=opponents`); `/personal-learnings/new` still loads create form | smoke | `cd e2e && npx playwright test pages.spec.ts -g "opponents\|personal-learnings/new"` | ⚠️ partial — `/personal-learnings/new` exists; `/opponents` redirect entry needs update |
| SC4 | Tier-2 team emits `data-records-tier="2"` (and tomes/analysis) on atmosphere root; fresh team emits tier 0 | unit + e2e | `cargo test --features ssr --lib -- growth` + `cd e2e && npx playwright test pages.spec.ts -g "library growth"` | ❌ W0 — add `library_*_tier` tests to `growth.rs` |
| SC5 | `/library` hydrates without tachys panic in both demacia and pandemonium regions | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/library"` | ❌ W0 — add `/library` to `SCOPED_ROUTES` |

---

## Per-Task Verification Map

> Concrete task IDs are assigned by the planner. This map records the standing per-requirement verification contract the planner must honor when emitting tasks.

| Plan area | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|-----------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| Growth model (`growth.rs` axes + tiers) | 1 | ROOM-05 | — | N/A | unit | `cargo test --features ssr --lib -- growth` | ❌ W0 | ⬜ pending |
| `get_library_growth_state` server fn | 1 | ROOM-05 | T-28-V4 | Validates team membership via `get_user_team_id` before query; no-team → `Ok(default)` not `Err` | unit/compile | `cargo check --features ssr` | ✅ | ⬜ pending |
| Surface extractions (5× `*Surface`) | 1–2 | ROOM-05 | — | N/A | compile | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| `LibraryRoom` + `LibrarySurfaces` compose | 2 | ROOM-05 | T-28-tamper | Region read once at top (gotcha #58); asset region server-set | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/library"` | ❌ W0 | ⬜ pending |
| Flat-route redirects (`app.rs`) | 2 | ROOM-07 | T-28-V5 | `/match/:id` id preserved; `/personal-learnings/new` intact; redirect uses authorized params | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ W0 | ⬜ pending |
| e2e wiring (`pages.spec.ts`, `hydration-no-panic.spec.ts`) | Final | ROOM-05, ROOM-07 | — | N/A | smoke+canary | `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/pages.spec.ts` — add `/library` to `AUTHED_PAGES`; update/remove the 5 redirected route entries (`/stats`, `/analytics`, `/post-game`, `/personal-learnings`, `/opponents`) to expect their redirect destination; keep `/personal-learnings/new` unchanged
- [ ] `e2e/tests/hydration-no-panic.spec.ts` — add `/library` to `SCOPED_ROUTES`; keep `/match/test-match-id` (now exercises the param-carrying redirect)
- [ ] `src/models/growth.rs` — add `library_records_tier`, `library_tomes_tier`, `library_analysis_tier` unit tests following the existing `tier_for` test pattern (lines ~229–282)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Atmosphere overlays visibly fade in by tier (placeholder gradients) | ROOM-05 (SC4) | Visual opacity differences with placeholder art are not pixel-asserted; the `data-*-tier` attribute is the automated proxy | Seed a Tier-2 team vs a fresh team; load `/library` for each; confirm `data-records-tier`/`data-tomes-tier`/`data-analysis-tier` differ and overlay opacity visibly increases |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (pages.spec.ts, hydration-no-panic.spec.ts, growth.rs tests)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
