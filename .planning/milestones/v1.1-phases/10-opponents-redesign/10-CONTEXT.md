# Phase 10: Opponents Redesign - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Rebuild the opponents page with a structured 5-role form (one Riot ID per role), auto-fetch champion data from the Riot API on save, OTP detection badges, champion mastery level badges, data recency indicators, and per-player pool analysis (pool size, weighted role distribution, comfort picks). Covers OPP-01 through OPP-04.

</domain>

<decisions>
## Implementation Decisions

### New Opponent Form (OPP-01)
- **D-01:** Replace the current "name + add player one-at-a-time" flow with an inline 5-role form: team name at top, then 5 pre-labeled role rows (Top/Jg/Mid/Bot/Sup) each with a single Riot ID field (`Name#Tag` format)
- **D-02:** Saving creates the opponent + all 5 player slots at once via a single server function
- **D-03:** Partial entry allowed — any combination of filled/empty Riot ID fields is valid. Empty roles still create player slots (to be filled later)
- **D-04:** Single `Name#Tag` input per role (not split fields). Validates on `#` separator
- **D-05:** The detail panel for existing opponents shows the same 5-role layout, editable, with current data pre-filled

### Player Card Design (OPP-02, OPP-03)
- **D-06:** OTP warning badge displayed in the card header next to the role label — orange/yellow `⚠ OTP: {champion}` badge. Triggers when >60% of recent games are on a single champion
- **D-07:** Champion mastery shown as level badges on champion pills: `Ahri M7`, `Zed M5`, etc. Champions sorted by mastery points descending
- **D-08:** Data recency shown as a relative time badge next to the summoner name: "Last fetched: 2d ago". Goes stale-orange after 7+ days. Uses existing `format_timestamp` helper
- **D-09:** Need to add `last_fetched` datetime field to `opponent_player` schema and model

### Pool Analysis (OPP-04)
- **D-10:** Collapsible "Pool Analysis" section below the champion pills on each player card, collapsed by default
- **D-11:** Shows: pool size (X champions), role flexibility (weighted distribution), and comfort picks (top 3 most-played with percentages)
- **D-12:** Role flexibility uses weighted role distribution from match history — shows percentage per role (e.g., "Mid 60% / Top 25% / Sup 15%"). Requires storing per-match role data from Riot API `teamPosition` field
- **D-13:** Need to extend `opponent_player` model/schema to store per-match role data (or derive from enriched match data)

### Auto-Fetch Behavior (OPP-01)
- **D-14:** "Save & Fetch" button saves the opponent and immediately kicks off Riot API fetches for all players with Riot IDs filled in
- **D-15:** Per-player status indicators during fetch: spinner while loading, green check on success, red X with error message on failure. Successfully fetched players keep their data; failed ones show specific error
- **D-16:** Individual player re-fetch via a small refresh icon button on each player card
- **D-17:** Team-level "Refresh All" button on the opponent detail panel header to re-fetch all 5 players at once
- **D-18:** Fetch pulls both recent match champions and mastery data (uses existing `fetch_player_champions` + `fetch_champion_masteries` from `riot.rs`)
- **D-19:** Fetch also needs to pull `teamPosition` per match to compute role distribution for pool analysis

### Claude's Discretion
- Exact card layout dimensions and spacing within the 5-role form
- Animation/transition for collapsible pool analysis section
- How many recent matches to fetch for role distribution (20 is current default for champion fetch)
- Whether to parallelize the 5 player fetches or run sequentially (rate limit consideration)
- Exact stale threshold for data recency coloring (7 days suggested, but tunable)
- Whether `last_fetched` updates on failed fetch attempts or only on success

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — OPP-01 through OPP-04 acceptance criteria
- `.planning/ROADMAP.md` — Phase 10 success criteria (4 items)

### Opponents Implementation (Current)
- `src/pages/opponents.rs` — Current opponents page: OpponentsPage, OpponentDetail, PlayerCard components, server fns (get_opponents, create_opponent, get_opponent_detail, save_opponent, add_player, save_player, remove_player, fetch_champions)
- `src/models/opponent.rs` — `Opponent`, `OpponentPlayer`, `OpponentPlayerIntel` structs (intel struct already has `otp_champion`, `champion_frequencies`, `mastery_data` fields)
- `src/server/db.rs:2222-2400` — Opponent DB functions (CRUD for opponents + players, `update_opponent_player_champions`)
- `schema.surql:173-187` — `opponent` and `opponent_player` table definitions

### Riot API
- `src/server/riot.rs` — `get_puuid()`, `fetch_player_champions()`, `fetch_champion_masteries()`, `fetch_match_history()` — all existing and reusable
- `src/server/riot.rs:133-168` — `fetch_player_champions` iterates match IDs and extracts champion names (extend to also capture `teamPosition`)

### Prior Phase Context
- `.planning/phases/09-draft-game-plan/09-CONTEXT.md` — D-05: Draft.opponent stores record ID (opponent selected from dropdown)
- `.planning/phases/08-champion-pool-role-ui/08-CONTEXT.md` — D-08: ChampionAutocomplete pattern for searchable champion inputs

### Existing Patterns
- `src/components/champion_autocomplete.rs` — Searchable dropdown pattern (reference for any autocomplete needs)
- `src/components/ui.rs` — EmptyState, NoTeamState, SkeletonCard, ToastContext components

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `OpponentPlayerIntel` model: Already has `otp_champion`, `champion_frequencies`, `mastery_data` fields — compute logic may exist or need wiring
- `fetch_champion_masteries()` in riot.rs: Returns `(champion_name, mastery_level, mastery_points)` tuples — directly usable for mastery badges
- `fetch_player_champions()` in riot.rs: Returns champion names from recent matches — needs extension to also return role/position data
- `format_timestamp` helper: Existing relative time formatting — reuse for "Last fetched: Xd ago"
- `ToastContext` + `ToastKind`: Existing toast notification system for save/fetch feedback

### Established Patterns
- Player card layout: `bg-elevated rounded-lg border border-divider/50 p-4` with role badge, form fields, action buttons
- Role labels: `["top", "jungle", "mid", "bot", "support"]` with label mapping already defined in opponents.rs
- Fetch pattern: `spawn_local` + `fetching: RwSignal<bool>` + loading/error state per card
- Champion pill display: `text-xs bg-surface border border-divider/50 text-secondary rounded px-2 py-0.5`

### Integration Points
- Replace `create_opponent` server fn: Currently takes just `name`, needs to accept 5 Riot ID fields
- Replace `add_player` one-at-a-time flow: New form creates all 5 players in a single server call
- Extend `opponent_player` schema: Add `last_fetched`, `mastery_data`, `role_distribution` fields
- Extend `fetch_champions` server fn: Also fetch mastery data and role data in one call
- `OpponentPlayerIntel` computation: Wire up OTP detection and frequency counting (model exists, computation may need implementation)

</code_context>

<specifics>
## Specific Ideas

- Inline 5-role form with pre-labeled rows (Top/Jg/Mid/Bot/Sup) — each row is a single `Name#Tag` input
- OTP badge in card header: orange/yellow `⚠ OTP: {champion}` next to role label
- Champion pills with mastery level suffix: `[Ahri M7]` `[Zed M5]` sorted by mastery points
- Relative time recency badge going stale-orange after 7+ days
- Collapsible pool analysis section with weighted role distribution (e.g., "Mid 60% / Top 25%")
- Per-player fetch status indicators (spinner/check/X) during auto-fetch
- Both per-player refresh buttons AND a team-level "Refresh All"

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-opponents-redesign*
*Context gathered: 2026-03-23*
