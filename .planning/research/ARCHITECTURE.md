# Architecture Patterns: Cross-Feature Intelligence

**Domain:** Leptos/SurrealDB competitive team management app — adding data flow between independent page features
**Researched:** 2026-03-14
**Overall confidence:** HIGH — based on direct codebase inspection and verified Leptos docs

---

## Current Architecture (Brownfield Baseline)

The app is a Leptos 0.8 single-crate, dual-target (SSR + WASM) app. Every route is an island: each page component calls its own `#[server]` functions, which query SurrealDB directly via `use_context::<Arc<Surreal<Db>>>()`. No data flows between features at the component or query level.

```
Browser (WASM)           Server (SSR)
┌──────────────┐         ┌─────────────────────────────────┐
│  DraftPage   │─server──▶  save_draft() / list_drafts()   │
│  GamePlanPage│─fn────▶  create_plan() / list_plans()     │
│  PostGamePage│─────▶    list_reviews() / save_review()   │
│  StatsPage   │─────▶    get_team_stats()                 │
│  DashboardPage│────▶    get_team_dashboard()             │
└──────────────┘         └──────────────┬────────────────┘
                                        │ db.rs (3,243 lines)
                                        ▼
                              SurrealDB (SurrealKV)
                              ┌────────────────────┐
                              │ draft              │
                              │ draft_action       │
                              │ game_plan          │
                              │ post_game_learning │
                              │ player_match       │
                              │ match              │
                              │ champion_pool      │
                              │ action_item        │
                              │ opponent           │
                              └────────────────────┘
```

**What exists today that enables cross-feature work:**
- `GamePlan.draft_id: Option<String>` — game plan already points to the draft that preceded it
- `PostGameLearning.game_plan_id: Option<String>` — post-game review points to the game plan
- `PostGameLearning.draft_id: Option<String>` — post-game also points directly to draft
- `Draft.series_id: Option<String>` — drafts can be grouped into a series
- `ActionItem.source_review: Option<String>` — action items reference the post-game review they came from
- `db::get_draft_outcome_stats()` — multi-table JOIN query already spans draft + draft_action + post_game_learning + match + player_match
- `db::get_win_condition_stats()` — game_plan + post_game_learning + match correlation query exists

The foreign key linkages are already in the data model. The missing piece is surfacing those links in the UI.

---

## Recommended Architecture for Cross-Feature Intelligence

Cross-feature intelligence in this stack has three distinct layers. Build them in order — each enables the next.

### Layer 1: Deep-Link Navigation (URL Query Params)

**What it is:** Pages accept `?draft_id=`, `?plan_id=`, `?series_id=` query params. When a user clicks "Create game plan from this draft," the draft page navigates to `/game-plan?draft_id=draft:abc123`. The game plan page reads the param and pre-fills from the referenced draft.

**Why first:** Zero new server code. Uses `use_query_map()` (Leptos built-in, HIGH confidence) to read params reactively. The foreign key linkages in the data model already exist (`GamePlan.draft_id`). This gives the draft → game plan → post-game pipeline its navigational skeleton before adding any query complexity.

**Pattern:**
```rust
// In game_plan.rs component:
let query = use_query_map();
let draft_id_param = move || query.read().get("draft_id").map(|s| s.to_string());

// Server fn uses the param to pre-load draft data:
#[server]
pub async fn get_plan_init(draft_id: Option<String>) -> Result<GamePlanInitData, ServerFnError> {
    // If draft_id provided, fetch draft and populate our_champions, enemy_champions
    // from draft actions on our side / enemy side
}
```

**Pages that need this:**
- `/game-plan?draft_id=` — pre-fill from draft
- `/post-game?plan_id=` and `/post-game?draft_id=` — link review to game
- `/draft?series_id=` and `/draft?game=2` — series-aware draft creation
- `/opponents?name=TeamX` — jump to opponent from draft context

### Layer 2: Aggregation Server Functions in db.rs

**What it is:** New server functions that query across multiple tables in a single batched SurrealDB call, returning structured summaries. These are the intelligence computations — champion stats for draft suggestions, recent action items for dashboard surfacing, post-game patterns for next game prep.

**Why second:** Requires no new data model changes. The existing batch query pattern (multiple semicolon-separated statements in one `.query()` call, indexed by position) is established in `get_draft_outcome_stats()` and `get_win_condition_stats()`. New intelligence queries follow the same pattern.

