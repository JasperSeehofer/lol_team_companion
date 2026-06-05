---
phase: 27
slug: war-room
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-05
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + `cargo test --features ssr --lib` (unit) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Full suite command** | `cargo test --features ssr --lib && cd e2e && npx playwright test` |
| **Estimated runtime** | ~120 seconds (unit ~10s, e2e ~110s) |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **After every plan wave:** Run `cargo test --features ssr --lib && cd e2e && npx playwright test pages.spec.ts`
- **Before `/gsd:verify-work`:** Full suite must be green (`cd e2e && npx playwright test`)
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC1 | war-room compose | — | ROOM-04 | — | N/A | e2e smoke | `cd e2e && npx playwright test pages.spec.ts -g "/war-room"` | ❌ W0 | ⬜ pending |
| SC2 | war-room compose | — | ROOM-04 | — | N/A | e2e ARIA | `cd e2e && npx playwright test war-room.spec.ts -g "aria tablist"` | ❌ W0 | ⬜ pending |
| SC3 | growth wiring | — | ROOM-04 | — | No-team → all tiers 0 (never Err) | e2e data-attr | `cd e2e && npx playwright test war-room.spec.ts -g "growth tiers"` | ❌ W0 | ⬜ pending |
| SC4 | mousemove guard | — | ROOM-04 | — | N/A | e2e round-trip | `cd e2e && npx playwright test war-room.spec.ts -g "mousemove guard"` | ❌ W0 | ⬜ pending |
| SC5-demacia | hydration canary | — | ROOM-04 | — | N/A | e2e canary | `cd e2e && npx playwright test war-room.spec.ts -g "canary demacia"` | ❌ W0 | ⬜ pending |
| SC5-pandemonium | hydration canary | — | ROOM-04 | — | N/A | e2e canary | `cd e2e && npx playwright test war-room.spec.ts -g "canary pandemonium"` | ❌ W0 | ⬜ pending |
| RoomSurface-unit | room_surface refactor | — | ROOM-04 | — | N/A | unit | `cargo test --features ssr --lib room_surface` | ✅ (update dummy-surface test) | ⬜ pending |
| growth-unit | growth wiring | — | ROOM-04 | — | N/A | unit | `cargo test --features ssr --lib growth` | ✅ growth.rs:228-282 | ⬜ pending |
| flat-routes-regression | surface extraction | — | ROOM-04 (D-02) | — | Behavior preserved | e2e regression | `cd e2e && npx playwright test pages.spec.ts` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Observable Validation Signals (ground truth per success criterion)

- **SC1 (no atmosphere flash):** `#room-atmosphere-root` div stays in DOM on tab switch — assert count still 1 after click, `--px` CSS custom property still readable. Keep-mounted panels (D-06) guarantee zero mount churn.
- **SC2 (ARIA tablist):** `role="tablist"` present; each tab button has `role="tab"` + `aria-selected="true|false"`; `aria-controls` points to a valid panel `id`; inactive panels carry `hidden` + retain `role="tabpanel"` (D-07).
- **SC3 (growth overlay tiers):** Seed a team with ≥ thresholds for drafts/opponents, navigate to `/war-room`, assert `data-tactics-tier` / `data-intel-tier` / `data-command-tier` on `#room-atmosphere-root` ≥ 1; fresh/no-team user → all three equal `"0"` (D-10/D-11: verified via attributes, not visible text).
- **SC4 (mousemove guard):** Navigate `/draft` → `/war-room` → `/journey` → `/war-room`; assert exactly one `mousemove` listener (`getEventListeners(window).mousemove.length === 1`, or a `#[cfg(debug_assertions)]` `window.__mousemove_count` counter in `parallax.rs` if `getEventListeners` is unavailable in Playwright).
- **SC5 (hydration-no-panic):** Register a `pageerror` listener before navigating; after `networkidle` + a tab click, assert zero `pageerror` events and zero console errors containing `"panicked"` or `"html/mod.rs"`. Run once per theme (demacia default, pandemonium via `localStorage.setItem('theme','pandemonium')` before navigation).

---

## Wave 0 Requirements

- [ ] `e2e/tests/war-room.spec.ts` — new spec covering SC2 (ARIA), SC3 (growth tiers), SC4 (mousemove guard), SC5 (canary demacia + pandemonium)
- [ ] Add `/war-room` to `AUTHED_PAGES` in `e2e/tests/pages.spec.ts` — covers SC1 smoke
- [ ] Tier-≥1 seed fixture (drafts + scouted opponents in DB before the SC3 test) — seed via server fn call or dedicated seed hook in `e2e/tests/fixtures.ts`
- [ ] Update `room_surface.rs` dummy-surface render/unit test to the new keep-mounted panel contract (the `resolve_active_tab` pure-fn tests survive unchanged)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Subjective "atmosphere grows" feel with placeholder gradients | ROOM-04 (SC3) | Placeholder art makes the visual delta subtle; the objective proof is the `data-*-tier` attributes (automated above). | Optional eyeball check: seed Tier-2 team vs fresh team, compare `/war-room` overlay opacity side by side. |

*All blocking phase behaviors have automated verification via the signals above.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`war-room.spec.ts`, `pages.spec.ts` entry, seed fixture, dummy-surface test)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
