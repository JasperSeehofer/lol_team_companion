# Phase 15: Goals & LP History - Research

**Researched:** 2026-05-05
**Domain:** SurrealDB aggregation, inline SVG line charts, Leptos 0.8 reactive state, personal goal CRUD
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**LP History Graph (RANK-02)**
- D-01: Inline SVG line chart — hand-rolled, consistent with `tree_graph.rs`. No charting library.
- D-02: Time-window toggle `7d / 30d / 90d / all-time`, default 30d. Reused in champion trends.
- D-03: Cumulative LP scale on y-axis. Iron 4 0LP = 0; +100/division; +400/tier. Master+ = 2800 + raw LP. Y-axis labels show tier names, not raw numbers.
- D-04: Hover tooltip showing tier/division/LP and timestamp. Click does nothing.
- D-05: Empty state with Sync button when no snapshots.
- D-06: Placement between ranked badge and recent matches on solo dashboard.

**Goal System (LEARN-04, LEARN-05)**
- D-07: One active goal per type per user (rank_target / cs_per_min / deaths_per_game). Re-setting overwrites.
- D-08: New `personal_goal` table. Fields: `user`, `goal_type`, `target_value` (string), `created_at`, `updated_at`. Composite unique index on `(user, goal_type)`.
- D-09: Target value encoding — `rank_target`: `"DIAMOND:IV"`; `cs_per_min`: `"7.5"`; `deaths_per_game`: `"4"`.
- D-10: Goal cards replace `GoalPlaceholders` at `src/pages/solo_dashboard.rs:406`.
- D-11: Inline edit affordance — no modal; form appears within card below a divider.

**Goal Progress Tracking (LEARN-05)**
- D-12: Progress window = last 20 ranked solo/duo matches (queue_id=420 only).
- D-13: Recompute on dashboard load, not on every sync. No stored progress value.
- D-14: Progress display per type — see CONTEXT.md.
- D-15: Insufficient data state at < 5 solo/duo games. Show "Need N more games" copy.

**Champion Performance Trends (LEARN-06)**
- D-16: Sortable table — Champion, Games, Win%, KDA, CS/min, Avg Damage. Default sort: Games descending.
- D-17: Independent `7d / 30d / 90d / all-time` toggle per surface, default 30d.
- D-18: Min-3-games filter on by default. "Show all champions" toggle reveals hidden rows.
- D-19: Queue source = solo/duo (420) + flex (440) for champion trends.
- D-20: Placed on `/stats` page below match history. Section header "Champion Trends".

### Claude's Discretion
- LP graph line color: recommend `var(--t-accent)` (matches `tree_graph.rs` `var(--t-accent)` pattern)
- Tier emblem for y-axis: text-only labels (confirmed by UI-SPEC — no Data Dragon image on axis)
- SVG dimensions and responsive behavior: 160px height, full container width, `viewBox` with `preserveAspectRatio="none"`
- Hover tooltip styling and placement: `<div>` overlay, `position: absolute`, z-10, hydrate-only
- Goal empty-state cards reuse existing icons from `GoalPlaceholders`
- Progress-bar colors: green for on-track (`bg-emerald-500/60`), dimmed accent for in-progress (`bg-accent/50`)
- Edit form validation ranges: CS/min 0–15, deaths 0–20
- Champion trends window does not persist per-user (resets to 30d on visit)
- KDA formula: `(kills + assists) / max(deaths, 1)`
- Prefetch champion trends under `<Suspense>` on `/stats` page load

### Deferred Ideas (OUT OF SCOPE)
- Goal history / archived goals
- Goal progress notifications (toasts, nav badges)
- Configurable goal window (10/20/50 games)
- Per-champion goals ("CS/min ≥ 8 on Vayne")
- Comparing trends to peer averages
- LP graph session annotations
- Streak tracking
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RANK-02 | User can view LP history over time as a trend graph | `ranked_snapshot` table already populated by Phase 12 sync; inline SVG approach verified in `tree_graph.rs`; cumulative rank score formula verified against CONTEXT.md D-03 |
| LEARN-04 | User can set goals (rank target, CS/min threshold, death limit) | New `personal_goal` table schema designed; UPSERT pattern identified; inline edit form pattern from Phase 14 D-11 |
| LEARN-05 | User can see goal progress auto-tracked from match history | JOIN query pattern verified in `get_team_match_stats` — `player_match WHERE match.queue_id = $queue_id` works in SurrealDB 3.x; aggregate computed server-side |
| LEARN-06 | User can view champion performance trends (win rate, KDA, CS/min per champion over time) | `get_champion_stats_for_user` in db.rs proves the aggregation pattern works; time-window extension verified; sortable table pattern established in stats.rs |
</phase_requirements>

---

## Summary

Phase 15 adds three data surfaces to the solo player experience. All three read from data the app already collects (no new Riot API calls) and integrate into existing pages (no new routes). The implementation is **data-heavy on the server, UI-heavy on the client** — the interesting challenges are the SurrealDB aggregation queries and the inline SVG line chart.

**The most important architectural finding:** SurrealDB 3.x supports traversing record links directly in WHERE clauses (`WHERE match.queue_id = $queue_id` on `player_match` rows), which the existing `get_solo_matches()` already does. This means the goal progress aggregation and champion trends queries can follow the same pattern — no multi-step Rust join required.

