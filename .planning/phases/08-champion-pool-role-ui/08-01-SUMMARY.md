---
phase: 08-champion-pool-role-ui
plan: 01
subsystem: ui
tags: [leptos, tailwind, drag-and-drop, champion-pool, autocomplete]

# Dependency graph
requires: []
provides:
  - Champion pool card grid layout with portrait, name, comfort stars, meta badge, stats line
  - Drag-and-drop tier changes between tier buckets via DataTransfer API
  - Matchup note entry using ChampionAutocomplete with auto-fill "vs {name}" title
  - Opponent champion icon display in matchup notes list
affects:
  - 08-02 (draft role badges — same phase, separate plan)
  - 09 (game plan autofill from draft — builds on champion pool UX)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Drag-and-drop between tier buckets: dragstart sets DataTransfer (text/x-champion, text/x-role, text/x-source-tier); tier div listens on:dragover/on:dragleave/on:drop; dragging_over_tier RwSignal drives drop-zone highlight; dragging_champ RwSignal drives source card opacity"
    - "Card grid via CSS grid (grid grid-cols-3 gap-2) replacing flex wrap pill layout"
    - "Matchup autocomplete: ChampionAutocomplete with on_select Callback that auto-fills note_form_title as 'vs {name}'; matchup_opponent_value RwSignal stores the autocomplete input state"
    - "Opponent icon lookup: StoredValue<HashMap<String, Champion>> keyed by champion name; strip_prefix('vs ') to extract opponent name and map to image_full URL"

key-files:
  created: []
  modified:
    - src/pages/champion_pool.rs

key-decisions:
  - "Both tasks implemented in a single commit to champion_pool.rs since they are tightly coupled and the file cannot be partially committed without a broken intermediate state"
  - "dragging_over_tier and dragging_champ added as page-level RwSignals rather than per-tier state to avoid clone explosion across the TIERS iterator"
  - "matchup_opponent_value is a separate RwSignal from note_form_title to allow ChampionAutocomplete to display the champion name while the title stores 'vs {name}'"

patterns-established:
  - "Card-grid champion layout: relative bg-elevated border rounded-xl overflow-hidden cursor-pointer group; w-full aspect-square object-cover object-top for portrait; absolute top-1 right-1 opacity-0 group-hover:opacity-100 for remove button"

requirements-completed: [UX-08]

# Metrics
duration: 18min
completed: 2026-03-23
---

# Phase 08 Plan 01: Champion Pool Card Grid + Drag-and-Drop + Matchup Autocomplete Summary

**Champion pool overhauled from pill entries to 3-column card grid with portrait photos, drag-and-drop tier management via DataTransfer API, and ChampionAutocomplete-based matchup note entry with opponent icon display**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-03-23T09:00:00Z
- **Completed:** 2026-03-23T09:18:19Z
- **Tasks:** 2 (combined into 1 commit — same file, tightly coupled)
- **Files modified:** 1

## Accomplishments

- Champion pool tier buckets now render a 3-column card grid with full champion portraits (aspect-square object-cover), comfort stars, meta badge, and match stats line
- Dragging a card between tier buckets calls `set_champion_tier` server fn and refetches the pool; source card goes semi-transparent during drag; target tier shows accent border highlight
- Matchup note form shows `ChampionAutocomplete` instead of free-text input when note type is "matchup"; selecting a champion auto-fills title as "vs {name}"
- Matchup notes in the list display a 24px rounded opponent champion icon extracted via `strip_prefix("vs ")`

## Task Commits

Both tasks combined in one atomic commit since they modify the same file:

1. **Task 1: Card grid + drag-and-drop** - `f76d69d` (feat)
2. **Task 2: Matchup autocomplete + opponent icon** - `f76d69d` (feat — same commit)

## Files Created/Modified

- `/home/jasper/Repositories/lol_team_companion/.claude/worktrees/agent-ad6e360b/src/pages/champion_pool.rs` — Card grid layout, drag-and-drop signals and handlers, matchup autocomplete, opponent icon lookup

## Decisions Made

- Combined both tasks into one commit since both modify `champion_pool.rs` and separating them would require a broken intermediate state
- Used `dragging_over_tier: RwSignal<Option<&'static str>>` at page level so each TIERS iterator item captures the same signal without per-tier clones
- Added `matchup_opponent_value: RwSignal<String>` separate from `note_form_title` to let ChampionAutocomplete control its own display text while the title stores the full "vs {name}" string

## Deviations from Plan

None — plan executed exactly as written. All acceptance criteria met.

## Issues Encountered

None. Both SSR and WASM compile targets pass. All 59 unit tests pass.

## Known Stubs

None — champion portrait images, comfort stars, meta badges, and match stats are all wired to live data from `champions_resource`, `pool`, and `stats_resource` resources.

## Next Phase Readiness

- Plan 08-02 (draft role badges) is independent and can run immediately
- Phase 09 (game plan autofill from draft) can leverage the improved champion pool UX

---
*Phase: 08-champion-pool-role-ui*
*Completed: 2026-03-23*
