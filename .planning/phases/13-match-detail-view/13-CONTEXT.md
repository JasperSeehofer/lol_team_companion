# Phase 13: Match Detail View - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can open any match from their history and see the full 10-player scoreboard with items, damage, gold, and vision — plus timeline events — plus a personal performance breakdown. Match detail data is fetched on-demand the first time a user opens a match, then cached in the DB for instant subsequent loads.

Requirements covered: MATCH-01, MATCH-02, MATCH-03, MATCH-04

</domain>

<decisions>
## Implementation Decisions

### Navigation & Layout
- **D-01:** Dedicated route at `/match/:id` — clicking a match row in the stats page or solo dashboard navigates to a full-page match detail view. Back button returns to history.
- **D-02:** Classic two-team table layout — Blue team table on top, Red team table below. Columns: champion icon, summoner name, KDA, items (6 icon slots), damage dealt, gold earned, vision score.
- **D-03:** User's own row is highlighted with a subtle accent background in the scoreboard.
- **D-04:** Item icons use Data Dragon images (small squares), with tooltip on hover showing item name. Consistent with how champion icons are already used.

### Timeline Presentation
- **D-05:** Visual timeline bar — horizontal bar representing game duration with event markers placed at timestamps. Color-coded with both team color (blue/red as background/border) and event type (icon shape/symbol).
- **D-06:** Hover shows tooltip with event details; click expands a detail section below the timeline bar. Both behaviors on the same markers.
- **D-07:** Event types shown: objectives (dragons, baron, herald), tower kills, champion kills (first blood, multikills, aces), inhibitors, ward placements, recalls, and teamfight indicators (4+ participants).
- **D-08:** Filter toggle buttons above the timeline bar to show/hide event categories. Prevents visual overload on long games.
- **D-09:** User's own events get a special highlight indicator (glow, ring, or star) so they can trace their personal game story.

### Performance Breakdown
- **D-10:** Horizontal bar charts — each metric shown as a filled bar with game average as a marker/line. Numbers displayed alongside.
- **D-11:** Metrics included: damage share %, vision score, CS/min, gold earned. KDA excluded (already prominent in scoreboard).
- **D-12:** Comparison defaults to game average (all 10 players), with a toggle to switch to role-based comparison (vs lane opponent) when role data is available from the Riot API.

### Loading & Caching UX
- **D-13:** Skeleton placeholders while fetching — page layout with grey shimmer placeholders for scoreboard, timeline, and performance bars. Content fills in when data arrives.
- **D-14:** No cache indicator — instant load from cache is self-evident. No "fetched X ago" label.
- **D-15:** Error banner + retry button on API failure — uses existing `ErrorBanner` component with clear message and retry action.

### Claude's Discretion
- Whether the performance section includes short text verdicts (e.g. "Above average") alongside the bar charts, or numbers only
- Exact icon shapes/symbols for different timeline event types
- How the special highlight for user's events is styled (glow, ring, star, etc.)
- Teamfight detection heuristic details (4+ participants within time window)
- How role detection works for the role-based comparison toggle (which Riot API field to use, fallback behavior when data is unreliable)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Riot API & Data
- `src/server/riot.rs` — Existing `fetch_match_history()` already calls `match_v5().get_match()` but only extracts current user's participant data. Needs to fetch all 10 participants + timeline via `match_v5().get_timeline()`
- `src/server/data_dragon.rs` — Champion data CDN; will also source item images for the scoreboard
- `src/models/match_data.rs` — Current `PlayerMatchStats` model (KDA, CS, vision, damage) — needs expanding for full match detail with items, gold, all participants

### Data Storage
- `schema.surql` — DB schema; needs match_detail and match_timeline tables for caching full match data
- `src/server/db.rs` — Existing `store_matches()` and `TeamMatchRow` patterns; new functions needed for match detail cache read/write

### UI Integration
- `src/pages/stats.rs` — Stats page with match history list; needs click-to-navigate to `/match/:id`
- `src/pages/solo_dashboard.rs` — Solo dashboard with recent matches; also needs click-to-navigate
- `src/components/ui.rs` — `ErrorBanner` component for error states
- `src/components/stat_card.rs` — Reference pattern for stat display (performance section uses bar charts instead)
- `src/app.rs` — Route definitions; new `/match/:id` route

### Requirements
- `.planning/REQUIREMENTS.md` — MATCH-01 through MATCH-04

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ErrorBanner` component (`src/components/ui.rs`) — for API error states with retry
- `StatCard` component (`src/components/stat_card.rs`) — reference pattern, though performance section uses bar charts
- Data Dragon CDN integration (`src/server/data_dragon.rs`) — already fetches champion data, can extend for item images
- Protected page auth check pattern — all protected pages follow same `get_current_user()` + redirect pattern

### Established Patterns
- **Server fn + Resource + Suspense** — standard data fetching pattern; match detail will use this
- **DbStruct -> AppStruct conversion** — all DB types follow this two-struct pattern
- **`type::record('table', $key)` + strip prefix** — standard SurrealDB ID handling
- **`match_v5().get_match()`** — already used in `riot.rs` for match history; same endpoint provides all 10 participants

### Integration Points
- Stats page match rows need click handler → `<A>` link to `/match/{riot_match_id}`
- Solo dashboard recent matches need same click handler
- New route `/match/:id` in `app.rs`
- New `match_detail.rs` page in `src/pages/`
- New server functions: `fetch_match_detail()` (Riot API + cache), `get_cached_match_detail()`, `store_match_detail()`
- Timeline requires new Riot API call: `match_v5().get_timeline()` in `riot.rs`

</code_context>

<specifics>
## Specific Ideas

- Timeline bar with both team colors and event type icons gives maximum information density
- User's own events highlighted on the timeline lets them trace their personal game narrative
- Filter toggles on timeline are essential given the breadth of event types (objectives, kills, towers, wards, recalls, teamfights)
- Role-based comparison toggle adds depth for players who want to see how they matched up against their lane opponent specifically
- Skeleton loading shows page structure immediately — matches the expectation of a polished app

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 13-match-detail-view*
*Context gathered: 2026-03-25*