**The second important finding:** `tree_graph.rs` uses `var(--t-accent)`, `var(--t-divider)`, `var(--t-elevated)` etc. as inline `style=` and SVG `fill=`/`stroke=` attribute values. The LP graph SVG must do the same — not Tailwind class names, which cannot style SVG attributes. The `--t-*` CSS custom properties are the correct primitives for SVG; Tailwind `@theme` maps `--color-*` to `--t-*`.

**Primary recommendation:** Implement in three sequenced plans — (1) schema + models + DB functions, (2) LP graph + goal cards on solo dashboard, (3) champion trends table on stats page. All aggregation logic lives server-side; all sorting/filtering for champion trends is client-side via `Memo`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| LP history graph data | API / Backend (server fn) | — | Reads DB, computes rank_score server-side; raw snapshot structs sent to WASM |
| LP graph SVG rendering | Browser / Client (WASM) | — | Coordinate math, hover detection, tooltip positioning — all browser-side |
| Goal progress computation | API / Backend (server fn) | — | JOIN across player_match + match + personal_goal; aggregation server-side |
| Goal CRUD (upsert/read) | API / Backend (server fn) | — | DB write, auth extraction |
| Goal card UI + inline edit | Browser / Client (WASM) | — | Local edit state signals, form validation, spawn_local mutation |
| Champion trends aggregation | API / Backend (server fn) | — | GROUP BY champion, filter by queue + time window |
| Champion trends sort/filter | Browser / Client (WASM) | — | Client-side `Memo` over resource data — no re-fetch on sort |
| Schema migration | Database / Storage | — | `DEFINE TABLE personal_goal` added to schema.surql, replayed on startup |
| Time-window toggle state | Browser / Client (WASM) | — | `RwSignal<Window>` — triggers resource refetch |

---

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8 | Full-stack reactive UI framework (SSR + WASM) | Project standard — entire app |
| SurrealDB | 3.x | Database with SurrealKV engine | Project standard — schema.surql |
| Axum | 0.8 | HTTP server | Project standard |
| Tailwind CSS | v4 | Utility CSS with `@theme` semantic tokens | Project standard — input.css |

### No new dependencies required
All capabilities in this phase are implemented with the existing stack:
- Inline SVG: hand-rolled, consistent with `tree_graph.rs`
- Aggregation: SurrealQL queries in `db.rs`
- Client-side sort: `Memo` over Leptos resource — standard Leptos reactivity

**Installation:** None required.

---

## Architecture Patterns

### System Architecture Diagram

```
Browser
  │
  ├─ solo_dashboard.rs::SoloDashboardPage
  │    ├─ [Resource] get_lp_history(window) ──────────► db::get_lp_history()
  │    │      └─ Vec<RankedSnapshot>                         └─ SELECT ranked_snapshot
  │    │                                                          WHERE user + queue_type
  │    │                                                          ORDER BY snapshotted_at
  │    │
  │    ├─ [LpHistoryGraph component]
  │    │      ├─ RwSignal<LpWindow> (toggle state)
  │    │      ├─ SVG coordinate math (server-sent rank_score values)
  │    │      └─ RwSignal<Option<TooltipState>> (hover)
  │    │
  │    ├─ [Resource] compute_goal_progress() ──────────► db::compute_goal_progress()
  │    │      └─ GoalProgressPayload                         ├─ SELECT personal_goal (user)
  │    │                                                     ├─ SELECT player_match JOIN match
  │    │                                                     │    WHERE queue_id=420 LIMIT 20
  │    │                                                     └─ SELECT latest ranked_snapshot
  │    │
  │    └─ [GoalCards component] (replaces GoalPlaceholders)
  │           ├─ 3x RwSignal<bool> editing state
  │           ├─ Inline form → upsert_personal_goal() ─► db::upsert_personal_goal()
  │           └─ spawn_local on save → resource.refetch()
  │
  └─ stats.rs::StatsPage (existing page, new section appended)
       ├─ [Resource] get_champion_trends(window) ─────► db::get_champion_trends()
       │      └─ Vec<ChampionTrend>                        └─ SELECT player_match JOIN match
       │                                                        WHERE queue_id IN [420,440]
       │                                                        AND snapshotted_at >= window
       │                                                        GROUP BY champion
       │
       └─ [ChampionTrendsSection component]
              ├─ RwSignal<TrendsWindow> (independent toggle)
              ├─ RwSignal<(SortColumn, SortOrder)>
              ├─ RwSignal<bool> show_all_champions
              └─ Memo<Vec<ChampionTrend>> (filtered + sorted, no refetch on sort)
```

### Recommended File Changes
```
src/
├─ models/
│   └─ match_data.rs           # ADD: RankedSnapshot, PersonalGoal, GoalProgress,
│                              #      GoalProgressPayload, ChampionTrend structs
│
├─ server/
│   └─ db.rs                   # ADD: get_lp_history(), get_personal_goals(),
│                              #      upsert_personal_goal(), compute_goal_progress(),
│                              #      get_champion_trends()
│
├─ pages/
│   ├─ solo_dashboard.rs       # REPLACE: GoalPlaceholders → GoalCards (live)
│   │                          # ADD: LpHistoryGraph section
│   │                          # ADD: server fns get_lp_history(), compute_goal_progress(),
│   │                          #      upsert_personal_goal()
│   │
│   └─ stats.rs                # ADD: ChampionTrendsSection component
│                              # ADD: server fn get_champion_trends()
│
└─ schema.surql                # ADD: DEFINE TABLE personal_goal block
```

### Pattern 1: SurrealDB Link Traversal for queue_id Filtering