**Key aggregation queries to build:**

```
ChampionPerformanceSummary (stats → draft suggestions)
  SELECT champion, count() as games, math::sum(win) as wins, math::mean(kda) as avg_kda
  FROM player_match WHERE user_id IN [team members]
  GROUP BY champion
  → used by: DraftPage Intel sidebar, BanPriority suggestions

RecentActionItems (post-game → dashboard)
  SELECT * FROM action_item WHERE team = $team AND status != 'done'
  ORDER BY created_at DESC LIMIT 5
  → used by: Dashboard "open items" panel

NextOpponentContext (opponent scouting → draft prep)
  SELECT opponent.*, recent_champions FROM opponent_player
  WHERE opponent IN (drafts for upcoming series)
  → used by: DraftPage Intel sidebar

PostGamePatterns (post-game → next game prep)
  SELECT improvements, action_items FROM post_game_learning
  WHERE team = $team ORDER BY created_at DESC LIMIT 3
  → used by: GamePlanPage "recent lessons" section
```

**Do NOT use SurrealDB precomputed table views for these.** There are multiple open bugs with aggregate views and GROUP BY in SurrealDB 3.x (issues #4881, #2988, #2825). The existing pattern of fetching raw records and aggregating in Rust is more reliable and debuggable. This is consistent with how `get_draft_outcome_stats()` already works.

### Layer 3: Dashboard Intelligence Surface

**What it is:** The team dashboard becomes a smart surface that calls multiple aggregation server functions and presents cross-feature summaries in one place. Not a new page — the existing `/team/dashboard` gets new panels.

**Why third:** Depends on Layer 2 aggregation server functions existing. The dashboard is the integration point — it should surface the most relevant data from each feature without requiring the user to visit each page.

**Dashboard panels (new server functions, independent Resources):**

```
get_dashboard_summary() → DashboardSummary {
    open_action_items: Vec<ActionItem>,     // from action_item table
    recent_reviews: Vec<PostGameLearning>,  // last 3 post-game reviews
    upcoming_opponent: Option<Opponent>,    // if series in progress
    champion_gaps: Vec<ChampionGap>,        // champion pool coverage vs. recent picks
}
```

Each panel is a separate `Resource::new()` so they fail and load independently. A broken stats query should not prevent the roster panel from rendering.

---

## Component Boundaries

| Component | Responsibility | Reads From | Writes To |
|-----------|---------------|------------|-----------|
| `DraftPage` | Draft creation + Intel sidebar | draft, champion_pool, champion stats, opponent data | draft, draft_action |
| `GamePlanPage` | Pre-game strategy | game_plan, draft (via `?draft_id=`), recent post-game patterns | game_plan, checklist_instance |
| `PostGamePage` | Post-game review | post_game_learning, game_plan, draft | post_game_learning, action_item |
| `StatsPage` | Match history + analytics | player_match, match, draft (for analytics) | (read-only) |
| `TeamDashboard` | Cross-feature summary surface | action_item, post_game_learning, opponent, champion_pool | (read-only aggregation) |
| `ActionItemsPage` | Action item management | action_item | action_item |
| `OpponentsPage` | Opponent scouting | opponent, opponent_player | opponent, opponent_player |
| `db.rs` | All SurrealDB queries | (server-side only) | all tables |

**Boundary rule:** Pages do not call each other's server functions. Cross-feature intelligence is expressed as new aggregation functions in `db.rs` that JOIN the relevant tables, returning purpose-built structs. The calling page owns its server function, which calls the db function.

---

## Data Flow Diagram

```
┌─── PREP LOOP ────────────────────────────────────────────────────────┐
│                                                                        │
│   OpponentsPage ──(opponent data)──▶ DraftPage                        │
│         │                               │                             │
│         │                         (draft_id param)                    │
│         │                               ▼                             │
│         │                          GamePlanPage ──(plan_id param)──▶  │
│         │                               │                             │
│         │                         (plan_id param)                     │
│         │                               ▼                             │
│         │                          PostGamePage                       │
│         │                               │                             │
│         │                         (action items                       │
│         │                          + patterns)                        │
│         │                               ▼                             │
│         └──────────────────────▶ TeamDashboard ◀── StatsPage          │
│                                   (surfaced intel)  (champion stats)  │
└────────────────────────────────────────────────────────────────────────┘

Data linkage (foreign keys, already in schema):
  Draft ──draft_id──▶ GamePlan
  GamePlan ──plan_id──▶ PostGameLearning
  PostGameLearning ──draft_id──▶ Draft (direct reference too)
  PostGameLearning ──source_review──▶ ActionItem
  Draft ──series_id──▶ Series ──opponent_id──▶ Opponent
```

---

## Build Order (Phase Dependencies)

### Phase 1: Navigation Wiring (no new server code)
Add `?draft_id=` param to GamePlanPage, `?plan_id=` and `?draft_id=` to PostGamePage. Add "Create game plan" and "Write post-game" CTA buttons on Draft and GamePlan pages respectively. Update the nav to reflect the flow.

Deliverable: User can navigate the draft → game plan → post-game loop without copy-pasting IDs.

Depends on: Nothing (uses existing `use_query_map()`)

### Phase 2: Cross-Feature Aggregation Queries
Add new server functions to `db.rs`:
- `get_champion_performance_summary(team_id, limit)` — for draft Intel
- `get_recent_post_game_patterns(team_id, limit)` — for game plan "recent lessons"
- `get_dashboard_summary(team_id)` — for dashboard intelligence panels

These are Rust-level aggregations over existing tables. No schema changes.

Depends on: Phase 1 (so the foreign key links are populated with real data to query)

### Phase 3: Dashboard Intelligence Surface
Upgrade `/team/dashboard` to show cross-feature panels using Phase 2 server functions. Add champion gap detection (champion pool tiers vs. recent draft picks). Add "Game Day Flow" guided sequence on dashboard.

Depends on: Phase 2 (needs aggregation queries)

### Phase 4: Contextual Suggestions
Surface intelligence inline on feature pages — draft ban suggestions from champion stats, game plan auto-suggestions from post-game patterns, meaningful empty states with guidance on what to do next.

Depends on: Phase 2 and 3 (intelligence queries + dashboard flow established)

---

## Patterns to Follow

### Pattern: Batch Multi-Table Query → Rust Aggregation

**What:** Fetch multiple related tables in one `.query()` call with semicolon-separated statements. Aggregate in Rust, not SurrealQL.

**When:** Any cross-feature intelligence computation.

**Why:** More reliable than SurrealDB precomputed views (multiple open bugs in 3.x with GROUP BY/aggregate views). Rust aggregation is type-safe and testable with `--features ssr` unit tests.

**Example (established in codebase):**
```rust
let mut result = db
    .query("SELECT * FROM draft WHERE team = type::record('team', $key); \
            SELECT * FROM post_game_learning WHERE team = type::record('team', $key); \
            SELECT match_id, id FROM match WHERE team_id = type::record('team', $key)")
    .bind(("key", team_key))
    .await?;
let drafts: Vec<DbDraft> = result.take(0).unwrap_or_default();
let reviews: Vec<DbPostGame> = result.take(1).unwrap_or_default();
let matches: Vec<DbMatchRef> = result.take(2).unwrap_or_default();
// Aggregate in Rust
```

### Pattern: Purpose-Built Return Structs

**What:** New intelligence server functions return dedicated structs (e.g., `DashboardSummary`, `ChampionPerformanceSummary`), not raw model types.

**When:** Any server function that aggregates data from multiple sources.

**Why:** Aggregated data has different shape than raw records. Keeping it separate avoids inflating shared model types with fields only used in one context. These structs live in the page file or a dedicated `src/models/analytics.rs`.

**Example pattern:**
```rust
// In src/models/analytics.rs (new file)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChampionPerformanceSummary {
    pub champion: String,
    pub games: i32,
    pub wins: i32,
    pub avg_kda: f64,
    pub suggested_for: Vec<String>, // "pick" | "ban"
}
```

### Pattern: Independent Resources per Dashboard Panel

**What:** Each dashboard panel uses its own `Resource::new()`, not a single mega-resource.

**When:** Any page that shows multiple independent data sections (especially the dashboard).

**Why:** A failure in one section (e.g., stats unavailable) should not block others from rendering. Leptos `<Suspense>` wraps each panel independently.

```rust
let action_items = Resource::new(|| (), |_| get_open_action_items());
let recent_reviews = Resource::new(|| (), |_| get_recent_reviews());
// Each wrapped in its own <Suspense fallback=|| view!{ <SkeletonPanel/> }>
```

### Pattern: URL as Cross-Feature Context

**What:** Use `?param=value` query params for context that flows from one page to another. Navigation is via `<A href="/game-plan?draft_id={id}">`.

**When:** Linking from one feature to a pre-filled version of another (draft → game plan, game plan → post-game, stats → draft detail).

**Why:** No global state needed. Works with SSR (URL is available on server). URL is bookmarkable and shareable within the team.

```rust
// Navigating to game plan pre-filled from draft:
view! {
    <A href=move || format!("/game-plan?draft_id={}", draft_id.get())>
        "Create game plan"
    </A>
}

// Reading in game plan page:
let query = use_query_map();
let prefill_draft_id = move || query.read().get("draft_id").map(|s| s.to_string());
```

---

## Anti-Patterns to Avoid

### Anti-Pattern: SurrealDB Precomputed Views for Analytics

**What:** `DEFINE TABLE stats_view AS SELECT champion, count() ... FROM player_match GROUP BY champion`

**Why bad:** Multiple open bugs in SurrealDB 3.x with aggregate views using GROUP BY (issues #4881, #2988). Risk of silent data corruption or incorrect counts. The existing codebase already uses Rust-level aggregation for `get_draft_outcome_stats()` — this is the proven pattern.

**Instead:** Fetch raw records, aggregate in Rust. For large datasets, add an index on `team` field to speed SELECT.

### Anti-Pattern: Global Leptos Signal for Cross-Feature State

**What:** Creating a `RwSignal<TeamContext>` at App root with `provide_context()` to share draft/plan/stats state across all pages.

**Why bad:** Leads to signal lifecycle bugs when navigating (stale state from previous page bleeds into next). The codebase already has documented warnings about auto-save timer bugs from stale signal captures (CLAUDE.md rules 54, 55). Adding more global signals increases this surface.

**Instead:** URL params for navigation context. `Resource::new()` per page for fetched data. Global context only for truly global state (auth user, theme — both already done this way).

### Anti-Pattern: Mega Server Function for Dashboard

**What:** One `get_all_dashboard_data()` that returns everything the dashboard needs in one call.

**Why bad:** Any single table query failure errors the entire response. Dashboard shows nothing instead of partial data. Takes longer because sequential processing blocks rendering.

**Instead:** Independent `Resource::new()` per panel, each calling a focused server function. Panels render as each resolves.

### Anti-Pattern: Duplicating Query Logic in Page Server Functions

**What:** Copy-pasting the draft outcome join logic into multiple page files to show similar data in different contexts.

**Why bad:** `db.rs` is already 3,243 lines. Duplicating multi-table JOIN logic doubles the maintenance surface. A schema change (e.g., adding a field to `post_game_learning`) requires updating multiple places.

**Instead:** Add aggregation functions to `db.rs` and call them from multiple page server functions. Server functions in different pages can share the same `db::` function.

---

## Scalability Considerations

| Concern | Now (1 team, ~50 games) | Future (many teams, 1000s of games) |
|---------|------------------------|--------------------------------------|
| Champion stats query | Full scan of `player_match` is fine | Add `DEFINE INDEX idx_player_match_team ON player_match FIELDS team_id` |
| Draft tendencies | Full scan of `draft_action` for team is fine | Index on `draft_action.draft` or use pre-filtered IDs |
| Dashboard aggregation | Multiple small queries in parallel fine | Consider caching computed summaries with TTL in a `team_cache` table |
| Cross-team isolation | All queries already filtered by `team_id` | No change needed, already scoped |

For this milestone, no indexing changes are needed. The data volumes for a single competitive team (5 players, ~50 matches, ~30 drafts) are well within SurrealKV's in-process performance.

---

## Sources

- [Leptos Global State Management (book.leptos.dev)](https://book.leptos.dev/15_global_state.html) — HIGH confidence, official docs
- [Leptos Params and Queries (book.leptos.dev)](https://book.leptos.dev/router/18_params_and_queries.html) — HIGH confidence, official docs
- [SurrealDB precomputed view GROUP BY bugs (GitHub #4881)](https://github.com/surrealdb/surrealdb/issues/4881) — HIGH confidence, open issue tracker
- [SurrealDB aggregate view bugs (GitHub #2988, #2825)](https://github.com/surrealdb/surrealdb/issues/2988) — HIGH confidence, open issue tracker
- Codebase inspection: `src/server/db.rs`, `src/models/`, `src/pages/` — HIGH confidence, ground truth
