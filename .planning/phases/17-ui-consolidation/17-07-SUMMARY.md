---
phase: 17-ui-consolidation
plan: 07
subsystem: ui

tags: [visual-regression, audit, gsd-ui-review, guardrails, g-01, g-12, raw-hex, focus-rings, playwright, six-pillar]

# Dependency graph
requires:
  - phase: 17-ui-consolidation/01
    provides: visual-regression.spec.ts scaffold + CI style_guardrails job
  - phase: 17-ui-consolidation/02
    provides: Open-Design lol-companion design system as the audit baseline
  - phase: 17-ui-consolidation/03a-d
    provides: All Strategy hub surfaces restyled (audit input)
  - phase: 17-ui-consolidation/04
    provides: All History hub surfaces restyled (audit input)
  - phase: 17-ui-consolidation/05
    provides: All Profile hub surfaces restyled (audit input)
  - phase: 17-ui-consolidation/06
    provides: Closed-beta + auth + admin + legal + bug-widget surfaces (audit input)
provides:
  - "22 pixel-baseline snapshots in e2e/tests/visual-regression.spec.ts-snapshots/ (5 public + 17 authed routes)"
  - "Strict-scope G-12 sweep clean: zero unpaired `outline:none` / `outline-none` hits in src/"
  - "Raw-hex sweep clean: zero 6-char colour hex codes in src/components/ + src/pages/"
  - "17-UI-REVIEW.md 6-pillar audit report — overall PASS-with-deferred verdict"
  - "Fixture compat update: authedPage now waits for /solo (D-03) instead of /team/dashboard"
affects: [18-region-variants, 18-bug-report-system, 22-pre-launch-a11y-review]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Playwright visual-regression suite with maxDiffPixelRatio: 0.02 tolerance per route"
    - "6-pillar audit format: PASS-with-deferred verdict + per-finding `status: fixed|open|deferred` + per-pillar evidence section that is grep-reproducible"
    - "Verification regex hygiene: avoid co-mentioning `status: open` and `HIGH` in prose adjacent to structured table rows or the verify command will over-match the narrative"
    - "Auth-fixture brittleness: when post-auth landing route changes (D-03: register_action returns /solo), every downstream e2e suite that uses `waitForURL(\"**/team/dashboard\")` silently times out — surface as a clear Rule-3 deviation rather than a vague 'auth failed'"

key-files:
  created:
    - .planning/phases/17-ui-consolidation/17-UI-REVIEW.md
    - e2e/tests/visual-regression.spec.ts-snapshots/  (22 PNG baselines)
  modified:
    - e2e/tests/visual-regression.spec.ts             (scaffold → 22 toHaveScreenshot tests)
    - e2e/tests/fixtures.ts                           (wait for /solo per D-03; pass ?invite=E2E-TEST)
    - e2e/tests/auth.spec.ts                          (?invite=E2E-TEST for direct register tests)
    - src/pages/champion_pool.rs                      (4 G-12 ring additions)
    - src/pages/draft.rs                              (4 G-12 ring additions)
    - src/components/champion_autocomplete.rs         (dropdown-item focus ring per A11Y-02)
    - .planning/phases/17-ui-consolidation/deferred-items.md (Phase 17-07 sweep findings appended)

key-decisions:
  - "Authentication-fixture rewrite (Rule 3 blocking deviation): authedPage was waiting for /team/dashboard; D-03 made register_action redirect to /solo. All 17 authed visual-regression tests + every authenticated downstream spec depended on this fixture. Fixed once, propagates everywhere."
  - "G-01 hits in .claude/skills/skill-creator/ (Poppins + Lora from googleapis.com) intentionally NOT fixed — they are out-of-band tooling assets that never reach a user browser. Deferred with a written rationale in deferred-items.md."
  - "champion_autocomplete dropdown items got a focus-visible ring even though the existing focus-visible:bg-elevated technically passes G-12 (background-change focus indicator) — consistency with the rest of the app was prioritised over strict guardrail-pass minimum."
  - "STATE.md and ROADMAP.md updates were intentionally NOT performed in the worktree (per pause-note `.continue-here.md`: orchestrator-owned files where main always wins on merge). Phase-17 metadata commit happens in main after merge."