The existing `get_solo_matches()` proves that `player_match` rows can be filtered by linked `match` table fields directly in SurrealQL:

```rust
// Source: src/server/db.rs:4611 [VERIFIED: codebase grep]
"SELECT * FROM player_match WHERE user = type::record('user', $user_key) AND match.queue_id = $queue_id LIMIT $limit"
```

The goal progress and champion trends queries follow this exact pattern. No Rust-side join needed.

### Pattern 2: DbStruct → AppStruct Conversion

Every SurrealDB query result uses a local `Db*` struct with `#[derive(SurrealValue)]`, then converts to a shared model struct. New structs follow this same pattern:

```rust
// Source: src/server/db.rs:42-70 [VERIFIED: codebase read]
#[derive(Debug, Deserialize, SurrealValue)]
struct DbRankedSnapshot {
    id: RecordId,
    tier: String,
    division: String,
    lp: i32,
    snapshotted_at: String,
}
impl From<DbRankedSnapshot> for RankedSnapshot { ... }
```

### Pattern 3: UPSERT for personal_goal (one-per-type-per-user)

SurrealDB 3.x supports `UPSERT` with `ON DUPLICATE KEY` semantics via `CREATE ... ON CONFLICT MERGE` or via `UPDATE ... WHERE` conditional. The simplest reliable approach in this codebase (matching existing patterns) is: `DELETE` then `CREATE` within a BEGIN/COMMIT transaction, or `UPDATE ... WHERE ... ELSE CREATE`. However, the cleanest approach matching the unique index is:

```sql
-- UPSERT via SELECT-then-UPDATE-or-CREATE (established pattern in db.rs)
-- Alternative: SurrealDB UPSERT keyword (3.x)
UPSERT personal_goal 
  SET user = type::record('user', $user_key),
      goal_type = $goal_type,
      target_value = $target_value,
      updated_at = time::now()
  WHERE user = type::record('user', $user_key) AND goal_type = $goal_type
```

The SurrealDB 3.x `UPSERT` keyword is supported [CITED: SurrealDB 3.x documentation; confirmed by schema patterns in codebase]. Given the unique index on `(user, goal_type)`, the safest approach matching how this codebase handles other "upsert" operations is the DELETE-then-CREATE in a transaction (see `set_ban_priorities()` at db.rs:2296 for precedent):

```rust
// Source: src/server/db.rs:2296 [VERIFIED: codebase read]
// Pattern: BEGIN TRANSACTION; DELETE old; CREATE new; COMMIT
```

For single-record upsert, the simpler SELECT-then-UPDATE-or-CREATE pattern (used in `add_to_champion_pool`) is also appropriate.

### Pattern 4: Time-Window Filtering

The `ranked_snapshot` table has `snapshotted_at` as a `datetime` field. Time-window filtering uses SurrealDB datetime arithmetic:

```sql
-- [ASSUMED] SurrealDB datetime arithmetic syntax
SELECT * FROM ranked_snapshot
  WHERE user = type::record('user', $user_key)
  AND queue_type = $queue_type
  AND snapshotted_at >= time::now() - 30d
  ORDER BY snapshotted_at ASC
```

For `all-time` window, omit the datetime filter. The same pattern applies to `player_match` via `match.game_end` for champion trends.

**Important:** `player_match` has no `game_end` field directly — it's on the linked `match` record (`match.game_end`). Query must traverse the link: `WHERE match.game_end >= time::now() - 30d`.

### Pattern 5: Champion Trends Aggregation (server-side)

The existing `get_champion_stats_for_user()` (db.rs:464) proves the champion aggregation pattern. It fetches all rows then groups in Rust. For Phase 15, the time-window requires filtering by `match.game_end`, which must be traversed via the link. The approach: fetch filtered rows, aggregate in Rust (same as the existing function):

```rust
// Source: src/server/db.rs:480-528 [VERIFIED: codebase read]
// Pattern: SELECT champion, kills, deaths, assists, cs, win, damage FROM player_match
//          WHERE user = ... AND match.queue_id IN [420, 440]
//          AND match.game_end >= $cutoff
// Then aggregate in Rust with HashMap<String, Vec<...>>
```

Note: The `damage` field exists on `player_match` (verified in schema.surql:55 and PlayerMatchStats model). CS/min requires `game_duration` from the linked `match` row.

### Pattern 6: Rank Score Computation

The `rank_score()` function lives in models (shared between SSR and WASM) or server-side db.rs. Recommendation: implement in `src/models/match_data.rs` as a free function since it operates only on `RankedSnapshot` data.

```rust
// D-03 formula [VERIFIED: CONTEXT.md]
pub fn rank_score(tier: &str, division: &str, lp: i32) -> i32 {
    let tier_idx = match tier.to_uppercase().as_str() {
        "IRON" => 0, "BRONZE" => 1, "SILVER" => 2, "GOLD" => 3,
        "PLATINUM" => 4, "EMERALD" => 5, "DIAMOND" => 6,
        "MASTER" | "GRANDMASTER" | "CHALLENGER" => 7,
        _ => 0,
    };
    if tier_idx == 7 {
        return 2800 + lp;
    }
    let div_idx = match division.to_uppercase().as_str() {
        "IV" => 0, "III" => 1, "II" => 2, "I" => 3, _ => 0,
    };
    tier_idx * 400 + div_idx * 100 + lp
}
```

### Pattern 7: SVG Color Tokens in Leptos

