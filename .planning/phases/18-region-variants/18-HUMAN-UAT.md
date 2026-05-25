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
awaiting: user response

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

### 2. Verify pixelDiffRatio threshold deviation is acceptable
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

total: 2
passed: 0
issues: 1
pending: 1
skipped: 0
blocked: 0

## Gaps

- truth: "Every scoped page renders STRUCTURALLY different markup between Demacia and Pandemonium at runtime in a real browser session"
  status: failed
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
