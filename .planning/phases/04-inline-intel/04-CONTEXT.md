# Phase 4: Inline Intel - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Surface contextual intelligence on draft planner and game plan pages — pool warnings, opponent tendencies, win condition tracking — without disrupting existing workflows. No new pages or routes. Intel appears inline on existing feature pages.

</domain>

<decisions>
## Implementation Decisions

### Pool Warning Badges (PIPE-03)
- Trigger: Champion not in player's pool AND exposes a class/archetype gap (using Phase 2 Data Dragon class tags)
- Player-to-slot matching: Best-guess role matching using champion's primary role from Data Dragon, with manual override available per slot
- Visual: Small ⚠️ icon badge at top-right of pick slot (mirrors existing "1st" badge pattern at top-left). Hover/click reveals tooltip with details: "Champion not in PlayerName's pool. No [Class] coverage."
- Scope: Our team's pick slots. Claude's discretion on whether to add indicators on opponent picks
- Computation: Client-side reactive — compare already-loaded pool data against current draft picks signal. No server refetch on pick changes

### Opponent Tendency Sidebar (PIPE-04)
- Location: Persistent right sidebar panel on draft page
- Data: Opponent players grouped by role with recent champions + frequency counts
- One-trick detection: Highlight when a player has disproportionate games on one champion (OTP alert)
- Champion mastery: Fetch from Riot API using opponent player's riot_puuid. Show mastery level + points alongside recent pick frequency. This balances recency with long-term champion comfort
- Visibility: Appears when the draft has an opponent set. If no opponent selected, show a prompt to select one
- Refresh: Loads once when opponent is set/changed. Server call only on opponent change, not on pick changes

### Win Condition Tracker (INTL-02)
- Location: Panel on game plan page, below or alongside the win conditions input section
- Pass/fail logic: Uses win_condition_tag + game outcome (win/loss) as proxy. get_win_condition_stats() already implements this
- Display: All historical win condition tags shown with wins/games/percentage. Current game plan's tag highlighted if set
- Opponent filter: Optional toggle — default shows all-time stats, can filter to "vs this opponent" when game plan has an opponent set. Useful for recurring matchups
- Load timing: Fetches on page load, batched with existing game plan data query

### Intel Refresh Behavior
- Pool warnings: Reactive client-side computation (no server calls) — already-loaded team pool data compared against draft picks signal. Updates instantly as picks change
- Opponent sidebar: Server-loaded once per opponent selection. Refetches only when opponent changes
- Win condition tracker: Loads on game plan page load. Static for the session (historical data doesn't change mid-session)
- Principle: No server refetches triggered by reactive draft pick changes. Server calls only for initial load and explicit context changes (opponent switch)

### Claude's Discretion
- Whether opponent-side pick slots get any indicator (e.g. "counters your class gap")
- Exact OTP detection threshold (e.g. >60% of games on one champion)
- Mastery display format (icons, numbers, abbreviated)
- Tooltip styling and animation
- How manual role override UI works on pick slots (dropdown, click-to-assign, drag)
- Empty state for win condition tracker when no data exists
- How to handle opponent players without a linked Riot account (no mastery data available)

</decisions>

<specifics>
## Specific Ideas

- Pool warnings should combine "not in pool" with class gap intel — not just a binary yes/no, but coaching-quality insight ("you're exploitable if opponents target your dominant class" from Phase 2 context)
- Opponent sidebar should surface one-trick-ponies prominently — a player who plays 85% of games on one champion is critical ban intel
- Champion mastery from Riot API prevents recency bias — a player's recent picks might not reflect their deepest champion pocket picks
- Win condition tracker gives historical feedback right where you're making the strategy decision — not buried on a separate analytics page

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DraftBoard` component (`draft_board.rs`): Has "1st" badge pattern at top-left — pool warning badge mirrors this at top-right
- `get_opponent_intel(opponent_id)` server fn: Already called from draft page, returns `Vec<OpponentPlayer>` with `recent_champions` per role
- `get_team_pools()` server fn: Already fetches all starter champion pools on draft page load
- `get_win_condition_stats(db, team_id)`: Returns `(tag, total_games, wins)` — ready for tracker panel
- `compute_pool_gaps_for_team()`: Returns `Vec<PoolGapWarning>` with `dominant_class`, `missing_classes`, `opponent_escalated`
- `data_dragon.rs`: Champion class tags (Fighter, Mage, etc.) — needed for role guessing and class gap analysis
- `OpponentPlayer` model: Has `riot_puuid` field for Riot API mastery lookup, `recent_champions: Vec<String>`
- `slot_meta()` in draft_board.rs: Maps 20 slots to (side, kind, label) — integration point for per-slot warnings

### Established Patterns
- `StoredValue::new()` for large non-reactive data (champion maps) shared across closures
- `Resource::new()` with `Suspense` for async data loading
- Batched SurrealDB queries with positional `.take(N)` for multi-panel data
- `Callback::new()` for Copy-safe closures in iterators
- Semantic tokens for theming: `bg-surface`, `border-divider`, `text-muted`, `bg-accent`

### Integration Points
- `src/pages/draft.rs`: Add opponent sidebar panel, wire pool warning data to DraftBoard
- `src/components/draft_board.rs`: Add warning badge overlay on pick slots
- `src/pages/game_plan.rs`: Add win condition tracker panel
- `src/server/db.rs`: May need new query for opponent-filtered win condition stats, Riot API mastery fetch
- `src/server/riot.rs`: Add champion mastery endpoint call

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-inline-intel*
*Context gathered: 2026-03-16*