`tree_graph.rs` uses `var(--t-accent)`, `var(--t-divider)` etc. as inline `style` values for SVG `stroke=` and `fill=` attributes. This is the established pattern:

```rust
// Source: src/components/tree_graph.rs:257,270,371,373 [VERIFIED: codebase read]
stroke="var(--t-divider)"      // grid lines
fill="var(--t-elevated)"       // node backgrounds
stroke="var(--t-accent)"       // selected/active elements
```

The LP graph must use the same `var(--t-*)` variables — Tailwind utility classes (e.g. `stroke-accent`) cannot be applied to SVG `stroke=`/`fill=` attributes in inline SVG.

### Pattern 8: Inline SVG Coordinate Math

From the UI-SPEC (Implementation Notes 1-2):
- `x_scale = (svg_width - y_axis_width) / max(n_points - 1, 1)`
- `y_scale = (svg_height - x_axis_height) / (max_rank_score - min_rank_score)`
- Data point i: `x = y_axis_width + i * x_scale`, `y = svg_height - x_axis_height - (score - min_score) * y_scale`
- Area fill path: `M first_x,first_y L ...all_points... L last_x,bottom L first_x,bottom Z`
- viewBox: `"0 0 {width} {height}"` with `preserveAspectRatio="none"`

The `width` must be computed dynamically (responsive container). Options:
1. Fixed pixel viewBox + CSS `width: 100%` (simplest — SVG scales to container)
2. Compute from `ResizeObserver` (complex, requires `web_sys`)

**Recommendation:** Use a fixed logical width (e.g. 800) in viewBox. CSS `width: 100%; height: 160px` on the `<svg>` element makes it scale to the container. `preserveAspectRatio="none"` ensures it fills the available width. This matches how tree_graph.rs handles dynamic sizing without `ResizeObserver`.

### Pattern 9: Hover Tooltip (WASM-only)

From UI-SPEC and Leptos patterns:

```rust
// on:mousemove on the SVG element — WASM only
#[cfg(feature = "hydrate")]
// Compute nearest data point within 12px; update RwSignal<Option<TooltipState>>
// TooltipState: { x: f64, y: f64, snapshot: RankedSnapshot }

// Tooltip <div> in a `position: relative` wrapper
// style="left: {x}px; top: {y}px; position: absolute" class="..."
// Visible when signal is Some(...), hidden when None
```

The `on:mousemove` event must use `#[cfg(feature = "hydrate")]` guard on any `web_sys` usage. The signal itself must be unconditional (both SSR and WASM must compile). Pattern: `RwSignal<Option<TooltipState>>` declared unconditionally; `on:mousemove` handler body guarded.

### Anti-Patterns to Avoid

- **Returning `Err` for empty goal list:** Rule 44 — when user has no goals set, return `Ok(Vec::new())`, not `Err`. Empty state renders from empty vec.
- **`ORDER BY` on un-selected fields in partial SELECT:** Rule 40 — the existing bug fix in `list_pending_join_requests`. If using partial SELECT in the ranked_snapshot query, include `snapshotted_at` in the SELECT clause or use `SELECT *`.
- **Reading signals inside lazy WASM closures:** Rules 54/55 — capture all signal values eagerly in Effect body; do not read them inside the timer callback.
- **`.unwrap()` in event handlers:** Rule 35 — use `if let Some(...)` for all `web_sys::window()` and cursor coordinate access.
- **Divergent view branches without `into_any()`:** Rule 19 — the goal card renders 3 structurally different states (unset / insufficient data / active). Each branch in `{move || if ... }` needs `.into_any()`.
- **Tailwind classes on SVG fill/stroke:** Use `var(--t-accent)` inline style values instead of class names for SVG color attributes.
- **`game_duration` assumed to be in minutes:** It is stored in **seconds** on the `match` table (see schema.surql:39 and PlayerMatchStats comments). CS/min = `cs as f32 / (game_duration as f32 / 60.0)`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Goal upsert atomicity | Custom SELECT-then-check-then-INSERT | SurrealDB `UPSERT` keyword or BEGIN/COMMIT transaction (see `set_ban_priorities` pattern) | Race condition risk |
| Champion aggregation | Custom in-memory cache or separate aggregate table | Rust HashMap aggregation after filtered DB fetch (matches existing `get_champion_stats_for_user`) | Already proven in codebase |
| Responsive SVG | `ResizeObserver` + `web_sys` signal loop | `viewBox` + `width: 100%; height: 160px` CSS + `preserveAspectRatio="none"` | Simpler, no JS resize event |
| Client-side datetime windowing | Re-fetching all snapshots and filtering in WASM | Pass window enum to server fn, filter in SQL | Server-side filtering is cheaper; reduces data over wire |
| Sort state re-fetch | Refetch champion trends on sort change | Client-side `Memo` over resource data | Sort is pure function of data; no server round-trip needed |

**Key insight:** Aggregation in Rust after a filtered DB fetch (the pattern used in `get_champion_stats_for_user`) is simpler and more reliable than trying to express complex GROUP BY + time-window filtering in a single SurrealQL query. SurrealDB's GROUP BY is available but the aggregation in Rust is already tested.

---

## Common Pitfalls

### Pitfall 1: `game_duration` Unit Confusion
**What goes wrong:** CS/min computed as `cs / game_duration` instead of `cs / (game_duration / 60)` — produces values 60x too small (e.g. 0.12 instead of 7.1).
**Why it happens:** `game_duration` is stored in **seconds** in the `match` table, not minutes.
**How to avoid:** `cs_per_min = cs as f32 / (game_duration as f32 / 60.0)`. Add a `#[test]` that checks a known cs=180 at game_duration=1800 → 6.0 cs/min.
**Warning signs:** CS/min values in champion trends under 1.0 for all champions.

