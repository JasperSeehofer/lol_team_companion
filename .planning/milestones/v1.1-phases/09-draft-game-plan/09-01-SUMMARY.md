---
phase: "09-draft-game-plan"
plan: "01"
subsystem: "draft-opponents"
tags: ["opponent-autocomplete", "intel-sidebar", "ban-filtering", "ux"]
dependency_graph:
  requires: []
  provides: ["opponent-autocomplete-in-draft", "back-to-draft-link", "ban-filtered-matchups"]
  affects: ["src/pages/draft.rs", "src/pages/opponents.rs"]
tech_stack:
  added: []
  patterns: ["searchable-dropdown-inline", "query-param-return-link", "ban-derived-filter"]
key_files:
  created: []
  modified:
    - src/pages/draft.rs
    - src/pages/opponents.rs
decisions:
  - "Used ToastKind::Error (not Warning, which doesn't exist) for name-required toast"
  - "Removed NOTE_TYPES and note_type_label imports that were auto-added by formatter from another plan's work"
  - "Added else branch to tab if-chain when reformatter changed else to else-if current_tab==matchups"
metrics:
  duration_minutes: 35
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_modified: 2
---

# Phase 09 Plan 01: Draft-Opponent Connection Summary

Opponent autocomplete dropdown in draft header connected to scouted opponents list, with auto-open intel sidebar on selection, Add New Opponent auto-save navigation, ban filtering in matchups tab, and back button in champion detail view.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Opponent autocomplete dropdown + Add New button + intel sidebar cleanup | 815da42 | src/pages/draft.rs, src/pages/opponents.rs |
| 2 | Ban filtering in matchups tab + back button in champion detail | bad077d | src/pages/draft.rs |

## What Was Built

**Task 1 — Opponent Autocomplete + Add New + Intel Cleanup (DRFT-01, DRFT-02, D-01 through D-06):**

- Replaced the free-text opponent `<input>` in the draft header with an inline searchable dropdown following the ChampionAutocomplete pattern
- Signals added: `opp_filter_text`, `opp_dropdown_open`
- `filtered_opponents` closure filters the existing `opponents_list` Resource by the typed text
- On:blur uses 150ms delayed close via `Closure::once` + `set_timeout` (same CLAUDE.md pattern as ChampionAutocomplete)
- On opponent select via mousedown: sets `opponent` (ID), `opp_filter_text` (display name), closes dropdown, and auto-opens intel sidebar by setting `selected_opponent_id` + `intel_open = true` (D-04)
- Backward compat (D-05): on draft load, checks if stored value matches an opponent ID; if yes, sets display name and `selected_opponent_id`; if no, uses the raw string as legacy free-text
- "Add New Opponent" button: eagerly captures all draft signals, validates name (shows error toast if empty), calls save_draft or update_draft, then navigates to `/opponents?return_to=draft&draft_id={id}`
- Removed duplicate opponent `<select>` from intel sidebar "Their Picks" tab (D-06) — replaced with a contextual hint message when no opponent is selected in the header
- Opponents page: reads `return_to` and `draft_id` query params; renders `← Back to Draft` link above the page heading when `return_to == "draft"`

**Task 2 — Ban Filtering + Back Button (DRFT-03, DRFT-04, D-07, D-08):**

- `banned_champions` closure derives list of banned champion names from draft slots using `slot_meta` to identify ban slots
- Matchup champion selection grid (when `matchup_champion` is None) now filters out banned champions before rendering the clickable list (D-07); shows "No non-banned champions drafted yet" when all picks are banned
- Back button added at top of champion detail view in Matchups tab: "← Back to champions" that sets `matchup_champion = None` (D-08)

## Verification

- `cargo check --features ssr`: PASS
- `cargo check --features hydrate --target wasm32-unknown-unknown`: PASS
- `cargo test --features ssr --lib`: PASS (69 tests, 0 failed)

## Deviations from Plan

**1. [Rule 1 - Bug] ToastKind::Warning does not exist**
- **Found during:** Task 1
- **Issue:** Plan specified `ToastKind::Warning` but the enum only has `Success` and `Error`
- **Fix:** Used `ToastKind::Error` for the name-required validation message
- **Files modified:** src/pages/draft.rs

**2. [Rule 1 - Bug] Unused imports from formatter interference**
- **Found during:** Task 2 verification
- **Issue:** `note_type_label` and `NOTE_TYPES` appeared in the import line — added by formatter from another parallel agent's plan (09-02 adds notes tab). Caused warnings
- **Fix:** Removed the unused imports; they will be re-added by plan 09-02 when the Notes tab is built
- **Files modified:** src/pages/draft.rs

**3. [Rule 1 - Bug] Tab if-chain lost else branch after formatter**
- **Found during:** Task 2 verification
- **Issue:** `} else {` at end of pool/their_picks/matchups chain was reformatted to `} else if current_tab == "matchups" {` without a final `else` arm, causing E0317 compiler error
- **Fix:** Added final `else { view! { <span></span> }.into_any() }` arm to complete the expression
- **Files modified:** src/pages/draft.rs

**4. [Rule 2 - Missing functionality] opp_filter_text reset on New Draft / Clear**
- **Found during:** Task 1
- **Issue:** The existing "New Draft"/"Clear" button reset `set_opponent` but not the new `set_opp_filter_text`, which would leave stale display text in the input
- **Fix:** Added `set_opp_filter_text.set(String::new())` alongside the existing `set_opponent.set(String::new())` in the clear handler
- **Files modified:** src/pages/draft.rs

## Known Stubs

None — all functionality is wired. The opponents list dropdown pulls from the live `opponents_list` Resource. The `banned_champions` closure derives from live `draft_slots`. The return_to link reads from real query params.

## Self-Check: PASSED
