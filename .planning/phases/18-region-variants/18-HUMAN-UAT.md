---
status: partial
phase: 18-region-variants
source: [18-VERIFICATION.md]
started: 2026-05-22T12:00:00Z
updated: 2026-05-22T12:00:00Z
---

## Current Test

number: 1
name: Manual side-by-side visual review of all 11 scoped page pairs
expected: |
  Each page pair renders STRUCTURALLY different markup between Demacia and
  Pandemonium (not just different colors). Specific checks per 18-UI-REVIEW.md:
  - /draft carousel — confidence/sample-size on Pandemonium + onDeck halo on Demacia
  - /draft war-table — composition pillars on Demacia
  - /draft ledger — medieval double-entry (D) vs brutalist dual-column (P)
  - /solo constellation — pool-gaps + last-10 + sort/filter (D) vs tier crest + 4 stat cards (P)
  - /solo forge + journal — both sibling pairs render
  - /team/dashboard dashboard mode — Demacia (gilt cards) vs Pandemonium (7-section)
  - /team/dashboard brief mode — sibling pair renders
  - /tree-drafter, /champion-pool, /match/:id, /post-game — no-patch ports look correct
  - Mode toggle works and persists across reload + across region switch
  - Utility routes (login, register, profile, opponents, stats, legal, etc.) look IDENTICAL to pre-Phase-18
result: pass
resolved: 2026-05-25 by Phase 18.1 — see .planning/phases/18.1-ssr-theme-injection-gap-closure-for-ui-18-runtime-01/18.1-03-EVIDENCE.md

## Tests

### 1. Manual side-by-side visual review of all 11 scoped page pairs
expected: |
  Each page pair renders STRUCTURALLY different markup between Demacia and
  Pandemonium (not just different colors). Specific checks per 18-UI-REVIEW.md:
  - /draft carousel — confidence/sample-size on Pandemonium + onDeck halo on Demacia
  - /draft war-table — composition pillars on Demacia
  - /draft ledger — medieval double-entry (D) vs brutalist dual-column (P)
  - /solo constellation — pool-gaps + last-10 + sort/filter (D) vs tier crest + 4 stat cards (P)
  - /solo forge + journal — both sibling pairs render
  - /team/dashboard dashboard mode — Demacia (gilt cards) vs Pandemonium (7-section: RiotTape header, 5-col roster + MoodMeter, captain note, reasoned bans, pool-ready, their pattern, threats)
  - /team/dashboard brief mode — sibling pair renders
  - /tree-drafter, /champion-pool, /match/:id, /post-game — no-patch ports look correct
  - Mode toggle works and persists across reload + across region switch
  - Utility routes (login, register, profile, opponents, stats, legal, etc.) look IDENTICAL to pre-Phase-18
