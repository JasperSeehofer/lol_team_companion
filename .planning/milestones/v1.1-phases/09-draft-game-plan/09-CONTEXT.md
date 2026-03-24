# Phase 9: Draft & Game Plan - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Connect the draft page to opponent context via a structured opponent dropdown, surface champion pool notes in the intel sidebar when pooled champions are picked, improve intel panel UX (ban filtering, back button), and enhance game plan creation to auto-fill name, strategy tag, champions, and per-role strategy fields from the source draft. Covers DRFT-01 through DRFT-05, PLAN-01, and PLAN-03.

</domain>

<decisions>
## Implementation Decisions

### Opponent Selection (DRFT-01, DRFT-02)
- **D-01:** Replace the free-text "Opponent" input in the draft header with a searchable dropdown of scouted opponents + a separate "Add New" button next to it
- **D-02:** Searchable dropdown (type-ahead filter) rather than a plain `<select>`, even though opponent lists are small — user preference
- **D-03:** "Add New Opponent" button navigates to `/opponents` page after auto-saving the current draft; includes a return link back to the draft
- **D-04:** Selecting an opponent auto-opens the intel sidebar to show their data immediately
- **D-05:** Store the opponent record ID in `Draft.opponent` (not the display name). Add a free-text fallback for unlinked opponents (backward compat with existing drafts that have name strings)
- **D-06:** Remove the duplicate opponent `<select>` from the intel sidebar's "Their Picks" tab — header dropdown is the single source of truth; intel sidebar auto-syncs from `selected_opponent_id`

### Intel Panel UX (DRFT-03, DRFT-04)
- **D-07:** Banned champions are filtered out of the matchup champion selection only (not from "Our Pools" tab click-to-pick)
- **D-08:** Add a back button to the existing champion detail view in the intel panel to navigate back to the champion selection list
- **D-09:** No changes to the "Our Pools" tab ban filtering — pools tab remains informational