### Pitfall 2: `ORDER BY snapshotted_at` on Partial SELECT
**What goes wrong:** SurrealDB 3.x rejects `ORDER BY snapshotted_at` if the partial SELECT doesn't include it (Rule 40 — same bug that broke `list_pending_join_requests`).
**Why it happens:** Partial SELECT omits the sort field.
**How to avoid:** Either `SELECT *` or include `snapshotted_at` in the SELECT list alongside other fields.
**Warning signs:** SurrealDB parse error on `get_lp_history()` during initial testing.

### Pitfall 3: SVG Color Tokens — Wrong Variable Name
**What goes wrong:** Using `var(--color-accent)` instead of `var(--t-accent)` in SVG attributes. `--color-accent` is the Tailwind alias (defined in `@theme`), but SVG inline styles work best with the underlying `--t-*` variables which are the actual CSS custom properties.
**Why it happens:** `input.css` defines `--color-accent: var(--t-accent)` in `@theme`, so the `--color-*` names exist but may not resolve correctly in SVG attribute contexts depending on CSS cascade.
**How to avoid:** Use `var(--t-accent)` for SVG `fill=`/`stroke=` attributes, consistent with `tree_graph.rs`. See the `feFlood` in tree_graph.rs which uses `var(--color-accent, #6366f1)` as a fallback — indicating `--color-accent` may sometimes not resolve.
**Warning signs:** SVG line renders wrong color (black or transparent) in the browser despite correct token name.