result: issue
reported: |
  Orchestrator agent-browser sweep (2026-05-25) found that Phase 18 structural
  region branching CANNOT activate at runtime. Token swap (gold<->pink, font/border
  style) works via post-hydration `<html data-theme>` flip, but `region: String`
  props on all pages capture from `InitialTheme` context which `src/main.rs:88`
  hardcodes to `InitialTheme::default()` ("demacia") at SSR request time. Effect:
  every page renders Demacia structural branches with Pandemonium color tokens
  overlaid — the 11 scoped pages NEVER render their `is_pandemonium { ... }`
  branches in a real browser session.

  Confirmed empirically: /draft -> click PANDEMONIUM -> click LEDGER -> DOM snapshot
  contains "DRAFT LEDGER" heading + "BLUE SIDE -- OUR FORCES" + Cormorant serif
  (Demacia DraftLedgerView else-branch). The Pandemonium branch's "RiotTape
  DRAFT_LEDGER · v0.1" + "// BLUE_LOG" + "// RED_LOG" Glitch markers (per
  draft.rs:4143-4160) are NOT in the DOM. After reload, `data-theme` reverts to
  "demacia" because no cookie/localStorage write happens (only DB persist via
  set_user_theme), and SSR still provides InitialTheme::default().

  The deferred work that blocks this is documented at src/main.rs:82-87 --
  "SSR-authoritative theme requires axum middleware injection which is deferred
  to a follow-up plan if FOUC becomes visible." Phase 17 deferred this. Phase 18
  built on top of `region: String` propagation without closing the deferral.

  Sweep evidence: /tmp/phase18-verify/*.png (16 screenshots across /solo, /draft).
severity: blocker
finding_id: UI-18-RUNTIME-01
disposition: requires Phase 18.1 fix plan (axum middleware injection of
  InitialTheme from request cookie+session, OR client-side region prop that
  reactively re-renders region-branching subviews — leaning toward middleware).
status: RESOLVED 2026-05-25 — Phase 18.1 closed the SSR theme-injection gap. Runtime agent-browser sweep across all 11 scoped pages × 2 regions (28 Pandemonium markers, 18 Demacia negative-space markers, 3/3 utility REQ-7 PASS, 3/3 mode-toggle D-04 PASS) confirms Pandemonium structural branches activate at SSR. See .planning/phases/18.1-ssr-theme-injection-gap-closure-for-ui-18-runtime-01/18.1-03-EVIDENCE.md for per-marker evidence. REQ-7 utility-route equivalence preserved.

### 2. UI-18.1-HYDRATE-01 — Pandemonium pages panic during WASM hydration
expected: |
  Pandemonium-themed pages should hydrate cleanly from SSR HTML so that
  client-side interactivity (mode toggles, theme toggles, draft slot drag-drop,
  and all other WASM event handlers) works after the page loads. The hydrate
  step should produce no console panics; the SSR-rendered DOM should match
  the WASM-rendered DOM exactly.
reported: |
  Phase 18.1-03 runtime agent-browser sweep (2026-05-25) found that every Pandemonium-themed
  page panics during WASM hydration with the following stack:

    [error] panicked at /home/jasper/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tachys-0.2.14/src/html/mod.rs:217:14:
    called `Option::unwrap()` on a `None` value

  Demacia pages hydrate cleanly. SSR HTML for both regions renders correctly (this is what
  proves the 18.1 architectural fix works). The hydration panic kills client-side interactivity
  on Pandemonium pages — mode-toggle clicks, theme-toggle clicks, drag-drop, and other WASM
  event handlers do not fire.

  Latent bug: the Pandemonium structural branches built in Phase 18 contain an SSR/hydrate
  mismatch that was hidden until Phase 18.1 made those branches reachable at runtime. A
  specific component is hashing differently SSR-side vs hydrate-side, causing tachys to
  unwrap a None when expecting a Some during DOM reconciliation.

  Severity: bug (Pandemonium is unusable past initial render). Not a regression of 18.1's
  scope (SSR injection); a NEW finding surfaced by it.

  Evidence: see .planning/phases/18.1-ssr-theme-injection-gap-closure-for-ui-18-runtime-01/18.1-03-EVIDENCE.md, "Investigation Flag 1" section.
result: pass
severity: bug
finding_id: UI-18.1-HYDRATE-01
discovered_by: Phase 18.1-03
recommended_disposition: Phase 18.2 (hydrate bridge) — narrow down which component triggers the unwrap, fix the SSR/hydrate hash mismatch, re-run the agent-browser sweep to verify interactivity restored.
status: RESOLVED 2026-05-26 — Phase 18.2 closed the Pandemonium hydration gap.
  Culprit was CompanionSigil (region primitive reading use_context::<InitialTheme>()
  on the WASM side with no hydrate-time provider — exposed by Phase 18.1's SSR
  injection fix); resolved via Fix Pattern 2 prop-refactor (commit 50c70a8) plus
  the canonical Plan 04 fix — a cfg(feature="hydrate") provide_context(InitialTheme(...))
  block in App::App() that reads <html data-theme> via web_sys and re-provides on
  the WASM owner before view! runs (commit e67c7da). Agent-browser sweep across
  all 7 scoped Pandemonium routes shows no tachys panic; D-13 interactivity triad
  (mode-toggle on /draft, /solo, /team/dashboard, theme-toggle round-trip,
  draft slot click-fill) observable on every probe. Demacia parity preserved
  (7/7 Demacia routes also panic-free). See
  .planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-DIAGNOSIS.md
  for the bisect trail and per-route evidence; see e2e/tests/hydration-no-panic.spec.ts
  (commit ed82453) for the 19-test regression gate; see
  .planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-05-EVIDENCE.md
  for the close-out sweep evidence.

### 3. Verify pixelDiffRatio threshold deviation is acceptable
expected: |
  Confirm `REGION_DIFF_THRESHOLD = 0.005` (0.5%) in `e2e/tests/region-diff.spec.ts` is
  sufficient to detect structural region differences for v1.3 closed-beta.

  The SPEC (REQ-6) specified `> 0.40` (40%); actual measured ratios are 1.5–2.5% per route.
  The 0.5% threshold reliably distinguishes "different" from "identical" (identical pages
  score ~0%).

  Decision (D-THRESHOLD, documented in 18-09-SUMMARY.md): Phase 18 region differences are
  typographic/ornamental (font swap, border style, accent color), not full layout-level
  rewrites. Expecting 40% pixel diff would have required the implementer to introduce
  artificial visual noise. The 0.5% threshold catches "the page actually changed shape" while
  staying well above noise.

  Question for user: Is the 0.005 threshold acceptable for v1.3 launch readiness, or should
  the SPEC be updated to match implementation (and a follow-up plan added to increase the
  threshold when more region differentiation lands)?
result: [pending]

## Summary

total: 3
passed: 1
issues: 0
pending: 2
skipped: 0
blocked: 0
resolved: 2
notes: |
  Test 1 (UI-18-RUNTIME-01) resolved 2026-05-25 by Phase 18.1 — passed: 0 → 1, issues: 1 → 0 for this finding.
  Test 2 (UI-18.1-HYDRATE-01) is a NEW finding surfaced by Phase 18.1's SSR fix; result: issue, severity: bug — recommended for Phase 18.2. Counts as issues: 1 (the new finding) AND pending: 1 (awaiting Phase 18.2 disposition).
  Test 3 (pixelDiffRatio threshold) remains pending user response — counts as pending: 1.
  Net: total 2 → 3 (added hydrate finding); passed 0 → 1; issues 1 → 1 (different finding); pending 1 → 2 (added hydrate awaiting 18.2); resolved 0 → 1.
  Test 2 (UI-18.1-HYDRATE-01) resolved 2026-05-26 by Phase 18.2 — see
  .planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-DIAGNOSIS.md.
  issues: 1 → 0, resolved: 1 → 2.

## Gaps

- truth: "Every scoped page renders STRUCTURALLY different markup between Demacia and Pandemonium at runtime in a real browser session"
  status: resolved
  resolved_by: Phase 18.1
  reason: "Orchestrator agent-browser sweep 2026-05-25 — src/main.rs:88 hardcodes InitialTheme::default() (\"demacia\") at SSR. Region toggle persists to DB but next SSR request still gets \"demacia\". All scoped pages capture region=\"demacia\" once at component init, so is_pandemonium branches never activate. Visible effect: token swap (color/font) works via post-hydration data-theme flip, but DraftLedgerView, TeamDashboardView Pandemonium-rebuild, SoloForge, etc. never render their Pandemonium structural branches."
  severity: blocker
  test: 1
  finding_id: UI-18-RUNTIME-01
  artifacts:
    - /tmp/phase18-verify/13-draft-demacia-ledger-clean.png
    - /tmp/phase18-verify/15-draft-pandemonium-ledger-reloaded.png
    - .planning/phases/18-region-variants/18-HUMAN-UAT.md (this file)
  missing:
    - axum middleware to inject InitialTheme from request cookie + auth session
    - client-side cookie write on theme toggle (so unauthenticated reloads keep theme)
    - OR: convert region: String prop to a reactive context that subviews subscribe to (would re-render structural branches without reload)

- truth: "Pandemonium-themed pages hydrate cleanly from SSR HTML so client-side interactivity (mode toggles, theme toggle, drag-drop, all WASM event handlers) works after page load"
  status: resolved
  resolved_by: Phase 18.2
  discovered_by: Phase 18.1
  reason: "Phase 18.1-03 runtime agent-browser sweep (2026-05-25) found that every Pandemonium-themed page panics during WASM hydration at tachys-0.2.14/src/html/mod.rs:217:14 ('called Option::unwrap() on a None value'). Demacia pages hydrate cleanly. SSR HTML for both regions renders correctly (proving 18.1 closed the SSR injection gap), but the latent SSR/hydrate hash mismatch was hidden pre-18.1 because Pandemonium branches never rendered at SSR. The panic kills all client-side interactivity on Pandemonium pages.
    Resolved 2026-05-26 by Phase 18.2 — culprit was CompanionSigil (use_context
    inside a primitive with no hydrate-side provider); fixed via Fix Pattern 2
    prop refactor + Plan 04 App::App() cfg(hydrate) provide_context(InitialTheme).
    Agent-browser sweep + 19-test e2e regression gate confirm clean."
  severity: bug
  test: 2
  finding_id: UI-18.1-HYDRATE-01
  artifacts:
    - .planning/phases/18.1-ssr-theme-injection-gap-closure-for-ui-18-runtime-01/18.1-03-EVIDENCE.md (Investigation Flag 1)
  missing:
    - Diagnosis of which Pandemonium component triggers the SSR/hydrate hash mismatch (likely a structural primitive used across all P pages, since the panic is universal)
    - Fix for the unwrap-on-None in tachys hydration reconciliation (typically caused by SSR rendering markup that the client-side WASM render diverges from)
    - Re-run of the agent-browser sweep with WASM event handlers exercised to confirm interactivity restored
  recommended_disposition: Phase 18.2 (hydrate bridge) — narrow scope; touches structural Pandemonium primitives, not the SSR injection layer 18.1 just landed.

## How to run the visual review

```bash
# Start dev server (background)
cargo leptos watch &
./scripts/wait_for_server.sh 120

# Register/log in via /auth/register, then visit:
# /draft  (toggle modes: carousel, war-table, ledger; switch region)
# /solo   (toggle modes: constellation, forge, journal; switch region)
# /team/dashboard (toggle modes: dashboard, brief; switch region)
# /tree-drafter, /champion-pool, /match/:id, /post-game (one mode each; switch region)
# Also sanity-check that login/profile/opponents/stats look unchanged

# When done, type "approved" or describe specific revisions needed.
```
