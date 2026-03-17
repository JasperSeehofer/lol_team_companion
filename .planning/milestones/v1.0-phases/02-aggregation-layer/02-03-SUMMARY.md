---
phase: 02-aggregation-layer
plan: 03
subsystem: ui
tags: [rust, leptos, champion-normalization, autocomplete, wasm]

# Dependency graph
requires:
  - 02-01 (ChampionAutocomplete stores canonical ID in value signal, normalize_champion_name)
  - 02-02 (migrate_champion_names function in db.rs, init_db wires startup migration)
provides:
  - Champion ID storage enforced at all picker/autocomplete ingestion points
  - Human-readable champion names displayed in UI via ID-to-name lookup
  - Consistent canonical IDs in tree drafter draft slots, champion pool, and game plan matchups
affects:
  - Phase 3 (dashboard data relies on canonically normalized champion fields)
  - Phase 4 (draft intel panels rely on consistent champion IDs across game plans and pools)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ID-to-name display: champs.iter().find(|c| c.id == val).map(|c| c.name.clone()).unwrap_or(val)"
    - "champion_map keyed by c.id (not c.name) so draft slots storing canonical IDs resolve correctly"
    - "ChampionPicker drag data and is_used check use champion.id (not champion.name)"

key-files:
  created: []
  modified:
    - src/components/champion_picker.rs
    - src/pages/champion_pool.rs
    - src/pages/game_plan.rs
    - src/pages/tree_drafter.rs

key-decisions:
  - "champion_map in DraftBoard/TreeGraph keyed by c.id throughout — slots store canonical IDs post Plan 01 so map keys must match"
  - "champion_pool tier list and detail panel show human-readable name via c.id == champ lookup with fallback to raw ID"
  - "game_plan locked champion display and role-strategy matchup label both resolve IDs to display names"
  - "migrate_champion_names not re-added to main.rs — Plan 02 already wired it inside init_db (before Arc::new); re-adding would run migration twice"

patterns-established:
  - "ID-to-name resolution pattern: find by c.id, map to c.name, fallback to raw id if not found"
  - "champion_picker.rs: all outputs (drag data, on_select callback, is_used check) use champion.id"

requirements-completed: []

# Metrics
duration: 25min
completed: 2026-03-15
---

# Phase 2 Plan 03: Champion Ingestion Normalization — Pages and Startup Migration

**Canonical ID storage enforced in tree_drafter, champion_pool, and game_plan; champion_map keys changed to c.id; human-readable display via ID-to-name lookup throughout**

## Performance

- **Duration:** 25 min
- **Started:** 2026-03-15T07:10:00Z
- **Completed:** 2026-03-15T07:35:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- `tree_drafter.rs`: `fill_slot` now stores `champ.id` instead of `champ.name`; all `champion_map` instances (NodeEditor, LiveNavigator, graph view) keyed by `c.id` to match canonical ID slots
- `champion_picker.rs`: `is_used` check and drag data both use `champion.id` (was `champion.name`), so the picker correctly grays out already-picked champions and drag-drop sends canonical IDs
- `champion_pool.rs`: image URL and display name lookup uses `c.id == champ` (not `c.name`); human-readable names shown via `display_name` resolved from canonical ID
- `game_plan.rs`: locked champion display and role-strategy matchup label both resolve canonical IDs to human names via the `champ_list` lookup; falls back to raw ID if Data Dragon list not yet loaded
- Both compile targets pass: `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown`
- Startup migration (Task 2): already wired in Plan 02 via `init_db` — no duplicate call needed

## Task Commits

1. **Task 1: Update game_plan.rs, champion_pool.rs, and tree_drafter.rs** - `adcadb7` (feat)

Task 2 (migration wiring) was completed by Plan 02 inside `init_db`. No separate commit needed.

## Files Created/Modified
- `src/components/champion_picker.rs` - `is_used` and drag data use `champion.id`
- `src/pages/champion_pool.rs` - image/name lookup by `c.id`; `display_name` shown in tier list and detail panel
- `src/pages/game_plan.rs` - locked display and role-strategy matchup label resolve IDs to display names
- `src/pages/tree_drafter.rs` - `fill_slot` uses `champ.id`; all `champion_map` instances keyed by `c.id`

## Decisions Made
- `champion_map` throughout tree_drafter (NodeEditor, LiveNavigator, graph view) changed to `c.id` key — slots store canonical IDs after Plan 01 fix, so map keys must match for icon lookups in DraftBoard/TreeGraph
- `champion_picker.rs` drag data now sends `champ.id` instead of `champ.name` — drag-drop to DraftBoard creates slot entries as canonical IDs
- `is_used` in champion_picker checks `champion.id` (not `.name`) because `used_champions` derived from `draft_slots` now contains canonical IDs
- Migration not duplicated in `main.rs` — Plan 02 pre-wired it inside `init_db`; behavior is identical (schema applied first, then migration)

## Deviations from Plan

### Pre-completed Items

**1. [Pre-completed by Plan 02] Task 2: Startup migration wiring**
- **Expected:** Add `migrate_champion_names` call to `main.rs` after `init_db`
- **Actual:** Plan 02 already added the call inside `init_db` in `db.rs` (committed as `6a7e7f5`)
- **Outcome:** Migration runs on startup after schema init — identical behavior to plan intent
- **Decision:** No duplicate call added to `main.rs`; calling twice would be wasteful and confusing

### Auto-fixed Issues

**2. [Rule 1 - Bug] Fixed champion_picker drag data and is_used to use champion.id**
- **Found during:** Task 1 (reviewing all champion ingestion points)
- **Issue:** `champion_picker.rs` sent `champ_for_drag.name` as drag data and checked `used_champions.contains(&champion.name)` — after our changes, slots store IDs so drag-drop would insert display names instead of IDs, and is_used would never match
- **Fix:** Changed both to use `champion.id`
- **Files modified:** `src/components/champion_picker.rs`
- **Verification:** Both compile targets pass
- **Committed in:** adcadb7

---

**Total deviations:** 1 pre-completed (Plan 02 early wiring), 1 auto-fixed (Rule 1 bug in champion_picker)
**Impact on plan:** Both handled correctly. No scope creep. Startup migration is live.

## Issues Encountered
None beyond the pre-completed Task 2 noted above.

## User Setup Required
None — no external service configuration required.

## Next Phase Readiness
- All champion input points now store canonical Data Dragon IDs
- Startup migration normalizes existing records on next server restart
- Phase 3 (smart dashboard) can rely on consistent champion IDs in pool, game plans, and draft actions
- Phase 4 (draft intel sidebar) champion lookups will be consistent across all data sources

---
*Phase: 02-aggregation-layer*
*Completed: 2026-03-15*
