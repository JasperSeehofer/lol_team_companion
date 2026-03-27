# Phase 14: Personal Learnings Journal - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can write personal post-game reflections with guided sections, tagged by champion and matchup, browse and filter their history, and start a new learning from a match detail (scoreboard or timeline event) with a single click. Learnings are personal (user-scoped), not team-scoped.

Requirements covered: LEARN-01, LEARN-02, LEARN-03

</domain>

<decisions>
## Implementation Decisions

### Entry Structure
- **D-01:** Learning type selector as first step — user picks **General**, **Champion**, or **Matchup**. This determines which fields appear (General = no champion/opponent, Champion = champion only, Matchup = champion + opponent).
- **D-02:** Guided sections: **What happened**, **What I learned**, **Next time I will...** — all three sections are **required** to save.
- **D-03:** Auto-generated editable title from champion + matchup + date (e.g. "Zed vs Ahri — Mar 27"). User can override.
- **D-04:** Full CRUD — learnings are editable and deletable after creation.
- **D-05:** When created from a match, auto-attach **win/loss result** and **match reference** (riot match ID). Standalone entries skip this.

### Tag System
- **D-06:** Predefined category tags (multi-select): **Laning**, **Teamfighting**, **Macro / Rotations**, **Vision**, **Trading**, **Wave Management**, **Objective Control**, **Mental / Tilt**.
- **D-07:** Users can select **multiple tags** per learning entry.

### Browse & Filter UX
- **D-08:** **Card grid** layout — each card shows title, type badge (General/Champion/Matchup), champion icon(s), tags as chips, win/loss badge, and a preview of the first section.
- **D-09:** **Filter bar** above cards with dropdowns: Type (All/General/Champion/Matchup), Champion, Tag. Filters apply instantly.
- **D-10:** **Sort control** in the filter bar with "Newest first" (default) and "By champion" options.
- **D-11:** Clicking a card **expands inline** to show full content with edit/delete actions. No separate detail page navigation.

### Match-to-Learning Flow
- **D-12:** "Add Learning" button on match detail scoreboard pre-fills: type as Matchup, champion, opponent, win/loss, and match reference. User writes reflection and picks tags.
- **D-13:** "Add Learning" button on **timeline event details** — clicking an event marker's detail panel offers an "Add Learning" CTA that pre-fills all of D-12 **plus** the event timestamp and event name. Event type auto-maps to a tag suggestion (e.g. Dragon kill → Objective Control, Champion kill → Trading/Teamfighting).
- **D-14:** Learning entries store optional **game timestamp** and **event name** when created from timeline events — provides context when reviewing the learning later.

### Navigation
- **D-15:** Top-level **"Learnings"** link in the main nav bar (auth-required, alongside Stats, Draft, etc.).

### Claude's Discretion
- Event-type-to-tag mapping logic (which timeline events map to which predefined tags)
- Card grid responsive breakpoints (2 vs 3 columns)
- Inline expand animation style
- Empty state messaging and CTA
- How the sort-by-champion grouping is visually presented (headers, separators)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Match Detail (Phase 13 — entry points for learnings)
- `src/pages/match_detail.rs` — Existing "Add Learning" CTA at line ~505, links to `/personal-learnings/new?champion={}&opponent={}`. Timeline event markers and detail panel where new "Add Learning" buttons go.

### Existing Models & Patterns
- `src/models/game_plan.rs` — `PostGameLearning` struct (team-scoped) — different entity but reference for field patterns. Personal learnings need a NEW model.
- `src/components/champion_autocomplete.rs` — Reusable champion autocomplete for champion/opponent fields.
- `schema.surql` — DB schema; needs new `personal_learning` table definition.
- `src/server/db.rs` — All DB queries; needs CRUD functions for personal learnings.

### UI Patterns
- `src/pages/stats.rs` — Queue filter dropdown pattern (reuse for Type/Champion/Tag dropdowns)
- `src/pages/draft.rs` — Card-based saved drafts list (reference for card grid layout)
- `src/components/nav.rs` — Nav bar; needs new "Learnings" link added

### Requirements
- `.planning/REQUIREMENTS.md` — LEARN-01, LEARN-02, LEARN-03 definitions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **ChampionAutocomplete** (`src/components/champion_autocomplete.rs`): Text input with champion dropdown — use for champion and opponent fields on the learning form
- **ErrorBanner / StatusMessage** (`src/components/ui.rs`): Existing error and success feedback components
- **Queue filter dropdown** (`src/pages/stats.rs`): Pattern for filter bar dropdowns — reuse for Type/Champion/Tag filters
- **Match detail page** (`src/pages/match_detail.rs`): Already has "Add Learning" link at line ~505 pointing to `/personal-learnings/new` with query params

### Established Patterns
- **Page auth guard**: All protected pages fetch `get_current_user()` and redirect to `/auth/login` if None (client-side `#[cfg(feature = "hydrate")]`)
- **Server fn CRUD**: Create/read/update/delete pattern in `db.rs` with `Db*` structs converting to app-facing models
- **Card layouts**: Drafts list in `draft.rs` uses cards in a grid — consistent pattern to follow
- **Filter bar**: Stats page and match detail timeline both have horizontal filter bars above content

### Integration Points
- **Nav bar** (`src/components/nav.rs`): Add "Learnings" link (auth-required)
- **App router** (`src/app.rs`): Add `/personal-learnings` route
- **Match detail** (`src/pages/match_detail.rs`): Update existing "Add Learning" CTA + add timeline event CTAs
- **Schema** (`schema.surql`): New `personal_learning` table

</code_context>

<specifics>
## Specific Ideas

- Learning type selector (General/Champion/Matchup) as first decision in the creation flow — determines which fields are shown and enables more specific filtering/reuse in future phases
- Timeline event "Add Learning" should attempt to auto-map event types to tag suggestions (e.g. Dragon → Objective Control)
- Cards expand inline rather than navigating to a separate detail page
- Sort by champion should be available alongside the default newest-first ordering

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 14-personal-learnings-journal*
*Context gathered: 2026-03-27*
