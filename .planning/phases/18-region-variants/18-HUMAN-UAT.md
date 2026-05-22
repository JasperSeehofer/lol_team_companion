---
status: partial
phase: 18-region-variants
source: [18-VERIFICATION.md]
started: 2026-05-22T12:00:00Z
updated: 2026-05-22T12:00:00Z
---

## Current Test

[awaiting human testing]

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
result: [pending]

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
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps

(None — phase implementation is complete with 7/8 must_haves passing automated verification. The remaining 1 item is human-only judgment, not a code defect.)

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
