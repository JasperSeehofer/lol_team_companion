# Phase 8: Champion Pool & Role UI - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Overhaul champion pool management with larger card-grid icons, drag-and-drop between tier buckets, and reworked matchup entry using champion search for opponents. Add role assignment to draft picks via clickable role icon badges with auto-guess from champion class. No new features beyond UX-08 and UX-10.

</domain>

<decisions>
## Implementation Decisions

### Champion Pool Layout (UX-08)
- **D-01:** Replace current 28px pill entries with full card grid — each card shows champion portrait, name, comfort stars, meta tag badge, and match stats line
- **D-02:** 3-column grid layout within each tier bucket (cards ~140px wide)
- **D-03:** Selected champion card gets accent border + subtle box-shadow glow (not just border)
- **D-04:** Remove button appears as corner × in top-right on card hover (same pattern as current pills but repositioned)

### Drag-and-Drop Between Tiers (UX-08)
- **D-05:** Entire tier container is a drop zone — drag a champion card and drop it anywhere in the target tier bucket; card appends to end of that tier's grid
- **D-06:** Visual feedback: source card goes semi-transparent (opacity 50%) during drag; target tier bucket gets an accent border highlight on dragover
- **D-07:** No within-tier reordering — drag-and-drop is for tier changes only

### Matchup Entry Rework (UX-08)
- **D-08:** For "matchup" note type: replace the free-text title field with a ChampionAutocomplete dropdown to pick the opponent champion; title auto-fills as "vs {opponent name}"
- **D-09:** Other note types (power_spike, combo, lesson, synergy, positioning) keep the existing free-text title — no changes
- **D-10:** Matchup notes in the notes list display a small opponent champion icon (24px) + name badge next to the "Matchup" label

### Draft Role Assignment (UX-10)
- **D-11:** Each filled pick slot in the DraftBoard gets a small role icon badge (16-20px) in the bottom-right corner, overlaid on the champion portrait
- **D-12:** Clicking the badge opens a popover with 5 role icons (top/jg/mid/bot/sup) as clickable buttons using Community Dragon role icon URLs
- **D-13:** Auto-guess: when a champion is placed in a pick slot, the role is auto-guessed from `Champion.tags` (e.g., Marksman → bot, Assassin → mid, Tank/Fighter → top, Support → sup). Jungle is guessed from known jungler champion lists or multi-tag heuristics
- **D-14:** Visual distinction: auto-guessed role badge shows at ~50% opacity with dashed border; user-confirmed role shows full opacity with solid border
- **D-15:** Add optional `role` field to `DraftAction` model and schema — role assignments persist with saved drafts. Phase 9 will use this for game plan autofill

### Claude's Discretion
- Exact card dimensions and spacing within the 3-column grid
- Champion class → role mapping heuristics (best-effort, doesn't need to be perfect)
- Popover positioning and dismiss behavior
- Drag ghost appearance (browser default vs custom)
- Whether to animate card tier transitions

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — UX-08, UX-10 acceptance criteria
- `.planning/ROADMAP.md` — Phase 8 success criteria (4 items)

### Champion Pool Implementation
- `src/pages/champion_pool.rs` — Current champion pool page (tier buckets, card rendering, note forms, detail panel)
- `src/models/champion.rs` — `ChampionPoolEntry`, `ChampionNote`, `Champion` structs, `NOTE_TYPES`, `note_type_label()`
- `src/components/champion_autocomplete.rs` — ChampionAutocomplete component (reuse for matchup opponent search)
- `src/server/db.rs` — Champion pool DB queries (`get_champion_pool`, `add_to_champion_pool`, `set_champion_tier`, etc.)

### Draft Board Implementation
- `src/components/draft_board.rs` — DraftBoard component, `slot_meta()`, pick/ban slot rendering, existing drag-and-drop handlers
- `src/pages/draft.rs` — Draft page, save_draft server fn, DraftAction serialization
- `src/models/draft.rs` — `DraftAction` struct (needs `role` field addition), `Draft` struct

### Role Icons
- `src/pages/team/dashboard.rs` — `role_icon_url()` helper function (Community Dragon SVG URLs per role, lines 559-568)

### Schema
- `schema.surql` — DB schema (needs `DEFINE FIELD role` on draft_action table)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ChampionAutocomplete` component: Already used for adding champions to pool — reuse for matchup opponent selection
- `role_icon_url(role)` in `dashboard.rs`: Returns Community Dragon SVG URL per role — reuse for draft role badges and popover icons
- `DraftBoard` drag-and-drop handlers: `on:dragstart`, `on:dragover`, `on:drop` pattern — reference for champion pool tier drag-and-drop
- `tier_color()`, `tier_label_color()`, `tier_label()` helpers in champion_pool.rs: Existing tier styling — extend for drop zone highlighting
- `champions_resource`: Already loaded in champion_pool.rs — provides `Champion.tags` for role auto-guess

### Established Patterns
- Semantic tokens (`bg-surface`, `bg-elevated`, `border-divider`, `text-primary`, etc.) — all new card UI must use these
- `StoredValue::new()` for non-reactive data shared across closures (champion maps)
- `Callback::new()` for Copy closures shared across multiple reactive contexts
- `spawn_local` for async server fn calls from event handlers
- `collect_view()` for rendering filtered lists in grids
- `ev.stop_propagation()` for nested click handlers (remove button inside selectable card)

### Integration Points
- `champion_pool.rs` tier rendering: Replace `flex flex-wrap gap-2` pill layout with CSS grid card layout
- `DraftAction` model: Add `role: Option<String>` field — both Rust struct and SurQL schema
- `draft_board.rs` pick slot rendering: Add role badge overlay + popover trigger
- `save_draft` server fn: Already serializes DraftAction as JSON — new `role` field will flow through automatically
- `schema.surql`: Add `DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>`

</code_context>

<specifics>
## Specific Ideas

- Card grid should feel like a champion select screen — portraits front and center, metadata secondary
- Drag-and-drop between tiers should feel snappy — immediate visual feedback on dragover, instant tier update on drop
- Matchup entry with champion search removes the "what type?" question for the most common note type (matchups)
- Role badge dimming for auto-guess makes it visually obvious which roles the user hasn't confirmed yet

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-champion-pool-role-ui*
*Context gathered: 2026-03-22*
