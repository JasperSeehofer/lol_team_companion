---
phase: 29
slug: throne-quicknav-room-a11y
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: `29-RESEARCH.md` § Validation Architecture (SC1–SC5 → Test Map).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (e2e) + Rust `#[cfg(test)]` unit tests |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| **Full suite command** | `cd e2e && npx playwright test` |
| **Unit tests** | `cargo test --features ssr --lib` (integration tests OOM under BFD — `--lib` only, rule #39) |
| **Estimated runtime** | unit ~30s · targeted e2e ~60s · full e2e ~3–5 min |

---

## Sampling Rate

- **After every task commit:** `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`; `cargo test --features ssr --lib` for any `growth.rs` edit.
- **After every plan wave:** `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts` plus the wave's new specs.
- **Before `/gsd:verify-work`:** Full e2e suite must be green in **both** regions.
- **Max feedback latency:** ~60 seconds (targeted spec).

---

## Per-Task Verification Map

| SC | Plan | Wave | Requirement | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|----|------|------|-------------|-----------------|-----------|-------------------|-------------|--------|
| SC1 (env reflects growth) | growth/room | — | ROOM-06 | `get_throne_growth_counts` uses pre-authorized `team_id` (V4) | e2e seed+opacity | `cd e2e && npx playwright test throne-growth.spec.ts` | ❌ W0 (new spec) | ⬜ pending |
| SC1b (blended no-team) | growth | — | ROOM-06 | `progression` reads only `auth.user.id` ranked (V4) | unit + e2e | `cargo test --features ssr --lib -- throne` | ❌ W0 (`growth.rs` tests) | ⬜ pending |
| SC2 (focus-restore) | quicknav | — | ROOM-08 | client-only filter, no server round-trip (V5) | e2e a11y | `cd e2e && npx playwright test quicknav-focus.spec.ts -g "restore"` | ❌ W0 (new spec) | ⬜ pending |
| SC3 (focus-trap) | quicknav | — | ROOM-08 / A11Y-02 | N/A | e2e a11y | `cd e2e && npx playwright test quicknav-focus.spec.ts -g "trap"` | ❌ W0 (new spec) | ⬜ pending |
| SC4 (hydrate-no-panic ×2 regions) | room/a11y | — | A11Y-02 | region server-set from cookie, never URL param (Tampering) | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/throne"` | ❌ W0 (add `/throne` to `SCOPED_ROUTES`) | ⬜ pending |
| SC5 (WCAG AA contrast) | a11y | — | A11Y-02 | N/A | e2e contrast | `cd e2e && npx playwright test room-contrast.spec.ts` | ❌ W0 (new spec) | ⬜ pending |
| Redirects (302, no 404) | routing | — | ROOM-06 | static literal redirect targets, no open redirect (Tampering) | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ W0 (update `AUTHED_PAGES`) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `input.css` — add the 9 tier→opacity selectors for `roster`/`progression`/`crown` (Pitfall 1 — **BLOCKER for SC1**: `.room-overlay` defaults to `opacity:0`, so growth is invisible and the data-tier assertion passes anyway — the Phase 27 growth-visibility gap).
- [ ] `public/img/rooms/throne/{demacia,pandemonium}/` — placeholder gradient WebPs (plane-sky/mid/near + overlay-roster/progression/crown); `loading="eager"` 404s without them.
- [ ] `e2e/tests/hydration-no-panic.spec.ts` — add `/throne` to `SCOPED_ROUTES` (`:72-89`); both regions auto-iterate.
- [ ] `e2e/tests/pages.spec.ts` — add `/throne` to `AUTHED_PAGES`; update `/profile`, `/team/dashboard` entries (now redirect).
- [ ] `src/models/growth.rs` — `ThroneGrowthCounts`/`ThroneGrowthState` (`from_parts`) + 5 unit tests (`throne_empty_all_seed`, `throne_roster_tracks_members`, `throne_crown_tracks_wins`, `throne_progression_grows_without_team`, `throne_state_round_trips_json`).
- [ ] New e2e specs: `throne-growth.spec.ts`, `quicknav-focus.spec.ts`, `room-contrast.spec.ts`.
- [ ] Resolve open questions A2 (`post_game_learning.win_loss` literal) + A3 (`team_member.team` field name) by grep before the DB query task.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Subjective parallax "feel"/atmosphere quality | ROOM-06 | Aesthetic judgement; placeholder gradients this phase | Visual spot-check `/throne` in both regions; defer real-art review to Phase 30 |
| Screen-reader announcement of tab/overlay state | A11Y-02 | NVDA/VoiceOver behavior not in automated harness | Optional manual AT pass; automated coverage asserts ARIA roles + focus order |

*Automated coverage exists for all five Success Criteria; the above are quality spot-checks, not SC gates.*

---

## Validation Sign-Off

- [ ] All SCs have an `<automated>` verify or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (CSS selectors, placeholder art, new specs, growth model)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter (after planner wires Wave 0)

**Approval:** pending
