---
phase: 01-pipeline-ctas
plan: 03
subsystem: ui
tags: [leptos, leptos-router, url-params, navigation, cta-buttons]

# Dependency graph
requires:
  - phase: 01-pipeline-ctas/01-01
    provides: check_draft_has_game_plan server fn, get_game_plans_for_draft DB query
  - phase: 01-pipeline-ctas/01-02
    provides: game plan ?draft_id prefill established, back-reference badge pattern

provides:
  - "Prep for This Draft" CTA on draft list rows with duplicate-check guard
  - Duplicate prompt inline UI (View Game Plan / Create New / Cancel)
  - Game plan count badge per draft row (e.g. "1 game plan")
  - "Prep for This Draft" CTA on loaded draft in editor action area
  - "Review This Game" CTA on loaded draft when linked game plans exist
  - ?draft_id=X URL param auto-loads specific draft in editor
  - Post-game page reads ?review_id=, ?plan_id=, ?draft_id= from URL
  - Post-game back-reference badges: Game Plan (→/game-plan) and Draft (→/draft?draft_id=X)
affects:
  - phase: 01-pipeline-ctas (completes the navigation endpoint task)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "URL param auto-load: use_query_map + Effect + guard signal prevents re-triggering"
    - "Batch game plan counts via list_game_plans computed client-side into StoredValue-friendly map"
    - "Duplicate CTA guard: check_draft_has_game_plan spawned async, result drives inline prompt or navigate"

key-files:
  created: []
  modified:
    - src/pages/draft.rs
    - src/pages/post_game.rs

key-decisions:
  - "Review This Game from draft navigates to /post-game?draft_id=X rather than calling start_post_game_review — avoids empty plan_id in the review record"
  - "Game plan counts loaded via get_draft_game_plan_counts (new server fn) that batch-queries all team plans and groups by draft_id"
  - "Back-reference badges in post-game editor use native <a> (not <A>): no attr:class needed, href is reactive for draft"

patterns-established:
  - "URL param guard: (url_X_loaded: RwSignal<bool>) prevents Effect from re-seeding signals after user edits"
  - "CTA loading guard: cta_loading signal disables button during async check to prevent double-click"

requirements-completed:
  - PIPE-01

# Metrics
duration: 30min
completed: 2026-03-14
---

# Phase 1 Plan 3: Draft CTAs, URL Params, and Post-Game Back-References Summary

**Prep for This Draft CTA with duplicate guard, draft URL deep-linking (?draft_id=X), and post-game back-reference badges with draft deep-link (?draft_id=X)**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-03-14
- **Completed:** 2026-03-14
- **Tasks:** 2/2 auto tasks complete (checkpoint pending human verify)
- **Files modified:** 2

## Accomplishments
- Added "Prep for This Draft" button on every draft list row and in the loaded draft editor header
- Duplicate-check guard: calling check_draft_has_game_plan before navigating, showing inline prompt with View Game Plan / Create New / Cancel
- Game plan count badges ("1 game plan", "2 game plans") on draft list rows with clickable link to /game-plan
- "Review This Game" button on loaded draft editor when draft has linked game plans (navigates /post-game?draft_id=X)
- /draft?draft_id=X URL param auto-loads the correct draft in editor on page load
- Post-game page reads ?review_id=, ?plan_id=, ?draft_id= from URL; auto-loads review when review_id present
- Back-reference badges in post-game editor: Game Plan badge links to /game-plan, Draft badge deep-links to /draft?draft_id={id}

## Task Commits

Each task was committed atomically:

1. **Task 1: Draft CTAs, game plan badges, URL param auto-load** - `19b8643` (feat)
2. **Task 2: Post-game URL params and back-reference badges** - `ebd4bb7` (feat)

**Plan metadata:** (pending final commit)

## Files Created/Modified
- `src/pages/draft.rs` - get_draft_game_plan_counts server fn, check_draft_has_game_plan import, URL param Effect, CTA buttons, duplicate prompt, game plan badges, Review This Game button
- `src/pages/post_game.rs` - URL param query map reading, auto-load review Effect, URL plan_id/draft_id seeding, back-reference badges

## Decisions Made
- "Review This Game" from draft navigates with ?draft_id= query param rather than calling start_post_game_review with empty plan_id — cleaner UX, avoids empty plan_id stored in review
- Game plan counts fetched via new get_draft_game_plan_counts server fn (batch fetch, grouped client-side) for efficiency
- Native `<a>` elements (not Leptos `<A>`) for back-reference badges to avoid attr:class requirement

## Deviations from Plan

None — plan executed as specified. The "Review This Game" button uses a simpler navigation approach (/post-game?draft_id=X) rather than start_post_game_review with empty plan_id to avoid storing a malformed review record. This is a minor implementation detail within plan scope.

## Issues Encountered
- `attr:class` on native `<a>` elements caused compile error (only needed for Leptos `<A>` component) — fixed to `class=`

## Next Phase Readiness
- Full draft -> game plan -> post-game navigation pipeline implemented with deep-linking
- Back-reference badges complete across all three pages (game plan page done in Plan 02)
- Phase 1 pipeline CTA goals met — awaiting human verification of end-to-end flow

---
*Phase: 01-pipeline-ctas*
*Completed: 2026-03-14*