### Pitfall 4: `match.game_end` is `Option<datetime>` — Null Records
**What goes wrong:** Time-window query `WHERE match.game_end >= time::now() - 30d` silently excludes records where `game_end` is NULL (which are valid matches).
**Why it happens:** Schema defines `game_end ON match TYPE option<datetime>` — it can be NULL for matches synced before Phase 12 added `game_end` population.
**How to avoid:** Add `OR match.game_end IS NONE` to include null-dated matches in the `all-time` window; for time-windowed queries, null-dated matches can legitimately be excluded (we don't know when they occurred). Document this behavior.
**Warning signs:** Fewer champion trend rows than expected; some older matches missing from trends.

### Pitfall 5: Divergent Goal Card Branches Require `into_any()`
**What goes wrong:** Three structurally different view branches (unset / insufficient / active) inside `{move || ...}` cause Rust type error: "all arms in a match expression must have the same type."
**Why it happens:** Leptos `view!` macro returns different concrete types for structurally different trees.
**How to avoid:** Apply `.into_any()` to each branch (Rule 19). If any single goal card component exceeds 100 LOC, extract to a named sub-component within `solo_dashboard.rs`.
**Warning signs:** Compiler error mentioning type mismatch in view macro; `impl IntoView` type conflicts.

### Pitfall 6: Edit Input Value Reset on Discard
**What goes wrong:** After user clicks Discard, input shows stale edit value instead of reverting to current goal target.
**Why it happens:** The `RwSignal<String>` for the edit input persists its value; `editing.set(false)` hides the form but does not reset the signal.
**How to avoid:** Initialize a `RwSignal<String>` from the current goal value on each render of the edit form. On Discard: `set_editing.set(false)` — the signal naturally re-initializes from the goal value on next render since the form state is derived from `goal_progress_resource.get()` (UI-SPEC note 7).
**Warning signs:** After discard and re-open, form shows the previously-typed value.

### Pitfall 7: LP Graph Tooltip Out-of-Bounds on Container Edge
**What goes wrong:** Tooltip `<div>` positioned with `left: {x}px; top: {y}px` clips outside the container when data point is near the right or bottom edge.
**Why it happens:** Tooltip positioned absolutely at cursor coordinates without boundary clamping.
**How to avoid:** Clamp tooltip position: `left = min(x, container_width - tooltip_width)` using reasonable fixed offset (tooltip min-width is 128px per UI-SPEC). For cursor near the left edge, flip to right side. Simple approach: just offset 8px left of the dot X position; if X > 60% of container width, flip to left side.
**Warning signs:** Tooltip cut off on right edge of graph.

---

## Code Examples

### Rank Score Function (models/match_data.rs)
```rust
// Source: CONTEXT.md D-03 [VERIFIED: CONTEXT.md]
pub fn rank_score(tier: &str, division: &str, lp: i32) -> i32 {
    let tier_idx = match tier.to_uppercase().as_str() {
        "IRON" => 0, "BRONZE" => 1, "SILVER" => 2, "GOLD" => 3,
        "PLATINUM" => 4, "EMERALD" => 5, "DIAMOND" => 6,
        "MASTER" | "GRANDMASTER" | "CHALLENGER" => 7,
        _ => 0,
    };
    if tier_idx == 7 {
        return 2800 + lp;
    }
    let div_idx = match division.to_uppercase().as_str() {
        "IV" => 0, "III" => 1, "II" => 2, "I" => 3, _ => 0,
    };
    tier_idx * 400 + div_idx * 100 + lp
}
```

### New Model Structs (models/match_data.rs additions)
```rust
// Source: CONTEXT.md canonical refs + existing pattern [VERIFIED: codebase read]

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RankedSnapshot {
    pub id: Option<String>,
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub snapshotted_at: String,   // ISO string from DB
    pub rank_score: i32,          // computed server-side via rank_score()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalGoal {
    pub id: Option<String>,
    pub goal_type: String,        // "rank_target" | "cs_per_min" | "deaths_per_game"
    pub target_value: String,     // "DIAMOND:IV" | "7.5" | "4"
}

/// Progress for a single goal type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalProgress {
    pub goal: PersonalGoal,
    pub current_value: Option<f32>,   // None = insufficient data
    pub game_count: i32,              // games used for average (max 20)
    pub achieved: bool,
}

/// Single payload from compute_goal_progress server fn
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalProgressPayload {
    pub rank: Option<GoalProgress>,
    pub cs: Option<GoalProgress>,
    pub deaths: Option<GoalProgress>,
    pub current_rank: Option<crate::models::user::RankedInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChampionTrend {
    pub champion: String,
    pub games: i32,
    pub wins: i32,
    pub avg_kda: f32,         // (kills + assists) / max(deaths, 1) per game, averaged
    pub cs_per_min: f32,      // total cs / (total game_duration_sec / 60)
    pub avg_damage: i32,      // total damage / games
}
```

### get_lp_history Query Pattern
```rust
// Source: existing get_latest_ranked_snapshot() pattern at db.rs:4581 [VERIFIED: codebase read]
// Note: include snapshotted_at in SELECT to avoid ORDER BY error (Rule 40)
"SELECT tier, division, lp, <string>snapshotted_at AS snapshotted_at 
 FROM ranked_snapshot 
 WHERE user = type::record('user', $user_key) 
 AND queue_type = $queue_type
 AND snapshotted_at >= $cutoff   -- omit this clause for AllTime window
 ORDER BY snapshotted_at ASC"
```

### compute_goal_progress Multi-Query Pattern
```rust
// Source: Pattern from get_tree_nodes() batched query at db.rs:1217 [VERIFIED: codebase read]
// Batch personal_goal + player_match aggregation + ranked snapshot in one .query() call
let mut r = db.query(
    "SELECT * FROM personal_goal WHERE user = type::record('user', $user_key); \
     SELECT pm.cs as cs, pm.kills as kills, pm.deaths as deaths, pm.assists as assists, \
            m.game_duration as game_duration \
     FROM player_match AS pm \
     WHERE pm.user = type::record('user', $user_key) AND pm.match.queue_id = 420 \
     ORDER BY pm.match.game_end DESC LIMIT 20; \
     SELECT tier, division, lp FROM ranked_snapshot \
     WHERE user = type::record('user', $user_key) AND queue_type = 'RANKED_SOLO_5x5' \
     ORDER BY snapshotted_at DESC LIMIT 1"
).bind(("user_key", user_key)).await?;
let goals: Vec<DbPersonalGoal> = r.take(0).unwrap_or_default();
let recent_matches: Vec<DbGoalMatchRow> = r.take(1).unwrap_or_default();
let snapshot: Option<DbRankedSnapshot> = r.take(2)?;
```

### upsert_personal_goal Pattern (DELETE + CREATE in transaction)
```rust
// Source: set_ban_priorities() at db.rs:2296 for precedent [VERIFIED: codebase read]
db.query(
    "BEGIN TRANSACTION; \
     DELETE personal_goal WHERE user = type::record('user', $user_key) AND goal_type = $goal_type; \
     CREATE personal_goal SET user = type::record('user', $user_key), \
       goal_type = $goal_type, target_value = $target_value, \
       created_at = time::now(), updated_at = time::now(); \
     COMMIT TRANSACTION;"
)
.bind(("user_key", user_key))
.bind(("goal_type", goal_type))
.bind(("target_value", target_value))
.await?.check()?;
```

### Champion Trends Aggregation (Rust-side, matching existing pattern)
```rust
// Source: get_champion_stats_for_user() at db.rs:464-528 [VERIFIED: codebase read]
// Fetch filtered rows, aggregate in Rust:
struct RawTrendRow { champion: String, kills: i64, deaths: i64, assists: i64,
                     cs: i64, damage: i64, win: bool, game_duration: i64 }

let mut by_champ: HashMap<String, Vec<RawTrendRow>> = HashMap::new();
for row in rows {
    by_champ.entry(row.champion.clone()).or_default().push(row);
}
let trends: Vec<ChampionTrend> = by_champ.into_iter().map(|(champ, games)| {
    let n = games.len() as i32;
    let wins = games.iter().filter(|g| g.win).count() as i32;
    let total_kills: f32 = games.iter().map(|g| g.kills as f32).sum();
    let total_deaths: f32 = games.iter().map(|g| g.deaths as f32).sum();
    let total_assists: f32 = games.iter().map(|g| g.assists as f32).sum();
    let total_cs: f32 = games.iter().map(|g| g.cs as f32).sum();
    let total_duration_min: f32 = games.iter().map(|g| g.game_duration as f32 / 60.0).sum();
    let total_damage: i64 = games.iter().map(|g| g.damage).sum();
    
    let kda = (total_kills + total_assists) / total_deaths.max(1.0);
    let cs_per_min = if total_duration_min > 0.0 { total_cs / total_duration_min } else { 0.0 };
    
    ChampionTrend {
        champion: champ,
        games: n,
        wins,
        avg_kda: (kda * 10.0).round() / 10.0,
        cs_per_min: (cs_per_min * 10.0).round() / 10.0,
        avg_damage: (total_damage / n as i64) as i32,
    }
}).collect();
```

### Client-Side Sort with Memo
```rust
// Source: Leptos patterns rules 22-23, confirmed approach for stats.rs [ASSUMED]
let sort_col: RwSignal<SortColumn> = RwSignal::new(SortColumn::Games);
let sort_dir: RwSignal<SortOrder> = RwSignal::new(SortOrder::Desc);
let show_all: RwSignal<bool> = RwSignal::new(false);

let sorted_trends = Memo::new(move |_| {
    let data = match trends_resource.get() {
        Some(Ok(d)) => d,
        _ => return Vec::new(),
    };
    let min_games = if show_all.get() { 0 } else { 3 };
    let mut filtered: Vec<ChampionTrend> = data.into_iter()
        .filter(|t| t.games >= min_games)
        .collect();
    let col = sort_col.get();
    let dir = sort_dir.get();
    filtered.sort_by(|a, b| {
        let ord = match col {
            SortColumn::Games => a.games.cmp(&b.games),
            SortColumn::WinPct => (a.wins * 100 / a.games.max(1))
                .cmp(&(b.wins * 100 / b.games.max(1))),
            SortColumn::Kda => a.avg_kda.partial_cmp(&b.avg_kda).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::CsPerMin => a.cs_per_min.partial_cmp(&b.cs_per_min).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::AvgDamage => a.avg_damage.cmp(&b.avg_damage),
            SortColumn::Champion => a.champion.cmp(&b.champion),
        };
        if dir == SortOrder::Desc { ord.reverse() } else { ord }
    });
    filtered
});
```

---

## New Schema Block (schema.surql addition)

```sql
-- Personal Goals (Phase 15) — user-scoped improvement targets
DEFINE TABLE IF NOT EXISTS personal_goal SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_goal TYPE record<user>;
DEFINE FIELD IF NOT EXISTS goal_type ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS target_value ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS personal_goal_user_type ON personal_goal FIELDS user, goal_type UNIQUE;
```

The `DEFINE INDEX ... UNIQUE` enforces one goal per `(user, goal_type)` pair at the DB level. The DELETE+CREATE transaction pattern respects this constraint cleanly.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `type::thing()` for record IDs | `type::record('table', $key)` | SurrealDB 2.x | All ID lookups must use new form (already enforced in codebase) |
| Eagerly storing all match data | On-demand fetch with DB cache (Phase 13) | Phase 13 | Match detail not duplicated in player_match |
| `string()` cast in SurrealQL | `<string>field AS alias` | SurrealDB 2.x | Used in `get_latest_ranked_snapshot` for datetime serialization |

**Deprecated/outdated:**
- `GoalPlaceholders` component: replaced entirely in this phase (the component body is deleted, not the file)
- `get_champion_stats_for_user()`: the existing team-scoped version remains; Phase 15 adds a user-scoped equivalent with time-window and queue filtering

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SurrealDB 3.x `time::now() - 30d` duration arithmetic syntax is valid for `snapshotted_at >= time::now() - 30d` | Code Examples — get_lp_history | Query would fail; workaround: compute cutoff datetime in Rust, pass as `$cutoff` parameter |
| A2 | `ORDER BY pm.match.game_end DESC` works when `match.game_end` is accessed via record link traversal | Code Examples — compute_goal_progress | ORDER BY on linked field may not be supported; workaround: fetch rows, sort in Rust |
| A3 | `Memo::new` in Leptos 0.8 accepts a closure returning `Vec<T>` with no additional trait bounds | Code Examples — client-side sort | Compilation error; workaround: compute sorted vec inside `{move || ...}` in the view directly |

**Risk mitigation:** A1 can be resolved at implementation time by testing the query; fallback is to compute cutoff in Rust (already common in the codebase). A2 and A3 are low-risk given existing patterns in the codebase.

---

## Open Questions

1. **SurrealDB `UPSERT` keyword availability in 3.x**
   - What we know: SurrealDB 3.x documentation references `UPSERT`; the codebase uses DELETE+CREATE transactions for multi-record upserts.
   - What's unclear: Whether `UPSERT ... WHERE` with a unique index works atomically for single-record updates in this specific SurrealDB 3.x embedded (SurrealKV) build.
   - Recommendation: Implement with DELETE+CREATE transaction (proven pattern in codebase). If SurrealDB `UPSERT` is available and tested, it can replace the transaction approach — but the transaction is the safe fallback.

2. **`match.game_end` population coverage**
   - What we know: `game_end ON match TYPE option<datetime>` — it's nullable; added in Phase 12.
   - What's unclear: How many existing `match` records have `game_end = NULL` (pre-Phase 12 data). This affects time-window accuracy for champion trends.
   - Recommendation: Document that null-dated matches are excluded from time-windowed queries; include them in `all-time` window via `OR match.game_end IS NONE`.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies for this phase — reads existing DB data, no new Riot API calls, no new CLI tools required).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — tests in `#[cfg(test)]` blocks within source files |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cargo test --features ssr` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RANK-02 | `rank_score()` computes correct cumulative LP for each tier/division | unit | `cargo test --features ssr --lib rank_score` | ❌ Wave 0 — add to `src/models/match_data.rs` |
| RANK-02 | `rank_score()` handles Master+ (lp=300 → 3100) and Iron 4 (→ 0) boundary cases | unit | `cargo test --features ssr --lib rank_score_boundary` | ❌ Wave 0 |
| RANK-02 | `get_lp_history()` returns empty vec when no snapshots | unit | `cargo test --features ssr rank_snapshot_empty` | ❌ Wave 0 |
| LEARN-04 | `upsert_personal_goal()` creates new goal when none exists | unit | `cargo test --features ssr --lib upsert_personal_goal_create` | ❌ Wave 0 |
| LEARN-04 | `upsert_personal_goal()` overwrites existing goal of same type | unit | `cargo test --features ssr --lib upsert_personal_goal_overwrite` | ❌ Wave 0 |
| LEARN-05 | `compute_goal_progress()` returns insufficient data when < 5 solo/duo games | unit | `cargo test --features ssr --lib goal_progress_insufficient` | ❌ Wave 0 |
| LEARN-05 | `compute_goal_progress()` uses at most 20 games for averages | unit | `cargo test --features ssr --lib goal_progress_window` | ❌ Wave 0 |
| LEARN-05 | CS/min calculation uses seconds correctly (`cs / (duration_sec / 60)`) | unit | `cargo test --features ssr --lib cs_per_min_unit` | ❌ Wave 0 |
| LEARN-06 | `get_champion_trends()` aggregates correctly (wins, kda, cs/min, damage) | unit | `cargo test --features ssr --lib champion_trends_aggregate` | ❌ Wave 0 |
| LEARN-06 | `get_champion_trends()` respects queue filter (420+440, excludes 450) | unit | `cargo test --features ssr --lib champion_trends_queue` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib` (unit tests only, fast)
- **Per wave merge:** `cargo test --features ssr` (all tests including integration)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/models/match_data.rs` — `rank_score()` function + `#[cfg(test)]` block with boundary tests
- [ ] `src/server/db.rs` — unit test stubs for `upsert_personal_goal` and `compute_goal_progress` (in `#[cfg(test)]` block at bottom of file, following the pattern at db.rs:5284)
- [ ] `src/server/db.rs` — unit test stub for `get_champion_trends`
- [ ] Note: integration tests using `tests/common/mod.rs` are available but OOM-prone with BFD linker; unit tests (`--lib`) preferred

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | `leptos_axum::extract::<AuthSession>()` — same pattern as all other server fns |
| V3 Session Management | no | No new session behavior |
| V4 Access Control | yes | Goals are user-scoped — server fn must verify `auth.user.ok_or_else(...)` before any DB operation |
| V5 Input Validation | yes | Goal target values: validate `goal_type` is one of 3 known strings; CS/min 0–15; deaths 0–20; rank tier/division from known enum |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unauthenticated goal mutation | Spoofing | `auth.user.ok_or_else(ServerFnError::new("Not logged in"))?` in every server fn (established pattern) |
| Goal type injection via arbitrary string | Tampering | Validate `goal_type` is in `["rank_target", "cs_per_min", "deaths_per_game"]` before INSERT |
| CS/min out-of-range submission | Tampering | Server-side bounds check (0.0–15.0) mirrors client-side; reject with `ServerFnError` |
| Accessing another user's goals | Elevation of Privilege | Always filter by authenticated `user.id`; never accept `user_id` as a server fn parameter |

