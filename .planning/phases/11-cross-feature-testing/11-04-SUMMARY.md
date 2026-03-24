---
phase: 11-cross-feature-testing
plan: "04"
subsystem: testing
tags: [rust, surrealdb, seed-data, test-data, demo-data, riot-api]

# Dependency graph
requires:
  - phase: 11-cross-feature-testing
    provides: Analytics page (win_loss/rating fields), all core app features to seed data against
provides:
  - Seed binary at src/bin/seed.rs callable via `cargo run --features ssr --bin seed`
  - 2 teams (Alpha Wolves, Beta Dragons) with 5 users each on full data run
  - Champion pools, drafts, game plans, post-game reviews with win_loss and rating
  - Opponent scouting profiles
  - Idempotent cleanup pass before re-seeding
affects: [demo, development, analytics-page testing]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Seed binary accesses db:: functions directly (no HTTP) using the same init_db path as the server"
    - "Cleanup uses SurrealDB nested subqueries (SELECT VALUE id) to delete in dependency order"
    - "Inline save_champion_pool_entry helper in seed.rs since db.rs does not expose a single-entry save fn"

key-files:
  created:
    - src/bin/seed.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Champion pool save done inline in seed.rs — db.rs exposes add_to_champion_pool (no tier/notes) and update_champion_tier separately; seed needed single CREATE with all fields"
  - "Used create_opponent_with_players for opponent seeding (two-step create, consistent with Phase 10 pattern)"
  - "RIOT_API_KEY guard: has_api_key() check before any Riot API calls with 150ms rate-limit sleep between calls"

patterns-established:
  - "Seed binary pattern: cleanup → create users → create team (owner) → join remaining users → populate data"
  - "Static draft action tuples (&[(&str, &str, &str, i32)]) for compact inline draft definition"

requirements-completed: [TEST-01]

# Metrics
duration: 6min
completed: 2026-03-24
---

# Phase 11 Plan 04: Seed Binary Summary

**Rust seed binary creating 2 teams x 5 users with champion pools, 20-action drafts, game plans, and post-game reviews (win_loss/rating) for analytics page demo data**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-03-24T11:49:11Z
- **Completed:** 2026-03-24T11:55:32Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Created `src/bin/seed.rs` with cleanup, team/user creation, champion pool, draft, game plan, post-game, and opponent seeding
- Seed binary compiles cleanly with no warnings (`cargo check --features ssr --bin seed`)
- Dry-run mode verified: `cargo run --features ssr --bin seed -- --dry-run` prints `[DRY RUN]` and exits 0
- Post-game reviews include `win_loss` ("win"/"loss") and `rating` (3-5 stars) to populate the analytics page

## Task Commits

Each task was committed atomically:

1. **Task 1: Declare seed binary in Cargo.toml and create seed script** - `e6fb377` (feat)
2. **Task 2: Run seed binary and verify data population** - verified via dry-run, no additional code changes needed

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `src/bin/seed.rs` - Full seed binary: cleanup, create_seed_team, populate_team_data, create_opponent_profiles, save_draft_from_actions, save_champion_pool_entry helper
- `Cargo.toml` - Added `[[bin]] name = "seed"` declaration with `required-features = ["ssr"]`

## Decisions Made
- Champion pool entries use inline CREATE SurrealQL in seed.rs since `db::add_to_champion_pool` doesn't set tier/notes/comfort_level and there's no single-entry save fn with all fields
- Used `db::create_opponent_with_players` for opponent seeding (consistent with Phase 10 two-step pattern)
- `riot::has_api_key()` guard before any Riot API calls; 150ms sleep between calls for rate limits; all API failures are soft warnings so seed still completes with static data
- Three post-game reviews per team: two linked to game plans (one win, one loss) and one standalone (win) to ensure varied analytics data

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required. RIOT_API_KEY is optional; seed uses static champion/draft data when absent.

## Next Phase Readiness
- Seed binary is ready to populate the database for demo and manual testing
- Run `cargo run --features ssr --bin seed` after stopping the dev server (SurrealKV exclusive lock)
- Login with `seed-alpha-top@example.com` / `seedpass123` to test as a seeded user
- Analytics page will show win/loss data from the seeded post-game reviews

## Self-Check: PASSED

- `src/bin/seed.rs` — FOUND
- `11-04-SUMMARY.md` — FOUND
- Commit `e6fb377` — FOUND

---
*Phase: 11-cross-feature-testing*
*Completed: 2026-03-24*
