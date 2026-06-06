---
phase: 28-library-ia-healing
plan: "06"
subsystem: rooms
tags: [leptos, rust, routing, ia-healing, redirects, library]

# Dependency graph
requires:
  - phase: 28-05
    provides: LibraryRoom + /library route + tab IDs stats/match/analytics/post-game/personal
  - phase: 27
    provides: WarRoomRoom + OpponentsSurface at /war-room?tab=opponents
provides:
  - six IA-redirect server fns in library.rs (redirect_to_library_{stats,analytics,post_game,personal,match} + redirect_to_war_room_opponents)
  - six redirect shell components in library.rs (StatsRedirect, AnalyticsRedirect, PostGameRedirect, PersonalLearningsRedirect, MatchRedirect, OpponentsRedirect)
  - six flat routes rebound in app.rs to redirect shells
  - /match/:id id preserved into /library?tab=match&id={id}
  - /personal-learnings/new, /draft, /tree-drafter, /game-plan unchanged
affects:
  - 28-07 (e2e suite — expects 302 redirects on these six routes, not standalone renders)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dual redirect pattern: leptos_axum::redirect() in #[server] fn (SSR 302) + #[cfg(feature = \"hydrate\")] window.location().set_href() (client path) — analog roster.rs/profile.rs"
    - "MatchRedirect: use_params_map().read_untracked().get(\"id\").unwrap_or_default() — no bare unwrap, id carried via server fn arg into format!(\"/library?tab=match&id={id}\")"
    - "Unused page imports (StatsPage etc.) removed from app.rs on rebind — warning-clean build"

key-files:
  modified:
    - src/pages/rooms/library.rs
    - src/app.rs

key-decisions:
  - "Dual redirect pattern (SSR leptos_axum::redirect + hydrate set_href) used for all six shells — same contract as roster.rs/profile.rs"
  - "MatchRedirect reads params with read_untracked() at component creation; dispatches RedirectToLibraryMatch(id) server fn so SSR 302 carries the preserved id"
  - "Dead page imports (StatsPage, AnalyticsPage, PostGamePage, OpponentsPage, MatchDetailPage, PersonalLearningsPage) removed from app.rs — build stays warning-clean"
  - "/personal-learnings/new route order preserved after /personal-learnings redirect shell — exact-match routing still resolves NewLearningPage (Pitfall 3)"

# Metrics
duration: 10min
completed: 2026-06-06T02:05:00Z
---

# Phase 28 Plan 06: IA Route Redirects Summary

**Six legacy flat routes converted to 302 redirects into their intent rooms using the dual SSR-redirect + cfg(hydrate)-set_href pattern, with /match/:id id preserved and all excluded routes untouched**

## Performance

- **Duration:** ~10 min
- **Tasks completed:** 2 of 2
- **Files modified:** 2

## Accomplishments

### Task 1 — Six redirect server fns + shell components added to library.rs

- Five static-target server fns: `redirect_to_library_stats` → `/library?tab=stats`, `redirect_to_library_analytics` → `/library?tab=analytics`, `redirect_to_library_post_game` → `/library?tab=post-game`, `redirect_to_library_personal` → `/library?tab=personal`, `redirect_to_war_room_opponents` → `/war-room?tab=opponents`
- One parameterized server fn: `redirect_to_library_match(id: String)` builds `/library?tab=match&id={id}` — id carried from the component arg so the SSR 302 target preserves it (D-11)
- Six shell components using the dual pattern (SSR `leptos_axum::redirect()` via server action + `#[cfg(feature = "hydrate")]` `window.location().set_href()`)
- `MatchRedirect` reads `use_params_map().read_untracked().get("id").unwrap_or_default()` at component top — no bare `.unwrap()` (wasm-patterns rule #35), no reactive closure (rule #58)
- All components co-located in `library.rs` — no separate redirects.rs file

### Task 2 — Six flat routes rebound in app.rs

- Import block updated: `rooms::library::{AnalyticsRedirect, LibraryRoom, MatchRedirect, OpponentsRedirect, PersonalLearningsRedirect, PostGameRedirect, StatsRedirect}`
- Six `view=` swaps: StatsPage→StatsRedirect, AnalyticsPage→AnalyticsRedirect, PostGamePage→PostGameRedirect, OpponentsPage→OpponentsRedirect, MatchDetailPage→MatchRedirect, PersonalLearningsPage→PersonalLearningsRedirect
- No `path!(...)` string changed — only `view=` swapped (routes preserved)
- Dead page imports removed: StatsPage, AnalyticsPage, PostGamePage, OpponentsPage, MatchDetailPage, PersonalLearningsPage — build stays warning-clean
- UNCHANGED: `/personal-learnings/new` (NewLearningPage), `/draft` (DraftPage), `/tree-drafter` (TreeDrafterPage), `/game-plan` (GamePlanPage), `/library` (LibraryRoom), `/war-room` (WarRoomRoom)
- Route order preserved: `/personal-learnings` (redirect shell) declared before `/personal-learnings/new` (NewLearningPage) — exact-match still resolves new-learning form (Pitfall 3)

## Task Commits

1. **Task 1: Six redirect server fns + shell components** — `57625e7` (feat)
2. **Task 2: Rebind six flat routes in app.rs** — `f5d96b1` (feat)

## Files Created/Modified

- `src/pages/rooms/library.rs` — added 158 lines: 6 server fns + 6 redirect shell components (modified)
- `src/app.rs` — updated import block + 6 view= swaps + removed 6 dead imports (modified)

## Deviations from Plan

None — plan executed exactly as written. Both tasks completed without deviation.

## Known Stubs

None — this plan wires live 302 redirect behavior. No placeholder data flows involved.

## Threat Flags

None — all six redirect targets are same-origin hard-coded paths. The MatchRedirect `:id` interpolation appends to a fixed `/library?tab=match&id=` prefix (no open-redirect possible — T-28-06-01 accepted). No new network endpoints, auth paths, file access patterns, or schema changes beyond the plan's threat model.

## Self-Check: PASSED

- `src/pages/rooms/library.rs` — FOUND
- `src/app.rs` (six redirects) — FOUND
- Commit `57625e7` (redirect server fns + shells) — FOUND
- Commit `f5d96b1` (app.rs rebind) — FOUND
- `grep -q 'view=StatsRedirect' src/app.rs` — PASSED
- `grep -q 'view=NewLearningPage' src/app.rs` — PASSED (/personal-learnings/new untouched)
- `grep -q 'view=DraftPage' src/app.rs` — PASSED (D-10 excluded)
- Both SSR and hydrate compile targets exit 0 with pre-existing warnings only
