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
| **Full suite command** | `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts library.spec.ts` |
| **Estimated runtime** | ~60–120 seconds (e2e subset); ~30s checks |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts library.spec.ts`
- **Before `/gsd:verify-work`:** Full e2e suite green (`cd e2e && npx playwright test`); `cargo test --features ssr --lib` green
- **Max feedback latency:** ~120 seconds

---

## SC → Test Map

| SC | Behavior | Test Type | Automated Command | File Exists |
|----|----------|-----------|-------------------|-------------|
| SC1 | `/library` loads with tab strip visible, no JS errors; tabs switch without atmosphere remount | smoke | `cd e2e && npx playwright test pages.spec.ts -g "/library"` | ❌ W0 — add `/library` to `AUTHED_PAGES` |
| SC2 | `/personal-learnings`, `/stats`, `/analytics`, `/post-game` resolve via redirect (not 404) | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ W0 — update those entries to expect redirect destination |
| SC3 | `/opponents` redirects into War Room (`/war-room?tab=opponents`); `/personal-learnings/new` still loads create form | smoke | `cd e2e && npx playwright test pages.spec.ts -g "opponents\|personal-learnings/new"` | ⚠️ partial — `/personal-learnings/new` exists; `/opponents` redirect entry needs update |
| SC4 | Seeded team emits `data-{records,tomes,analysis}-tier ≥ 1` with computed overlay opacity > 0 on a seeded axis; fresh/tier-0 axis is opacity 0. Tier math also unit-covered. | unit + e2e | `cargo test --features ssr --lib -- library` + `cd e2e && npx playwright test library.spec.ts -g "growth tiers"` | ❌ W0 — add `library_*` tests to `growth.rs` (Plan 01) + new `library.spec.ts` "growth tiers" seed-and-assert test (Plan 07, mirrors war-room.spec.ts) + `input.css` tier→opacity selectors (Plan 05) |
| SC5 | `/library` hydrates without tachys panic in both demacia and pandemonium regions | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/library"` | ❌ W0 — add `/library` to `SCOPED_ROUTES` |

---

## Per-Task Verification Map

> Concrete task IDs are assigned by the planner. This map records the standing per-requirement verification contract the planner must honor when emitting tasks.

| Plan area | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|-----------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| Growth model (`growth.rs` axes + tiers) | 1 | ROOM-05 | — | N/A | unit | `cargo test --features ssr --lib -- library` | ❌ W0 | ⬜ pending |
| `get_library_growth_state` server fn | 1 | ROOM-05 | T-28-V4 | Validates team membership via `get_user_team_id` before query; no-team → `Ok(default)` not `Err` | unit/compile | `cargo check --features ssr` | ✅ | ⬜ pending |
| Surface extractions (5× `*Surface`) | 1–2 | ROOM-05 | — | N/A | compile | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| `LibraryRoom` + `LibrarySurfaces` compose | 2 | ROOM-05 | T-28-tamper | Region read once at top (gotcha #58); asset region server-set | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/library"` | ❌ W0 | ⬜ pending |
| `input.css` tier→opacity selectors (records/tomes/analysis) | 2 | ROOM-05 | T-28-05-03 | Static CSS matching server-set `data-*-tier` (0..=3); no user input; makes growth visible (closes the Phase 27 SC3 gap) | e2e (computed opacity) | `cd e2e && npx playwright test library.spec.ts -g "growth tiers"` | ❌ W0 | ⬜ pending |
| Flat-route redirects (`app.rs`) | 2 | ROOM-07 | T-28-V5 | `/match/:id` id preserved; `/personal-learnings/new` intact; redirect uses authorized params | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ W0 | ⬜ pending |
| Library growth-tiers seed-and-assert (`library.spec.ts` + `seedLibraryTier1Team`) | Final | ROOM-05 | — | N/A | e2e | `cd e2e && npx playwright test library.spec.ts -g "growth tiers"` | ❌ W0 | ⬜ pending |
| e2e wiring (`pages.spec.ts`, `hydration-no-panic.spec.ts`) | Final | ROOM-05, ROOM-07 | — | N/A | smoke+canary | `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/tests/pages.spec.ts` — add `/library` to `AUTHED_PAGES`; update/remove the 5 redirected route entries (`/stats`, `/analytics`, `/post-game`, `/personal-learnings`, `/opponents`) to expect their redirect destination; keep `/personal-learnings/new` unchanged
- [ ] `e2e/tests/hydration-no-panic.spec.ts` — add `/library` to `SCOPED_ROUTES`; keep `/match/test-match-id` (now exercises the param-carrying redirect)
- [ ] `e2e/tests/library.spec.ts` — NEW (Plan 07): a `test.describe("Library")` "growth tiers" seed-and-assert test mirroring `war-room.spec.ts` — seed a Library axis to tier ≥ 1, read `data-{records,tomes,analysis}-tier` off `.room-atmosphere` (present/numeric/0..=3, max ≥ 1), then read computed `.room-overlay--{axis}` opacity (tier≥1 → >0, tier-0 → 0). Closes the Phase 27 SC3 blind spot (data attrs wired but CSS missing).
- [ ] `e2e/tests/fixtures.ts` — NEW helper `seedLibraryTier1Team` (Plan 07): pushes a Library axis (tomes/analysis via one post-game learning) to tier ≥ 1. Distinct from `seedTier1Team`, which moves only the War Room tactics axis (draft) and does NOT touch any Library axis.
- [ ] `input.css` — NEW (Plan 05): 9 Library tier→opacity selectors (`[data-records-tier="N"] .room-overlay--records { opacity: ... }` for records/tomes/analysis × tiers 1/2/3), mirroring the War Room block. Without these the `data-*-tier` attributes are inert and SC4 silently fails (the exact Phase 27 SC3 gap).
- [ ] `src/models/growth.rs` — add `library_records_tier`, `library_tomes_tier`, `library_analysis_tier` unit tests following the existing `tier_for` test pattern (lines ~229–282)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none required) | ROOM-05 (SC4) | Previously listed: "atmosphere overlays visibly fade in by tier". This is now AUTOMATED — `library.spec.ts` "growth tiers" reads computed `.room-overlay--{axis}` opacity in a real browser (tier≥1 → >0, tier-0 → 0), so the visual claim is asserted, not eyeballed. A manual spot-check remains in the Plan 05 human-verify checkpoint for defense-in-depth but is no longer the sole evidence for SC4. | Optional spot-check (Plan 05 checkpoint step 8): seed a team with one post-game review vs a fresh team; load `/library` for each; confirm a Library overlay is faintly visible (opacity > 0) for the seeded team and fully transparent for the fresh team. |

*All phase behaviors now have automated verification (the SC4 overlay-opacity visual is automated by `library.spec.ts`).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (pages.spec.ts, hydration-no-panic.spec.ts, library.spec.ts, fixtures.ts seedLibraryTier1Team, input.css selectors, growth.rs tests)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
</content>
