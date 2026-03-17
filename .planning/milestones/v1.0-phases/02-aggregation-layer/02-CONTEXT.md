# Phase 2: Aggregation Layer - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Add cross-table intelligence queries to `db.rs` and enforce champion name normalization at all ingestion points. No UI changes except replacing freeform champion text inputs with validated autocomplete. Queries power Phases 3, 4, and 5.

</domain>

<decisions>
## Implementation Decisions

### Champion Pool Gap Detection
- Gaps are about **champion class/archetype diversity**, not lane coverage
- Use Riot's Data Dragon class tags (Fighter, Mage, Assassin, Tank, Marksman, Support) to assess pool diversity per player per role
- Warn when a player's pool is dominated by a single class — threshold is Claude's discretion
- Cross-reference with opponent scouting data: if an opponent's ban/pick tendencies target a player's dominant class, escalate the warning
- Gap query joins champion_pool entries with opponent_player champion data

### Champion Performance Summary
- Four data sources feed performance: Riot API match stats, post-game review outcomes, draft pick frequency, and game plan results
- Both per-player and team-wide aggregation queries (separate functions)
- Default time window: last 30 days or last 20 games (whichever is more); query accepts optional window parameter for Phase 3/4 UI flexibility
- Win/loss attribution uses the post-game review outcome field (not win condition pass rate)
- Aggregation happens in Rust, not SurrealDB GROUP BY views

### Champion Name Normalization
- **Canonical form:** Data Dragon key (e.g. `"KSante"`, `"AurelionSol"`, `"Jinx"`) — URL-safe, matches Riot API, display name looked up from Data Dragon
- **Migration:** Existing records with display names normalized (Claude decides startup script vs lazy migration)
- **Enforce at ingestion:** All champion input points must use validated autocomplete — no freeform champion name entry. This is part of Phase 2, not deferred to polish
- Applies to: drafts, champion pools, game plans (our_champions/enemy_champions), opponent scouting (recent_champions), tree drafter nodes

### Dashboard Summary Query
- Single batched `get_dashboard_summary(team_id)` returning all panel data in one round-trip
- **Action items:** Total open count + top 3 most recent items with titles
- **Post-game summaries:** Last 5 reviews with patterns identified
- **Pool gap warnings:** Per-player class diversity issues (as defined above)
- **Workflow completeness:** Counts of drafts without game plans, game plans without post-game reviews (leverages Phase 1 FK links)
- Returns `Ok(empty defaults)` when team has no data — never `Err`

### Claude's Discretion
- Exact threshold for single-class dominance warning (e.g. >70% or missing class entirely)
- Migration approach (startup script vs lazy normalization)
- Exact return type shapes for summary structs
- How to batch the dashboard summary sub-queries efficiently
- Integration test structure and coverage depth

</decisions>

<specifics>
## Specific Ideas

- Pool gaps should feel like a coaching insight: "You're exploitable if opponents target your dominant class" — not a generic coverage checklist
- The user explicitly wants champion input validation enforced now, not as future polish — prevent bad data at the source
- Performance data should paint a comprehensive picture: raw stats + team context (drafts, game plans, post-game reviews)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ChampionPoolEntry` model: has `champion: String`, `role`, `tier`, `comfort` — join target for gap analysis
- `ChampionStatSummary` model: has `champion`, `games`, `wins`, `avg_kda` — Riot API stat aggregation already exists
- `PlayerMatchStats` model: per-game data with `champion`, `win` — raw data for performance queries
- `champion_autocomplete.rs` component: text input with champion dropdown — already exists, needs to enforce selection-only
- `champion_picker.rs` component: grid-based selection — already enforces valid names
- `data_dragon.rs`: fetches champion data from CDN — source of canonical names and class tags
- `get_champion_stats_for_user()`: existing per-user champion stats aggregation in db.rs
- `list_open_action_items()`: existing open action items query
- `list_post_game_learnings()`: existing post-game list query
- `get_game_plans_for_draft()`: Phase 1 FK link query — basis for workflow completeness

### Established Patterns
- `Db*` struct with `RecordId` → app struct with `String` ID conversion
- Batched queries via chained `.query("SELECT ...; SELECT ...")` with positional `.take(N)`
- `DbResult<T>` / `DbResult<Vec<T>>` return types
- `unwrap_or_default()` for list queries, `?` for single lookups

### Integration Points
- `src/server/db.rs` — all new query functions added here
- `src/server/data_dragon.rs` — champion class tags for gap analysis
- `src/models/` — new summary structs (DashboardSummary, ChampionPerformance, PoolGapWarning)
- `src/components/champion_autocomplete.rs` — enforce selection-only mode
- All pages with champion text input — enforce autocomplete validation
- `schema.surql` — may need index additions for aggregation query performance

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-aggregation-layer*
*Context gathered: 2026-03-15*