patterns-established:
  - "Visual-regression suite with `--update-snapshots` capture + plain re-run verification pattern — every future restyle phase can adopt this directly. Baselines live alongside the spec, get diffed on every CI run, and provide a one-shot pixel-diff PR review."
  - "6-pillar audit report shape (per pillar: criteria, evidence, findings table with severity/disposition/status, disposition narrative) — generalises to any future phase that has UI/UX as the deliverable."
  - "Strict-scope G-12 grep: `grep -rnE 'outline-none' --include='*.rs' src/ | grep -v 'focus-visible:ring'` — single command that reduces 282 false-positive matches to the 0 actually-unpaired cases."

requirements-completed: [SC-4-ui-review-pass, G-01, G-12]

# Metrics
duration: ~25min (executor wall time; excludes prior WIP carried over from paused agent run)
completed: 2026-05-11
---

# Phase 17 Plan 07: Final Audit + Visual Regression Baselines Summary

**Captured 22 visual-regression pixel baselines across every Phase-17 restyled route, ran the final G-01/G-12/raw-hex guardrail sweep clean, fixed one A11Y consistency gap in the autocomplete dropdown, and authored the 6-pillar audit report — overall verdict PASS-with-deferred with no HIGH/CRITICAL findings open.**

## Performance

- **Duration:** ~25 min wall clock for this executor session (prior paused-agent WIP for the spec scaffold + 8 G-12 input fixes was carried into Task 1/Task 2 commits)
- **Build cost:** `cargo leptos watch` cold start ≈ 4 min; SSR check + WASM check + lib test all sub-minute on warm caches
- **Tasks:** 4 of 5 (Task 5 is the user-approval checkpoint, intentionally left for the next session per project convention)
- **Files touched:** 7 source + 23 snapshot binaries + 1 audit report + 1 deferred-items append

## Accomplishments

- **22 pixel baselines committed.** Every Phase-17 restyled route now has a `toHaveScreenshot` test with a committed baseline PNG. The suite is split into a `public routes` describe block (5 tests: `/closed-beta`, `/auth/login`, `/auth/register?invite=…`, `/legal/impressum`, `/legal/datenschutz`) and an `auth-required routes` describe block (17 tests covering every protected page including `/match/:id` with a synthetic ID for the empty-state surface). Baselines re-run cleanly without `--update-snapshots` (verified 50.9 s wall clock).
- **Authentication fixture realigned with D-03.** `e2e/tests/fixtures.ts` previously waited for `/team/dashboard` after registration; the D-03 change to `register_action` (plan 17-06) now returns `/solo`, which silently broke every authenticated spec. Updated `authenticatePage` to wait for `/solo`, propagated the same `?invite=E2E-TEST` query-param threading to `auth.spec.ts` so the closed-beta gate stops rejecting test users.
- **Strict G-12 sweep clean.** `grep -rnE 'outline-none' src/ | grep -v 'focus-visible:ring'` returns 0. The pre-existing 8 G-12 hits in `champion_pool.rs` + `draft.rs` (form inputs missing focus rings) plus the one autocomplete dropdown item gap were resolved. Strict raw-hex sweep also returns 0 colour-shaped hits.
- **6-pillar audit authored.** `17-UI-REVIEW.md` documents PASS verdicts on all 6 pillars with grep-reproducible evidence per pillar. The four deferred items (one MEDIUM → Phase 18 region variants, one LOW → Phase 22 pre-launch a11y, two INFO → non-product surface) each carry an explicit disposition.
- **Build gates green.** `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown` + `cargo test --features ssr --lib` (111 passed, 0 failed, 5 ignored) all clean. The visual-regression suite re-runs against committed baselines cleanly.

## Task Commits

Each task was committed atomically:

1. **Task 1: Visual-regression baselines** — `b029b06` (test). 22 baselines captured (5 public + 17 authed), fixture updates for D-03/closed-beta gate, auth.spec compat tweak.
2. **Task 2: G-12 / G-01 / raw-hex sweep + fixes** — `7ec40de` (fix). 8 ring additions across champion_pool + draft + 1 autocomplete dropdown fix, deferred-items append for the out-of-scope skill-creator G-01 hits.
3. **Task 3: 6-pillar audit report** — `50b0a64` (docs). Full `17-UI-REVIEW.md` with PASS verdicts per pillar.
4. **Task 4: Final test pass + STATE/ROADMAP updates** — Build gates run inline in this session (all green). STATE.md and ROADMAP.md edits intentionally deferred to the post-merge main session per project convention (orchestrator-owned files where main wins on merge).
5. **Task 5: User approval (checkpoint:human-verify)** — left to next session as designed by plan.

