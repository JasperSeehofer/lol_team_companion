# Phase 15: Goals & LP History - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Solo players can:
1. View an LP-over-time line graph derived from the existing `ranked_snapshot` table populated by Phase 12 sync.
2. Set personal goals for **rank target**, **CS/min**, and **deaths/game** — replacing the placeholder cards already on the solo dashboard.
3. See goal progress auto-tracked from solo/duo match history (current value vs. target).
4. View champion performance trends (win rate, KDA, CS/min per champion) over a selectable time window.

Requirements covered: RANK-02, LEARN-04, LEARN-05, LEARN-06.

**Carrying forward from earlier phases:**
- [Phase 12 D-04] Mode is persisted in the DB `user` record — solo-mode-only features (this phase's goals UI) gate on `user.mode = 'solo'`.
- [Phase 12 D-07] Solo dashboard uses single-column stacked sections — new LP graph and goals slot into this layout, not a separate page.
- [Phase 12 D-09] Solo/duo queue rank is the only rank tracked — goals + LP graph also use solo/duo only.
- [Phase 12 D-11] Goal placeholder cards (`GoalPlaceholders` in `solo_dashboard.rs:406`) name Rank Target, CS/min, Deaths — replace with live goal cards in this phase.
- [Phase 12 specifics] LP delta per match was a "specific idea" but not implemented — included here as part of the LP graph deliverable.
- [Phase 14 D-08/D-09] Card grid + filter bar pattern — reused for champion trends layout.
- [STATE.md blocker] LP delta boundary logic across tier/division changes (promos, relegation, Master+ no-division) — addressed by D-03 below (cumulative LP scale).

</domain>

<decisions>
## Implementation Decisions

### LP History Graph (RANK-02)
- **D-01: Inline SVG line chart.** Hand-rolled SVG, consistent with the existing `tree_graph.rs` SVG approach. No charting library dependency. Style with semantic tokens (`stroke-accent`, `text-secondary`, etc.) so theme + accent colors apply.
- **D-02: Selectable time window** — `7d / 30d / 90d / all-time` toggle above the graph. Default: **30d**. (Same window pattern reused for champion trends — D-12.)
- **D-03: Cumulative LP scale on the y-axis.** Convert `tier + division + lp` → a single integer "rank score" so tier/division/promo boundaries plot smoothly without discontinuities. Mapping:
  - Tier order (low → high): Iron, Bronze, Silver, Gold, Platinum, Emerald, Diamond, Master, Grandmaster, Challenger.
  - Iron 4 0 LP = 0; each division = 100 LP; tier = 4 × 100 = 400. Diamond 1 99 LP = 2799.
  - Master+ has no division — "rank score" = 2800 + raw `lp` (Master/GM/Challenger share the same continuous scale; Riot returns LP unbounded above 2800).
  - Y-axis labels show tier emblems at boundary values, not raw rank score numbers.
- **D-04: Tooltip on hover** showing exact tier/division/LP and snapshot timestamp. Click does nothing (no detail navigation needed).
- **D-05: Empty state** — "No ranked games synced yet — sync your matches to start tracking." with a Sync button (reuses solo dashboard's existing manual sync action).
- **D-06: Placement on solo dashboard** — new section between the ranked badge and recent matches. Single-column stacked layout from Phase 12 D-07 preserved.

### Goal System (LEARN-04, LEARN-05)
- **D-07: One active goal per type per user.** Each user has at most one `rank_target`, one `cs_per_min`, one `deaths_per_game`. Re-setting overwrites. No multi-goal or goal history in this phase (deferred — see `<deferred>`).
- **D-08: Storage = new `personal_goal` table** with fields `user`, `goal_type` ('rank_target'/'cs_per_min'/'deaths_per_game'), `target_value` (string — see D-09 for encoding), `created_at`, `updated_at`. Composite uniqueness via index on `(user, goal_type)`. Rationale: keeps the user record stable, mirrors the `personal_learning` pattern from Phase 14, and leaves room for a future "goal history" without schema churn.
- **D-09: Target value encoding** — string field for forward compatibility:
  - `rank_target`: `"DIAMOND:IV"` (tier:division; division empty for Master+).
  - `cs_per_min`: `"7.5"` (parsed as f32 in app code).
  - `deaths_per_game`: `"4"` (parsed as i32 in app code).
- **D-10: Goal cards replace `GoalPlaceholders`** at `src/pages/solo_dashboard.rs:406`. Each card shows: icon, label, **target** value, **current** value (D-11 below), progress bar, and an Edit button.
- **D-11: Inline edit affordance.** Clicking Edit on a card swaps to an inline form (tier/division dropdowns for Rank Target; numeric input for CS/min and Deaths). Save/Cancel inline — no modal. Empty state ("Not set") shows a "Set goal" button instead.

### Goal Progress Tracking (LEARN-05)
- **D-12: Progress window = last 20 ranked solo/duo matches.** Fixed window for v1 simplicity. Excludes flex, normals, ARAM, and unranked. Computed server-side from joined `player_match` + `match` rows where `queue_id = 420`.
- **D-13: Recompute on dashboard load**, not on every match sync. Goal progress is a derived view, not a stored value — server function joins goal targets with the user's recent solo/duo `player_match` aggregates and returns a single `GoalProgress` payload.
- **D-14: Progress display per type:**
  - **Rank Target:** "Currently *Gold II 47 LP* → target *Diamond IV 0 LP*. **653 LP to go.**" Progress bar fills proportionally on cumulative LP scale (D-03). If target reached: green "Achieved" badge.
  - **CS/min:** "Avg over last 20 games: **6.8 / 7.5 target**" with progress bar (current / target, capped at 100%). If avg ≥ target: green "On track" badge.
  - **Deaths/game:** "Avg over last 20 games: **5.2 / 4.0 target**" with progress bar (target / current, since lower is better; capped at 100% when current ≤ target). If avg ≤ target: green "On track" badge.
- **D-15: Insufficient data state** — when fewer than 5 solo/duo games are available, show "Need *N* more solo/duo games to track progress" instead of a skewed average. Threshold of 5 chosen so a single bad/good game doesn't dominate the readout.

### Champion Performance Trends (LEARN-06)
- **D-16: Sortable table layout** — columns: Champion (icon + name), Games, Win %, KDA, CS/min, Avg Damage. Sortable by clicking any header (default sort: Games descending). Reuses the stats page table styling.
- **D-17: Selectable time window** — same `7d / 30d / 90d / all-time` toggle as the LP graph (D-02). Default: **30d**. Window is independent of the LP graph's window (each has its own toggle).
- **D-18: Min-games filter** — hide champions with fewer than **3 games** in the selected window by default. Toggle "Show all champions" reveals everyone (useful for reviewing one-off picks).
- **D-19: Queue source = solo/duo + flex** for champion trends (broader than goal tracking — D-12). Rationale: champion trends inform draft/pool decisions and benefit from more data; flex games are still meaningful skill signal. Excludes ARAM, normals, and other non-ranked queues.
- **D-20: Placement = new section on `/stats` page**, below the existing match history list. `/stats` is already the analytics home; this keeps the solo dashboard from becoming overlong (already has ranked badge, LP graph, goals, recent matches). Section header "Champion Trends" with the time-window toggle inline.

### Claude's Discretion
- Exact color scheme for the LP graph line (recommend `--color-accent` for the line, `--color-border` for axis grid).
- Tier emblem image source for y-axis labels (Data Dragon CDN vs. bundled static — same decision as Phase 12 D-08 ranked badge).
- SVG graph dimensions and responsive breakpoints (full-width container on dashboard).
- Hover tooltip styling and placement (reuse any existing tooltip pattern, otherwise minimal `<div>` overlay).
- Whether goal "Set goal" empty state cards still show the existing icon (recommend yes — keeps visual continuity).
- Progress-bar colors for "on-track" vs "behind" (recommend green for on-track, dimmed accent for in-progress, no red).
- Edit form validation (minimum CS/min 0–15 range, deaths 0–20 range, reasonable client-side bounds).
- Whether champion trends time-window toggle persists per-user or resets on each visit (recommend reset — fewer signals).
- KDA formula in the trends table (recommend `(K+A)/max(D, 1)` to handle zero-deaths gracefully).
- Whether to prefetch champion trends on `/stats` initial load or fetch on first user interaction (recommend prefetch under `<Suspense>`, consistent with stats page match history).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — RANK-02, LEARN-04, LEARN-05, LEARN-06 definitions.
- `.planning/ROADMAP.md` — Phase 15 entry: goal, depends-on (Phase 12 only), success criteria 1–4.

### Prior Phase Context (carryover decisions)
- `.planning/phases/12-solo-mode-infrastructure/12-CONTEXT.md` — Solo dashboard layout (D-07), ranked badge (D-08), solo/duo-only rank scope (D-09), `GoalPlaceholders` placement (D-11), region parameterization.
- `.planning/phases/13-match-detail-view/13-CONTEXT.md` — Match data model patterns, `Resource + Suspense` data fetching pattern.
- `.planning/phases/14-personal-learnings-journal/14-CONTEXT.md` — Card grid + filter bar pattern, table-vs-card UX precedent.

### Schema & Data Model
- `schema.surql` lines 35–58 — `match` and `player_match` tables (queue_id, game_duration on match; KDA/CS/win on player_match).
- `schema.surql` lines 249–258 — `ranked_snapshot` table (already populated by Phase 12 on each sync). Powers the LP graph directly; **no new columns needed**.
- `schema.surql` — needs new `DEFINE TABLE personal_goal` (D-08) with `(user, goal_type)` unique index.
- `src/models/match_data.rs:5–28` — `PlayerMatchStats` (no `queue_id`/`game_duration` — both live on the parent `match` row, must be joined for CS/min and queue filtering).
- `src/models/match_data.rs` — needs new `RankedSnapshot`, `PersonalGoal`, `GoalProgress`, `ChampionTrend` model structs.

### Server / DB
- `src/server/db.rs` — All SurrealDB queries; needs new functions:
  - `get_lp_history(user_id, window)` — read `ranked_snapshot` rows for the user, queue_type='solo_duo', sorted by `snapshotted_at`.
  - `get_personal_goals(user_id)` / `upsert_personal_goal(user_id, type, target)` / `delete_personal_goal(user_id, type)`.
  - `compute_goal_progress(user_id)` — joins active goals with last-20 solo/duo `player_match` aggregates and the latest `ranked_snapshot`.
  - `get_champion_trends(user_id, window)` — group `player_match` by champion, filter to solo_duo + flex, aggregate win%/KDA/CS-per-min/damage.
- `src/server/riot.rs` — Existing `fetch_ranked_snapshot()` (Phase 12) is sufficient; **no new Riot endpoints needed**.

### UI Integration Points
- `src/pages/solo_dashboard.rs:244–264` — section ordering for the LP graph and goals; **`GoalPlaceholders` at line 406 is being replaced** (do not delete the file pattern — replace the component body).
- `src/pages/stats.rs` — Add "Champion Trends" section below match history; reuse queue filter dropdown + table styling.
- `src/components/nav.rs` — No new nav links (everything lives on existing solo dashboard + stats routes).
- `src/app.rs` — No new routes (LP graph + goals stay on `/solo/dashboard`; champion trends stays on `/stats`).
- `src/components/stat_card.rs` — Possible reference for goal-card visual pattern (icon + value + label).

### Patterns / Rules (auto-loaded path-specific)
- `.claude/rules/leptos-patterns.md` — Rules 11/12 (server fn DB+auth), 23 (`resource.refetch()` after mutations), 44 (`Ok(Vec::new())` not `Err` for empty data), 26 (stable `<For>` keys for goal cards / trend rows), 19 (`into_any()` for divergent goal-card branches: empty vs set).
- `.claude/rules/wasm-patterns.md` — Rule 35 (no `.unwrap()` in event handlers), 42 (debounced auto-save not strictly needed here — goal edit uses explicit Save button).
- `.claude/rules/surreal-patterns.md` — `type::record('table', $key)` ID handling, `.check()` after writes, schemafull table definitions.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ranked_snapshot` table** (`schema.surql:249`) — already populated by Phase 12 on every sync. LP graph reads it directly; no new sync logic needed.
- **`PlayerMatchStats` + `match` join** (`schema.surql:36–58`, `models/match_data.rs:16`) — provides KDA/CS/win/queue_id/game_duration to compute goal averages and champion trends.
- **`ChampionAutocomplete`** (`src/components/champion_autocomplete.rs`) — not strictly needed here; champion trends shows champions the user has played (no input).
- **`StatCard`** (`src/components/stat_card.rs`) — reference visual for goal cards.
- **`ErrorBanner` / `StatusMessage`** (`src/components/ui.rs`) — for error states on goal mutations.
- **Queue filter dropdown** (`src/pages/stats.rs`) — pattern reference for the `7d/30d/90d/all` time-window toggle.
- **SVG drawing** (`src/components/tree_graph.rs:1–709`) — proves inline SVG with semantic-token styling works in this stack; lift the same approach for the LP graph.

### Established Patterns
- **Server fn + Resource + Suspense** — used everywhere; LP graph data, goal progress, and champion trends each get their own server fn + Resource.
- **`DbStruct → AppStruct` conversion** — `DbRankedSnapshot → RankedSnapshot`, `DbPersonalGoal → PersonalGoal`.
- **`type::record('table', $key)`** — standard SurrealDB ID handling.
- **`Ok(Vec::new())` for empty** (rule 44) — when user has no ranked snapshots / no goals yet.
- **Inline edit (no modal)** — Phase 14 D-11 set the precedent (cards expand inline). Same approach for goal edit.
- **Single-column stacked dashboard** (Phase 12 D-07) — LP graph and goals stack vertically, not side-by-side.

### Integration Points
- **Solo dashboard** (`src/pages/solo_dashboard.rs`): replace `GoalPlaceholders` (line 406) with live `<Goals>` component; add `<LpHistoryGraph>` between ranked badge and recent matches.
- **Stats page** (`src/pages/stats.rs`): add `<ChampionTrends>` section below match history.
- **Schema** (`schema.surql`): add `DEFINE TABLE personal_goal` block.
- **Models** (`src/models/`): add `ranked_snapshot.rs` (or extend `match_data.rs`) with `RankedSnapshot`, `PersonalGoal`, `GoalProgress`, `ChampionTrend`.
- **`src/server/db.rs`**: add the five DB functions listed in canonical_refs.

</code_context>

<specifics>
## Specific Ideas

- LP graph uses a **cumulative rank score** y-axis (D-03) — the only sane way to plot tier/division/promo boundaries on one continuous axis. Master+ continuation handled by adding raw `lp` past 2800.
- **Last-20-ranked-matches** window for goal averages (D-12) gives a stable "recent form" reading without needing a configurable window in v1.
- **Match window mismatch is intentional**: goals use solo/duo only (D-12) because goals are personal solo improvement; champion trends use solo/duo + flex (D-19) because trend data benefits from breadth.
- LP graph and champion trends each have an **independent** `7d/30d/90d/all-time` toggle (D-02, D-17) — they're on different pages anyway.
- "Insufficient data" state (D-15) at < 5 games avoids showing skewed averages when a user has just started syncing.

</specifics>

<deferred>
## Deferred Ideas

- **Goal history / archived goals** — only one active goal per type for now; a future phase can add history-of-goals once users want to see "I hit Diamond in March, now targeting Master."
- **Goal progress notifications** — toast or nav badge when a user hits a goal. Out of scope for v1 — visual achievement on the card is enough.
- **Configurable goal window** — letting the user choose 10/20/50 games per goal. Defer until users ask; the fixed-20 default keeps the v1 UI simple.
- **Per-champion goals** ("CS/min ≥ 8 on Vayne") — bigger UX problem than v1 deserves.
- **Comparing trends to peer averages** (e.g., "your Vayne win rate vs. all Vayne players") — needs aggregated cross-user data we don't have / shouldn't compute (privacy + cost).
- **LP graph annotations for synced sessions** ("Last sync: Mar 27") — overlay markers on the graph. Nice-to-have, defer.
- **Streak tracking** (W3, L2 streaks) — interesting metric but unrelated to goals; could land in a future "ranked deep-dive" phase.

</deferred>

---

*Phase: 15-goals-lp-history*
*Context gathered: 2026-05-05*
