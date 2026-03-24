---
phase: "09-draft-game-plan"
plan: "02"
subsystem: "draft-intel-notes"
tags: ["notes-tab", "pool-notes", "intel-sidebar", "auto-open"]
dependency_graph:
  requires: []
  provides: ["pool-notes-in-draft", "notes-tab-intel-sidebar", "auto-open-notes-effect"]
  affects: ["src/pages/draft.rs", "src/server/db.rs"]
tech_stack:
  added: []
  patterns: ["resource-driven-tab", "collapsible-sections", "auto-open-effect"]
key_files:
  created: []
  modified:
    - src/pages/draft.rs
    - src/server/db.rs
decisions:
  - "Used champion name matching (not ID) for pool notes query since champion_note table stores by name"
  - "Collapsed all note types except matchup by default per UI-SPEC"
  - "Auto-open effect tracks pick count to detect newly added champions"
metrics:
  duration_minutes: 30
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_modified: 2
---

# Phase 09 Plan 02: Notes Tab for Champion Pool Notes

Added a 4th Notes tab to the intel sidebar that surfaces champion pool notes (matchups, power spikes, combos, lessons, synergies, positioning) for our-side picked champions, with auto-open behavior when a pooled champion is picked.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | DB query + server fn for pool notes by champion list | 392c23c | src/server/db.rs, src/pages/draft.rs |
| 2 | Notes tab UI with champion sub-tabs, collapsible sections, auto-open Effect | 9c0b79b | src/pages/draft.rs |

## What Was Built

**Task 1 — DB Query + Server Function (DRFT-05):**

- `get_pool_notes_for_champions()` in `db.rs`: queries `champion_note` table for all notes matching a list of champions, using `AND champion IN $champions`, ordered by champion, note_type, created_at DESC
- Server fn in `draft.rs`: takes `champions_json: String` (per CLAUDE.md rule 33), deserializes, calls DB function, groups results by champion, returns `Vec<(String, Vec<ChampionNote>)>`

**Task 2 — Notes Tab UI (D-10 through D-14):**

- Added "notes" as 4th tab in intel sidebar tab array (D-10)
- New signals: `notes_champion_tab`, `collapsed_note_types` (matchup expanded by default), `prev_our_picks_count`
- `our_picks_for_notes()` closure: derives our-side pick champions from draft_slots using slot_meta
- `pool_notes` Resource: reactive on our_picks_for_notes, calls server fn with JSON-serialized champion list
- Auto-open Effect (D-13): watches pick count, when a new champion is picked and has pool notes, auto-opens intel sidebar to Notes tab for that champion
- Auto-select Effect: when Notes tab is opened with no sub-tab selected, selects first our-side pick
- Champion sub-tab row with active/inactive styling (D-11)
- Per-champion notes content (D-12): groups notes by note_type using NOTE_TYPES constant, renders collapsible sections with chevron toggle via Callback::new
- Empty state per champion (D-14): "No notes for {champion} yet." with link to Champion Pool
- Empty state when no picks: "Pick champions on your side to see pool notes."
- Used `collect_view()` for all iterator rendering (CLAUDE.md rule 25)
- Used `.into_any()` for all divergent view branches (CLAUDE.md rule 19)

## Verification

- `cargo check --features ssr`: PASS
- `cargo check --features hydrate --target wasm32-unknown-unknown`: PASS
- `cargo test --features ssr --lib`: PASS (69 tests, 0 failed)

## Deviations from Plan

**1. [Rule 1 - Bug] Import interference from parallel agent**
- **Found during:** Task 2 verification
- **Issue:** The 09-01 agent removed `note_type_label` and `NOTE_TYPES` imports since they were unused at the time. The Notes tab code references both.
- **Fix:** Re-added the imports in a separate commit after the Notes tab UI was built
- **Files modified:** src/pages/draft.rs

## Known Stubs

None — pool_notes Resource pulls live data from the champion_note table. Collapsible sections and auto-open are fully wired.

## Self-Check: PASSED