**Plan metadata:** _to be added in the next commit (this SUMMARY.md)_

## Files Created/Modified

### Created

- `.planning/phases/17-ui-consolidation/17-UI-REVIEW.md` — 6-pillar audit report with frontmatter verdict-summary, per-pillar criteria/evidence/findings, deferred-items roll-up, and a methodology-notes appendix.
- `e2e/tests/visual-regression.spec.ts-snapshots/authed-action-items-chromium-linux.png` — and 16 other authed-route baselines.
- `e2e/tests/visual-regression.spec.ts-snapshots/public-*-chromium-linux.png` — 5 public-route baselines (4 of these were captured by the prior paused-agent run; `public-auth-login` was re-captured with the updated invite-query baseline).

### Modified

- `e2e/tests/visual-regression.spec.ts` — Spec scaffold replaced with full 22-test suite. Uses `maxDiffPixelRatio: 0.02` (allow ~2% pixel drift for anti-aliasing variance) and `fullPage: true` per route.
- `e2e/tests/fixtures.ts` — `authenticatePage` now passes `?invite=E2E-TEST` and waits for `**/solo` (not `**/team/dashboard`). Documented inline why D-03 + D-16 demand both changes together.
- `e2e/tests/auth.spec.ts` — Direct register-test path now uses `?invite=E2E-TEST` so the test never lands on `/closed-beta`.
- `src/pages/champion_pool.rs` — 4 form inputs (notes textarea, note-type select, title input, content textarea) gained `focus-visible:ring-2 focus-visible:ring-accent/50`.
- `src/pages/draft.rs` — 4 form inputs gained the same ring pattern.
- `src/components/champion_autocomplete.rs` — Dropdown buttons gained `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset` so keyboard focus matches the rest of the app (the existing `focus-visible:bg-elevated` was technically AA-compliant but inconsistent).
- `.planning/phases/17-ui-consolidation/deferred-items.md` — Appended a "Plan 17-07 sweep findings" section with the rationale for deferring the skill-creator G-01 hits and a note on the autocomplete fix.

## Decisions Made

- **D-03 propagation through e2e fixtures (deviation Rule 3 — blocking).** When plan 17-06 changed `register_action` to redirect new users to `/solo` (the D-03 default-to-solo decision), the `authenticatePage` helper in `fixtures.ts` was left waiting for `/team/dashboard`. Every authenticated visual-regression test failed with a 20-second `waitForURL` timeout. Fix was one line + one comment update; the lesson is fixture brittleness: a single redirect-target change at the server-fn layer can silently sink every downstream auth spec.
- **Out-of-scope G-01 deferral.** The two `fonts.googleapis.com` hits in `.claude/skills/skill-creator/eval_review.html` and `.claude/skills/skill-creator/eval-viewer/viewer.html` are tooling assets, never served by the lol-companion app, never reach a user browser. SCOPE BOUNDARY says don't fix pre-existing issues unrelated to the current task. Documented as PERF-01 in 17-UI-REVIEW.md (INFO severity, deferred-forever unless skill-creator is repackaged).
- **Autocomplete dropdown focus-ring upgrade (Rule 1 — bug).** The dropdown items had `focus-visible:bg-elevated` + `outline-none` and a comment that read "focus-visible ring for keyboard parity (G-12)" — the comment described an intent the code didn't honour. Added the ring so code matches comment and consistency matches the rest of the app.
- **STATE.md / ROADMAP.md left untouched in worktree.** Per `.continue-here.md` (orchestrator notes), those files are owned by the main branch and any worktree edit will lose to main on merge. Phase-17 metadata commit will happen in the main session post-merge.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] Auth fixture waited for stale post-auth route**

