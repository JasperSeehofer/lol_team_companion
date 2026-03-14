# Phase 1: Pipeline CTAs - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire navigational CTAs between draft, game plan, and post-game review pages so users can flow through the prep loop without copy-pasting IDs. Includes URL query param support for draft→game plan prefill, back-reference badges, and duplicate handling.

</domain>

<decisions>
## Implementation Decisions

### CTA Placement
- "Prep for This Draft" button appears on both the draft list (quick action per draft) and the draft detail view (prominent)
- "Review This Game" button appears on both the game plan page (bottom of prep flow) and the linked draft detail view
- Buttons are accent/primary style — prominent and discoverable, not subtle
- Labels are flow-based: "Prep for This Draft", "Review This Game" (not action-based like "Create Game Plan")

### Prefill Behavior (Draft → Game Plan)
- Prefill all available data: champions (our picks + their picks from draft actions), side (blue/red), opponent name, win conditions, and draft notes
- Prefilled champions are locked by default with an "Edit" toggle to override
- Other prefilled fields (opponent, win conditions, notes) are freely editable
- URL pattern: `/game-plan?draft_id=X` — direct navigation and bookmarkable

### Prefill Behavior (Game Plan → Post-Game Review)
- Claude's discretion on what carries over — pick what's most useful for writing a review (opponent, champions as context, win conditions as checkable items)

### Back-References
- Game plan shows source draft as a small info badge — clickable, links to draft detail
- Draft detail shows linked game plans as a badge (e.g., "1 game plan") — clickable
- Post-game review shows badges linking back to BOTH the source game plan AND the source draft
- Badge pattern is consistent across all three pages

### Draft Selection & Duplicates
- Primary flow: "Prep for This Draft" button creates a new game plan linked to that draft
- Also support manual linking: game plan page has a "Link to draft" dropdown for retroactive linking
- When clicking "Prep for This Draft" and a game plan already exists for that draft: show a prompt — "A game plan already exists for this draft. View it or create another?"

### Claude's Discretion
- Exact badge/tag visual design (should use existing semantic tokens from theme system)
- How "Link to draft" dropdown is implemented (autocomplete, modal, inline select)
- Loading/error states for the prefill query
- Whether the "locked with override" champion UI uses a toggle, edit icon, or click-to-unlock pattern

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GamePlan.draft_id: Option<String>` — FK field already exists in model, currently always None
- `PostGameLearning.game_plan_id: Option<String>` and `draft_id: Option<String>` — FK fields exist
- `save_game_plan()` in `db.rs` — already handles game plan persistence
- `DraftAction` model with `phase`, `side`, `champion`, `order` fields — can extract our/their picks
- Draft model has `opponent`, `our_side`, `win_conditions`, `notes` fields — all available for prefill

### Established Patterns
- Server functions use `use_context::<Arc<Surreal<Db>>>()` for DB access
- Auth extraction via `let mut auth: AuthSession = leptos_axum::extract().await?;`
- Navigation uses `window.location().set_href()` for hard navigation (not SPA routing)
- No existing `use_query_map()` usage — URL param reading is new for this codebase

### Integration Points
- `src/pages/draft.rs` — add CTA buttons to draft list and detail view
- `src/pages/game_plan.rs` — add URL param reading, prefill logic, back-reference badge, "Link to draft" dropdown
- `src/pages/post_game.rs` — add URL param reading, back-reference badges to game plan and draft
- `src/server/db.rs` — may need query to fetch draft by ID for prefill, query to check existing game plans for a draft
- `src/app.rs` — no route changes needed (pages already exist)

</code_context>

<specifics>
## Specific Ideas

- The flow should feel like a natural progression: draft your comp → prep the game plan → review after the game
- "Prep for This Draft" naming emphasizes the purpose (preparation) over the action (creating a game plan)
- Badges should be visually lightweight but clearly clickable — info at a glance without cluttering the page

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-pipeline-ctas*
*Context gathered: 2026-03-14*