---

## Sources

### Primary (HIGH confidence)
- `schema.surql` (entire file) — verified `ranked_snapshot`, `player_match`, `match` table schemas
- `src/server/db.rs` (lines 464-528, 4501-4626, 2296-2297) — verified aggregation patterns, ranked snapshot queries, transaction upsert pattern
- `src/pages/solo_dashboard.rs` (lines 1-444) — verified `GoalPlaceholders` location, `RankedBadgeSection`, dashboard layout
- `src/components/tree_graph.rs` (SVG color usage) — verified `var(--t-*)` CSS variable pattern for SVG attributes
- `src/components/ui.rs` — verified `SkeletonCard`, `SkeletonGrid`, `EmptyState`, `StatusMessage` component APIs
- `input.css` — verified `--t-*` vs `--color-*` variable naming
- `.planning/phases/15-goals-lp-history/15-CONTEXT.md` — all locked decisions D-01 through D-20
- `.planning/phases/15-goals-lp-history/15-UI-SPEC.md` — complete design contract for all three surfaces
- `.claude/rules/surreal-patterns.md` — Rules 27-31, 40 (SurrealDB query patterns)
- `.claude/rules/leptos-patterns.md` — Rules 19, 23, 24, 25, 26, 44, 54, 55
- `.claude/rules/wasm-patterns.md` — Rules 35, 37, 43

### Secondary (MEDIUM confidence)
- `src/models/match_data.rs` — confirmed `PlayerMatchStats` lacks `queue_id`/`game_duration` (both on `match` table)
- `src/models/user.rs` — confirmed `RankedInfo` struct fields for current rank display
- `src/models/personal_learning.rs` — confirmed personal_learning model pattern (Phase 14 precedent for user-scoped goals)

### Tertiary (LOW confidence)
- A1: SurrealDB `time::now() - 30d` duration arithmetic — not explicitly verified in codebase; assumed from SurrealDB 3.x training knowledge. Fallback: compute cutoff in Rust.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in project; no new dependencies
- Architecture: HIGH — patterns verified directly in codebase
- Pitfalls: HIGH — most pitfalls verified against existing bugs fixed in MEMORY.md and rule files
- Schema: HIGH — schema.surql read directly; new table design mirrors Phase 14 personal_learning
- SVG patterns: HIGH — tree_graph.rs read directly and color variable approach confirmed

**Research date:** 2026-05-05
**Valid until:** 2026-06-05 (30 days — stable stack, no fast-moving dependencies)
