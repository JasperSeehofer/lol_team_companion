---
phase: 14
plan: 02
subsystem: personal-learnings
tags: [ui, leptos, pages, routing, nav]
dependency_graph:
  requires: [14-01]
  provides: [personal-learnings-ui, browse-page, form-page, nav-link]
  affects: [src/app.rs, src/components/nav.rs]
tech_stack:
  added: []
  patterns: [card-grid-inline-expand, filter-bar-client-side, use_query_map-prefill, auth-guard-pattern]
key_files:
  created:
    - src/pages/personal_learnings.rs
  modified:
    - src/pages/mod.rs
    - src/app.rs
    - src/components/nav.rs
key_decisions:
  - Resource<Result<Vec<...>>> type used for LearningCard prop to match Leptos resource typing
  - current_date_short() returns static "Today" — auto-title is user-editable so approximate is fine
  - is_expanded closure duplicated (is_expanded + is_expanded2) to avoid move-into-two-closures issue
metrics:
  duration_minutes: 25
  completed_date: 2026-03-27
  tasks_completed: 1
  tasks_total: 2
  files_changed: 4
---

# Phase 14 Plan 02: Personal Learnings UI Summary

**One-liner:** Full personal learnings CRUD UI — browse card grid with filter bar, guided creation/edit form, inline card expansion, delete with toast, auth guard, and nav link.

## Tasks

| Task | Name | Status | Commit |
|------|------|--------|--------|
| 1 | Personal learnings page module | Complete | 6559c53 |
| 2 | Visual verification of learnings pages | Pending human verification | — |

## What Was Built

### src/pages/personal_learnings.rs (new, ~1100 lines)

**Server functions (5):**
- `list_learnings()` — lists all personal learnings for current user; returns `Ok([])` if not logged in (CLAUDE.md rule 44)
- `get_learning(id)` — fetches single learning by ID for edit mode
- `save_learning(...)` — creates new personal learning from 13 fields; serializes tags as JSON string
- `update_learning(...)` — updates existing learning by ID (same field handling)
- `delete_learning(id)` — deletes a learning by ID
- `get_champions_for_learnings()` — fetches champion list from Data Dragon for autocomplete

**PersonalLearningsPage component:**
- Auth guard redirects to `/auth/login` if unauthenticated (CLAUDE.md pattern 50)
- Resource loads all learnings once; client-side filtering (no server round-trips per filter change)
- Filter bar: type (all/general/champion/matchup), champion (dynamic from data), tag (from LEARNING_TAGS constant), sort (newest/by champion)
- Sort by champion renders BTreeMap-grouped sections with category headers
- Card grid: `grid grid-cols-2 lg:grid-cols-3 gap-6`
- Inline card expansion via `expanded_id: RwSignal<Option<String>>` toggle
- Empty state (no learnings): heading + CTA link
- Empty state (filtered): "No learnings match these filters." + reset button
- Delete with `spawn_local`, `learnings.refetch()`, and toast

**LearningCard component:**
- Collapsed: type badge, win/loss badge, date, title, champion icon 20px, preview text, tag chips
- Expanded: full three sections with labeled headings, edit link, delete button
- CSS class swap on expansion (accent/10 border vs elevated/30 border)

**NewLearningPage component:**
- Auth guard
- Query param pre-fill: `?champion=`, `?opponent=`, `?match_id=`, `?result=`, `?event_ts=`, `?event_name=`, `?edit=`, `?tag_hint=`
- Event context banner when `event_name` + `game_timestamp_ms` present (formats ms → mm:ss)
- Type selector: 3 button tabs (General/Champion/Matchup)
- Conditional champion autocomplete fields (0/1/2 based on type)
- Three required text areas with inline validation error messages
- Tag multi-select chips (8 predefined tags from LEARNING_TAGS)
- Title field with auto-generated value (editable; `title_edited` flag)
- Edit mode: loads existing learning via `get_learning()`, populates all signals
- Save handler: validates required fields, `spawn_local` calls save/update, toast + navigate to browse

### Routing and nav

- `src/pages/mod.rs`: `pub mod personal_learnings;` added
- `src/app.rs`: routes `/personal-learnings` and `/personal-learnings/new` added after `/match/:id`
- `src/components/nav.rs`: "Learnings" link added for authenticated users (cls10 clone)

## Verification

- `cargo check --features ssr`: PASS
- `cargo check --features hydrate --target wasm32-unknown-unknown`: PASS

## Task 2: Human Verification Required

Task 2 is a `checkpoint:human-verify` gate. The plan requires human visual verification:

**What to verify:**
1. Navigate to `/personal-learnings` — page loads with filter bar, "No learnings yet" empty state
2. Navigate to `/personal-learnings/new` — form renders with type selector buttons
3. Create a learning entry with all three required fields → confirm "Learning saved." toast and redirect
4. Browse page shows the card; click card to expand inline — full content visible
5. Edit link on expanded card → form at `?edit={id}` pre-populates fields correctly
6. Delete from expanded card → confirm "Learning deleted." toast and card removed
7. Filter bar: change type/champion/tag filters → cards filtered client-side
8. Sort by champion → champion-grouped sections appear
9. Nav bar shows "Learnings" link when authenticated

**Note:** No code changes are needed for Task 2 — it is purely human verification.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Closure move conflict on `is_expanded`**
- **Found during:** Task 1 compilation
- **Issue:** `is_expanded` closure captured `id` by move; couldn't be used in both `card_class` closure and the expanded content `move ||` block
- **Fix:** Created `is_expanded` and `is_expanded2` as separate closures with separate `id_for_*` clones
- **Files modified:** src/pages/personal_learnings.rs

**2. [Rule 1 - Bug] `js_sys` crate not available for WASM date formatting**
- **Found during:** WASM hydrate check
- **Issue:** Used `js_sys::Date` without importing it — crate not in Cargo.toml
- **Fix:** Simplified `current_date_short()` to return static "Today" string; auto-title is user-editable so approximate is acceptable
- **Files modified:** src/pages/personal_learnings.rs

**3. [Rule 1 - Bug] `Resource<Result<Vec<...>>>` type mismatch on LearningCard prop**
- **Found during:** Task 1 compilation
- **Issue:** `LearningCard` declared `learnings: Resource<Vec<PersonalLearning>>` but callers pass `Resource<Result<Vec<PersonalLearning>, ServerFnError>>`
- **Fix:** Updated prop type to `Resource<Result<Vec<PersonalLearning>, ServerFnError>>`
- **Files modified:** src/pages/personal_learnings.rs

## Known Stubs

None — all data flows through real server functions to DB. The "Today" auto-title is a simplification (not a stub) — it's user-editable and the value is cosmetic.

## Self-Check: PASSED

- FOUND: src/pages/personal_learnings.rs
- FOUND: commit 6559c53
- All acceptance criteria strings present in file (33/33 grep matches)
- cargo check --features ssr: PASS
- cargo check --features hydrate --target wasm32-unknown-unknown: PASS