- **Found during:** Task 1 (visual-regression baseline capture, first run)
- **Issue:** `authenticatePage` in `e2e/tests/fixtures.ts` called `waitForURL("**/team/dashboard", { timeout: 20000 })` after submitting the register form. Plan 17-06 introduced D-03 ("new users default to solo mode") which changed `register_action` to `Ok("/solo".to_string())`. The fixture timed out after 20 s on every authed test; all 17 authed visual-regression tests failed.
- **Fix:** Changed `waitForURL("**/solo", { timeout: 20000 })` + updated the inline comment to document the D-03 / D-16 dependency. The `createTeam` helper already handled the navigation correctly (`page.goto("/team/roster")`), so no change there.
- **Files modified:** `e2e/tests/fixtures.ts`
- **Verification:** Re-ran `npx playwright test visual-regression.spec.ts --update-snapshots` — all 22 baselines captured cleanly in 54.4 s.
- **Committed in:** `b029b06` (Task 1)

---

**2. [Rule 1 - Bug] Autocomplete dropdown items had comment-vs-code drift on focus indicator**

- **Found during:** Task 2 (strict G-12 sweep)
- **Issue:** `src/components/champion_autocomplete.rs` line 114 had `focus-visible:bg-elevated focus-visible:text-primary focus-visible:outline-none` with a comment claiming "focus-visible ring for keyboard parity (G-12)". The code did not include a ring; the focus indicator was background-only.
- **Fix:** Added `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset` so the comment is true and keyboard focus matches the rest of the app.
- **Files modified:** `src/components/champion_autocomplete.rs`
- **Verification:** `cargo check --features ssr` clean; strict G-12 grep returns 0 unpaired hits.
- **Committed in:** `7ec40de` (Task 2)

---

**Total deviations:** 2 auto-fixed (1 Rule-3 blocker, 1 Rule-1 bug).
**Impact on plan:** None on scope; the fixture fix is the load-bearing change that unblocked every authed baseline.

## Issues Encountered

- **Verify-regex over-match on prose narrative.** The audit file's automated verify (`grep -qE "status:\s*open.*HIGH|status:\s*open.*CRITICAL"`) over-matched a single line of prose where I wrote `(status: open × severity: HIGH|CRITICAL = empty set)` to describe the phase-completion gate. Rephrased the sentence so the disposition is described in plain English without echoing the regex tokens; verify now correctly exits non-zero. Lesson logged in `tech-stack.patterns`.
- **First-pass dev-server cold start was 4 minutes.** `cargo leptos watch` ran a clean rebuild of both SSR + WASM targets after the worktree had been idle on the previous pause — not really an issue, just expected wall-clock cost.

## Threat Flags

None — the audit surface introduces no new trust boundaries beyond what the plan's `<threat_model>` enumerated. T-17-33 (cross-platform baseline drift) is mitigated by the `maxDiffPixelRatio: 0.02` tolerance; T-17-34 (audit findings not acted on) is mitigated by the `status: fixed|open|deferred` per-finding tracking; T-17-35 (sweep bypass via inline comments) remains `accept` per plan disposition.

## Known Stubs

None new from this plan. The existing stubs documented in 17-06 (admin invites placeholder rows, register_action invite_code unused parameter, bug-report widget console.log Submit) remain as documented; this plan did not change them.

## Self-Check: PASSED

- **Files claimed created exist:**
  - `[ -f .planning/phases/17-ui-consolidation/17-UI-REVIEW.md ]` → FOUND
  - `[ -d e2e/tests/visual-regression.spec.ts-snapshots ]` → FOUND with 22 .png files
- **Commits claimed exist:**
  - `b029b06` → FOUND (`test(17-07): visual-regression baselines for restyled routes (Demacia)`)
  - `7ec40de` → FOUND (`fix(17-07): G-12 focus rings on champion-pool / draft / autocomplete`)
  - `50b0a64` → FOUND (`docs(17-07): 6-pillar UI audit report`)
- **Build gates green:**
  - `cargo check --features ssr` → Finished in 0.15s
  - `cargo check --features hydrate --target wasm32-unknown-unknown` → Finished in 30.78s (1 pre-existing dead-code warning, out of scope)
  - `cargo test --features ssr --lib` → 111 passed; 0 failed; 5 ignored
  - `npx playwright test visual-regression.spec.ts` → 22 passed (50.9s)
