# Phase 17 — Deferred Items

Tracked here so they aren't lost between plans. Each item has a target plan
where it should land.

## G-12 pre-existing form-input violations (62 hits)

The `style_guardrails` CI sweep added in plan 17-01 task 12 enforces G-12
(`outline:none` only when paired with `focus-visible:ring`) **scoped to
files modified by Phase 17 plans**. Pre-existing form-input fields across
the codebase use the legacy `focus:outline-none focus:border-accent`
pattern (no ring) and currently fail the strict sweep.

**Files with violations** (62 total `outline-none`/`outline:none` hits):

| File | Lines | Plan to migrate |
|------|-------|-----------------|
| src/pages/post_game.rs | 311, 315 | 02-strategy-hub or dedicated cleanup |
| src/components/champion_autocomplete.rs | 61 | 02-strategy-hub |
| src/pages/personal_learnings.rs | 260, 270, 281, 292, 960, 983, 1006, 1065 | 03-history-hub |
| src/pages/action_items.rs | 293, 324, 350, 365 | 02-strategy-hub |
| src/components/champion_picker.rs | 102 | 02-strategy-hub |
| src/pages/team/dashboard.rs | 743, 751, 1096, 1428, 1531 | 04-profile-hub |
| src/pages/solo_dashboard.rs | 783, 792, 899, 991 | 04-profile-hub |
| src/pages/team_builder.rs | 568, 749, 845 | 04-profile-hub |
| src/pages/opponents.rs | 719, 745, 893, 954, 1029, 1206 | 02-strategy-hub |
| src/pages/auth/register.rs | 74, 83, 93 | 06-closed-beta |
| src/pages/auth/login.rs | 74, 83 | 06-closed-beta |
| src/pages/team/roster.rs | 190, 197, 226, 304 | 04-profile-hub |
| src/pages/tree_drafter.rs | 745, 755, 992, 1246, 1260, 1272 | 02-strategy-hub |
| src/pages/profile.rs | 202, 300 | 04-profile-hub |
| src/pages/game_plan.rs | 510, 514 | 02-strategy-hub |
| src/pages/champion_pool.rs | 974, 1257, 1296, 1336 | 02-strategy-hub |
| src/pages/stats.rs | 334, 606, 626, 651 | 03-history-hub |
| src/pages/draft.rs | 1533, 1549, 1572, 1811, 1821, 1955, 1962, 1967, 2154, 2192, 2244, 3571, 3581 | 02-strategy-hub |

**Migration:** swap `focus:outline-none focus:border-accent` →
`focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
focus:border-accent` (or remove `focus:outline-none` and use the ring
exclusively). Once each hub plan migrates its surface, remove that file
from the scoped CI sweep list and move it to the global sweep.

**Acceptance gate:** at the end of Phase 17, the `style_guardrails` job's
G-12 step should drop the `PHASE_17_FILES` scope and run codebase-wide.

## Plan 17-01 specific

- **`/solo-dashboard` route inconsistency:** Plan 17-01 task 11 references
  `/solo-dashboard` in `AUTHED_PAGES` but the actual route is `/solo`. No
  action needed — `/solo` is already in `AUTHED_PAGES`.
- **SSR-authoritative `data-theme`:** Per task 6 fallback, the leptos
  context closure runs synchronously and cannot await the AuthSession
  extractor. The current implementation defaults to `demacia` and lets
  the `ThemeToggle` component perform a post-hydration sync from the DB.
  Brief flicker on the very first authenticated page load is acceptable
  per the documented FALLBACK in the plan. If SSR-authoritative becomes
  required (FOUC complaint), inject AuthSession via axum middleware into
  request extensions and read from the context closure.
