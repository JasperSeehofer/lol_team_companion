---
phase: 10-opponents-redesign
plan: "03"
subsystem: ui
tags: [opponent, leptos, tailwind, wasm, e2e, playwright, player-card, otp, mastery, pool-analysis]
dependency_graph:
  requires:
    - phase: 10-01
      provides: [OpponentPlayer.compute_intel, is_stale, create_opponent_with_players, mastery_data_json, role_distribution_json]
    - phase: 10-02
      provides: [create_opponent_with_players_fn, fetch_player_intel_fn, PlayerIntelData]
  provides: [OpponentsPage redesign, 5-role creation form, PlayerCard with OTP/mastery/pool analysis, e2e smoke test]
  affects: [src/pages/opponents.rs, e2e/tests/opponents.spec.ts]
tech-stack:
  added: []
  patterns: [FetchState enum for per-player status, StoredValue for non-reactive data sharing, collect_view for iterators, ROLES constant array, Callback::new for shared closures]
key-files:
  created: [e2e/tests/opponents.spec.ts]
  modified: [src/pages/opponents.rs]
key-decisions:
  - "EmptyRoleSlot component for missing roles in existing opponent detail panel (avoids None handling complexity)"
  - "teamPage fixture used in e2e tests — opponents require a team to show anything meaningful"
  - "Player sorted by mastery_points descending using sort on recent_champions vec, deduped via HashSet"
  - "Server built from main repo (not worktree) for e2e tests due to parallel execution environment"

patterns-established:
  - "FetchState enum: Idle/Fetching/Success/Error(String) — per-player async status indicator pattern"
  - "StoredValue for non-Copy data (mastery_map, players_by_role) shared across multiple closures"
  - "ROLES constant: [(&str, &str); 5] for role key + display name pairs"

requirements-completed: [OPP-01, OPP-02, OPP-03, OPP-04]

duration: 90min
completed: "2026-03-23"
---

# Phase 10 Plan 03: Opponents Page UI Redesign Summary

**5-role creation form with Save & Fetch, enriched player cards showing OTP badges/mastery pills/recency indicators/collapsible pool analysis, individual and team-level refresh buttons — full opponents page redesign.**

## Performance

- **Duration:** ~90 min
- **Started:** 2026-03-23T12:40:00Z
- **Completed:** 2026-03-23T14:10:00Z
- **Tasks:** 2 of 3 (Task 3 is checkpoint:human-verify)
- **Files modified:** 2

## Accomplishments

- Replaced the old header input + button with `+ New Opponent` button that opens a 5-role inline creation form
- `CreationForm` component: team name input, 5 pre-labeled role rows (Top/Jg/Mid/Bot/Sup) with `Name#Tag` inputs, save-time validation, `Save & Fetch` CTA that creates opponent+players then sequentially fetches intel using returned `player_ids`
- `PlayerCard` redesign: OTP badge (⚠ OTP: {champion} if >60%), recency badge (orange when stale ≥7 days), mastery-sorted champion pills (Ahri M7, Zed M5 format), collapsible Pool Analysis (pool size, role distribution, comfort picks), individual refresh icon button with spinner/check/X status
- `OpponentDetail`: `Refresh All` button, 5-role layout using `ROLES` constant, `confirm_delete` pattern with "Keep Opponent" / "Delete" buttons
- `EmptyRoleSlot` component: inline add slot for missing roles in existing opponent
- `FetchState` enum: `Idle | Fetching | Success | Error(String)` for per-player async status
- E2e smoke test: 3 tests verifying page load, 5-role form appearance, and discard functionality — all passing

## Task Commits

1. **Task 1: Redesign OpponentsPage** - `9686538` (feat)
2. **Task 2: Add e2e smoke test** - `c18e226` (test)
3. **Task 3: Human verification** - PENDING (checkpoint:human-verify)

## Files Created/Modified

- `src/pages/opponents.rs` — Complete rewrite of components (server fns unchanged): `OpponentsPage`, `CreationForm`, `OpponentDetail`, `EmptyRoleSlot`, `PlayerCard` with `FetchState` enum
- `e2e/tests/opponents.spec.ts` — E2e smoke tests for opponents page (3 tests)

## Decisions Made

- `EmptyRoleSlot` handles the case where an existing opponent is missing a player for a given role — shows a quick-add input instead of Nothing
- `teamPage` fixture required for e2e tests (not `authedPage`) — opponents need a team to display non-empty-state content
- Champion pills sorted by mastery points descending, deduped via `seen: HashSet` while preserving mastery sort order
- `StoredValue` used for `mastery_map` (HashMap) and `players_by_role` ([Option<OpponentPlayer>; 5]) to share non-Copy data across multiple closures in the view
- Success auto-clear: `FetchState::Success` transitions back to `Idle` after 3s via `Closure::once` + `setTimeout`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] E2e server infrastructure — parallel worktree had stale WASM**

- **Found during:** Task 2 (e2e test execution)
- **Issue:** The running cargo-leptos process on port 3002 was serving a different project (physics-tree), and building from the worktree produced stale WASM (built from old source). Needed to `cargo leptos build` from the main repo and start a fresh server with the newly-built binary on port 3012.
- **Fix:** Ran `cargo leptos build` from main repo, killed old server processes, started fresh server on port 3012, ran tests against `BASE_URL=http://127.0.0.1:3012`
- **Files modified:** none (infrastructure fix)
- **Verification:** All 3 e2e tests passed

**2. [Rule 1 - Bug] E2e test: Insufficient wait time for WASM hydration after button click**

- **Found during:** Task 2 (e2e test run)
- **Issue:** Tests clicked `+ New Opponent` then immediately checked for form elements — WASM hydration hadn't wired the `on:click` handler yet, so the form never appeared
- **Fix:** Added `page.waitForTimeout(1000)` before click (WASM hydrate), `page.waitForTimeout(500)` after click, and `{ timeout: 10000 }` on expect assertions
- **Files modified:** e2e/tests/opponents.spec.ts
- **Verification:** Tests pass

---

**Total deviations:** 2 auto-fixed (1 blocking infrastructure, 1 bug)
**Impact on plan:** Infrastructure fix required to get tests running against new code. WASM timing fix needed for reliable click-then-verify pattern.

## Known Stubs

None — all data is wired to real server functions. Pool Analysis section shows "Fetch player data to see pool analysis." when no data is available (correct empty state, not a stub).

## Checkpoint Pending

Task 3 is `checkpoint:human-verify` — requires visual verification at `http://127.0.0.1:3002/opponents`:
1. Click "+ New Opponent" → verify 5 role rows appear
2. Enter team name + Riot ID → "Save & Fetch" → verify opponent appears
3. Verify player cards with OTP badge, recency, champion pills, Pool Analysis toggle
4. Test "Refresh All" and individual refresh buttons
5. Verify "Discard Form" works

## Issues Encountered

- Parallel execution environment: worktree builds don't share the running dev server's WASM. Required building from main repo and starting on alternate port. This is a known limitation of the parallel executor in worktrees.

## Next Phase Readiness

- OPP-01 through OPP-04 requirements fully implemented
- Opponents redesign complete pending human visual verification (checkpoint)
- Phase 10 will be complete after checkpoint approval

---
*Phase: 10-opponents-redesign*
*Completed: 2026-03-23 (checkpoint pending)*