### Pool Notes Sidebar (DRFT-05)
- **D-10:** Add a 4th tab "Notes" to the intel sidebar (tabs become: Our Pools, Their Picks, Matchups, Notes)
- **D-11:** Within the Notes tab, show a second row of sub-tabs — one per our-side picked champion (by champion name)
- **D-12:** Each champion sub-tab shows ALL note types (matchup, power_spike, combo, lesson, synergy, positioning) in collapsible sections grouped by type
- **D-13:** When a pooled champion is picked into our side, the intel sidebar auto-opens and auto-switches to the Notes tab + that champion's sub-tab
- **D-14:** Champions without pool notes still get a sub-tab with an empty state message ("No notes for {champion} — add notes in Champion Pool") — makes it obvious which champs lack notes
- **D-15:** Notes data comes from the existing `ChampionNote` model via a new server fn that fetches notes for specific champions by the current user (or team members' pools)

### Game Plan Autofill (PLAN-01, PLAN-03)
- **D-16:** Game plan name pre-fills as `"<draft name> plan"` but is editable before saving
- **D-17:** Strategy tag (`win_condition_tag`) auto-fills from the most common composition tag in the draft. If all tags appear once, use the first tag. User can change it on the game plan page
- **D-18:** Our/enemy champions pre-fill from draft picks (existing behavior, confirmed working)
- **D-19:** Role assignments from `DraftAction.role` map to per-role strategy fields — e.g., if Jinx has `role="bot"`, pre-fill `bot_strategy` with `"Jinx: "` as a header prompt. User fills in the strategy after the champion name
- **D-20:** All prefilled fields remain editable — prefill is a convenience, not a constraint

### Claude's Discretion
- Searchable dropdown implementation approach (reuse ChampionAutocomplete pattern or build a new OpponentAutocomplete)
- How to determine "most common" tag when there's a tie (first in list is fine)
- Exact collapsible section animation/toggle behavior for note types
- Whether to show champion portraits in the Notes sub-tabs
- Auto-save mechanism before navigating to opponents page (reuse existing save_draft or create a lightweight auto-save)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — DRFT-01 through DRFT-05, PLAN-01, PLAN-03 acceptance criteria
- `.planning/ROADMAP.md` — Phase 9 success criteria (6 items)

### Draft Page Implementation
- `src/pages/draft.rs` — Draft page component, intel sidebar (tabs at line 1973), opponent signals (lines 854-912), save_draft server fn, auto-save pattern
- `src/models/draft.rs` — `Draft`, `DraftAction` structs, `guess_role_from_tags()`, `role_icon_url()`
- `src/components/draft_board.rs` — DraftBoard component, slot rendering, role badges
- `src/components/champion_autocomplete.rs` — Searchable champion dropdown (reference pattern for opponent autocomplete)

### Intel Panel
- `src/pages/draft.rs:1964-2160` — Intel sidebar: tab buttons, Our Pools tab, Their Picks tab with opponent selector, Matchups tab
- `src/pages/draft.rs:898-912` — Intel signals: `intel_open`, `intel_tab`, `selected_opponent_id`, `opponents_list` resource

### Game Plan Implementation
- `src/pages/game_plan.rs` — Game plan page, `create_plan` server fn, prefill Effect (lines 598-646), `get_draft_for_prefill` server fn
- `src/models/game_plan.rs` — `GamePlan` struct (fields: `win_condition_tag`, `top_strategy` through `support_strategy`, `our_champions`, `enemy_champions`)

### Champion Pool Notes
- `src/models/champion.rs` — `ChampionNote` struct, `NOTE_TYPES` constant, `note_type_label()` helper
- `src/pages/champion_pool.rs` — Champion pool page with note rendering patterns
- `src/server/db.rs` — Champion pool DB queries (`get_champion_pool`, note CRUD functions)

### Opponent Data
- `src/models/opponent.rs` — `Opponent`, `OpponentPlayer`, `OpponentPlayerIntel` structs
- `src/server/db.rs:2230-2405` — Opponent CRUD functions (`create_opponent`, `list_opponents`, `get_opponent`, etc.)
- `src/pages/opponents.rs` — Opponents page (navigation target for "Add New")

### Prior Phase Context
- `.planning/phases/08-champion-pool-role-ui/08-CONTEXT.md` — D-15: DraftAction.role field, D-08: ChampionAutocomplete for matchups

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ChampionAutocomplete` component: Searchable dropdown with type-ahead — reference pattern for building an opponent autocomplete
- `role_icon_url(role)` in `models/draft.rs`: Community Dragon SVG URLs per role — use in Notes tab sub-tabs
- `opponents_list` Resource (already in draft.rs): Loads scouted opponents — feed into the new searchable dropdown
- `opponent_players` Resource (already in draft.rs): Keyed on `selected_opponent_id` — already wired for intel
- `get_draft_for_prefill` server fn: Returns full Draft for game plan prefill — extend to include role data
- `NOTE_TYPES` and `note_type_label()` in champion model: For rendering collapsible note sections by type

### Established Patterns
- Intel sidebar tabs: Array of tab names → button loop → conditional content render. Add "notes" to the array
- Auto-save with debounced timer: `RwSignal<Option<i32>>` + `Closure::once` pattern (draft.rs)
- `Callback::new()` for Copy closures across reactive contexts
- `StoredValue::new()` for champion maps shared across closures
- `collect_view()` for rendering filtered lists

### Integration Points
- Draft header grid (line 1366): Replace 3rd column (Opponent text input) with searchable dropdown + button
- Intel sidebar tab array (line 1973): Add "notes" to `["pools", "their_picks", "matchups"]`
- Intel sidebar opponent `<select>` (lines 2136-2159): Remove, use header dropdown signal
- Game plan prefill Effect (lines 598-646): Extend to populate `plan_name`, `win_condition_tag`, and per-role strategy fields
- `Draft.opponent` field: Currently `Option<String>` with free-text name — needs to store opponent ID (backward-compat with existing string values)

</code_context>

<specifics>
## Specific Ideas

- Searchable opponent dropdown (not plain select) — user prefers type-ahead even for small lists
- Notes tab with nested champion sub-tabs gives a two-level navigation: intel tab → champion → note sections
- Collapsible note sections within each champion's sub-tab — matchups likely expanded by default, others collapsed
- Role-to-strategy mapping in game plan prefill: "Jinx: " as a prompt header in bot_strategy — user fills in the rest
- "Most common tag" heuristic for strategy tag: count occurrences, break ties with position in tags array

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-draft-game-plan*
*Context gathered: 2026-03-23*
