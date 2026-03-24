---
phase: 08-champion-pool-role-ui
plan: 02
subsystem: ui
tags: [leptos, draft, role-icons, community-dragon, surrealdb]

# Dependency graph
requires:
  - phase: 08-01
    provides: champion pool card grid and role-related UI patterns
provides:
  - DraftAction.role field persisted in SurrealDB
  - guess_role_from_tags() function with Marksman>Support>Assassin>Mage>Tank/Fighter>mid priority
  - role_icon_url() shared helper using Community Dragon SVG URLs
  - Role badge overlay on filled pick slots with auto-guess vs user-confirmed visual states
  - Role selection popover with 5 role options (top/jng/mid/bot/sup)
affects: [09-draft-game-plan, future-phases-using-DraftAction]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Role badge overlay: absolute bottom-right w-5 h-5 badge on pick slots with dashed/solid border indicating auto-guess vs user-confirmed"
    - "Role popover: absolute bottom-full popover with 5 Community Dragon SVG role buttons"
    - "Auto-guess on fill_slot: champion tags -> role via guess_role_from_tags, stored with auto flag"
    - "role_assignments + role_auto_guessed parallel signals (Vec<Option<String>>, Vec<bool>) for 20 draft slots"

key-files:
  created: []
  modified:
    - src/models/draft.rs
    - schema.surql
    - src/components/draft_board.rs
    - src/pages/draft.rs
    - src/server/db.rs
    - src/pages/tree_drafter.rs
    - src/pages/team_builder.rs
    - tests/db_drafts.rs
    - tests/db_game_plan_pipeline.rs

key-decisions:
  - "serde(default) on DraftAction.role ensures backward compatibility with saved drafts missing the role key"
  - "Role badge clones role value eagerly before closure capture to avoid Leptos borrow issues in view blocks"
  - "Popover uses roles_list as &[(&str, &str)] with label shortcuts (Jng/Sup) to fit in compact badge"
  - "on_slot_clear clears both role_assignments and resets role_auto_guessed to true for that slot"
  - "Loaded drafts have all role_auto_guessed set to false (user-confirmed) even if role is None"

requirements-completed: [UX-10]

# Metrics
duration: 8min
completed: 2026-03-23
---

# Phase 08 Plan 02: Role Badges on Draft Pick Slots Summary

**Role icon badges on draft pick slots with auto-guess from champion tags, Community Dragon SVGs, and popover selection — persisted via DraftAction.role field in SurrealDB**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-23T09:09:49Z
- **Completed:** 2026-03-23T09:18:00Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Added `pub role: Option<String>` with `serde(default)` to DraftAction struct and SurrealDB schema — backward compatible with all existing saved drafts
- Implemented `guess_role_from_tags()` with priority-ordered tag matching (Marksman→bot, Support→support, Assassin/Mage→mid, Tank/Fighter→top, fallback mid) and 8 unit tests
- Added role badge overlay on filled pick slots: auto-guessed shows at 50% opacity with dashed border; user-confirmed shows full opacity with solid accent border
- Role selection popover with 5 Community Dragon SVG role buttons fires `on_role_set` callback to mark slot as user-confirmed

## Task Commits

1. **Task 1: Add role field to DraftAction model, schema, and guess_role_from_tags** - `2c3d923` (feat) — TDD: RED tests first, GREEN implementation
2. **Task 2: Add role badge overlay and popover to DraftBoard pick slots** - `0819c9b` (feat)

## Files Created/Modified
- `src/models/draft.rs` - Added role field, guess_role_from_tags(), role_icon_url(), 11 new tests
- `schema.surql` - Added DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>
- `src/components/draft_board.rs` - Role badge overlay, popover with 5 role buttons, role_popover_open signal
- `src/pages/draft.rs` - role_assignments + role_auto_guessed signals, fill_slot auto-guess, on_role_set_cb, on_slot_clear role reset, load from saved draft
- `src/server/db.rs` - DbDraftAction.role field, FROM impl, save/update queries include role
- `src/pages/tree_drafter.rs` - Added role: None to DraftAction struct init
- `src/pages/team_builder.rs` - Added role: None to DraftAction struct init
- `tests/db_drafts.rs` - Added role: None to all DraftAction test instances
- `tests/db_game_plan_pipeline.rs` - Added role: None to all DraftAction test instances

## Decisions Made
- Used `#[serde(default)]` on DraftAction.role for backward compatibility — serde already treats Option as optional for missing JSON keys, but explicit default annotation makes intent clear
- Roles popover uses abbreviated labels (Jng/Sup) to fit the compact badge; full role name used for title tooltip
- `on_slot_clear` resets role_auto_guessed to true for the cleared slot (so next fill gets auto-guessed again)
- Loaded drafts set all role_auto_guessed to false — if a user saved a draft with role data, it's treated as confirmed

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated all DraftAction struct initializations across codebase**
- **Found during:** Task 1 (implementing role field)
- **Issue:** Adding `role` field to DraftAction struct caused compile errors in db.rs, tree_drafter.rs, team_builder.rs, and integration test files
- **Fix:** Added `role: None` to all DraftAction literal initializations, added `role` field to DbDraftAction, updated FROM impl, updated save/update SurQL queries to bind role
- **Files modified:** src/server/db.rs, src/pages/tree_drafter.rs, src/pages/team_builder.rs, tests/db_drafts.rs, tests/db_game_plan_pipeline.rs
- **Verification:** cargo check --features ssr, cargo check --features hydrate, cargo test --lib all pass
- **Committed in:** 2c3d923 (Task 1 commit)

**2. [Rule 1 - Bug] build_actions needed role parameter from the start**
- **Found during:** Task 1 (updating build_actions signature in draft.rs)
- **Issue:** Plan intended role param to be added in Task 2 but build_actions is called in auto-save Effect which needed the signals defined first
- **Fix:** Added role_assignments + role_auto_guessed signals and updated build_actions calls alongside Task 1 to keep compilation clean between tasks
- **Files modified:** src/pages/draft.rs
- **Verification:** Both compile targets pass
- **Committed in:** 0819c9b (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - required for compilation/correctness)
**Impact on plan:** Necessary cascading changes from adding a new struct field. No scope creep.

## Issues Encountered
- None beyond the expected cascade of DraftAction struct changes across the codebase

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- DraftAction.role persisted and loaded correctly
- Role badges visible on draft pick slots
- Phase 09 (Draft & Game Plan) can use DraftAction.role for game plan role assignment
- The role_assignments signal is available for future pool-gap checking against player roles

---
*Phase: 08-champion-pool-role-ui*
*Completed: 2026-03-23*
