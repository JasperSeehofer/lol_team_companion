# Architecture Research: Solo Mode & Match Intelligence

**Domain:** Leptos/SurrealDB competitive gaming app — adding user-scoped solo mode and rich match data to an existing team-scoped system
**Researched:** 2026-03-25
**Confidence:** HIGH — based on direct codebase inspection of all affected files plus riven 2.78.0 API verification

---

## Existing Architecture (Brownfield Baseline)

The app is a Leptos 0.8 single-crate, dual-target (SSR + WASM) app. Current data is almost exclusively team-scoped: users belong to a team, and every major feature queries by `team_id`. The two exceptions are `champion_pool` and `champion_note`, which are already user-scoped (`user ON champion_pool TYPE record<user>`).

**Key structural facts for this milestone:**

- `src/server/db.rs` is ~4,000 lines — all DB queries live here as Rust functions
- `schema.surql` is loaded at startup via `include_str!` — schema changes apply on every restart
- `src/server/riot.rs` has `fetch_match_history()` which pulls only the current user's perspective (one participant out of 10) and stores in `player_match` — no full-match participant data is stored
- `riven 2.78.0` is available; `match_v5().get_timeline()` exists and returns `MatchTimeline` with per-minute frame data and timestamped events
- `league_v4().get_league_entries_by_puuid()` is available — returns tier, division, LP, wins, losses, hot streak
- The `match` table has a `team_id: option<record<team>>` field — this was the original team-scoping handle; solo matches will leave it `None`
- `PostGameLearning` has `team_id` baked into its schema as `TYPE record<team>` — this cannot be made nullable without a schema change

---

## System Overview

```
Browser (WASM)                   Server (SSR)
┌─────────────────────┐          ┌──────────────────────────────────────┐
│  Solo Dashboard     │─server──▶│  get_solo_dashboard()                │
│  Stats / SoloStats  │─fn──────▶│  get_solo_stats() / sync_solo_stats()│
│  Match Detail View  │─────────▶│  get_match_detail(match_id)          │
│  Learnings Journal  │─────────▶│  list_learnings() / save_learning()  │
│  Goals Tracker      │─────────▶│  get_goals() / save_goal()           │
│  Ranked Tracker     │─────────▶│  get_ranked_history()                │
│  Champ Pool (exists)│─────────▶│  get_pool()  [no changes needed]     │
│  Draft Page (exists)│─────────▶│  + get_matchup_notes(champion)       │
└─────────────────────┘          └─────────────────┬────────────────────┘
                                                   │ db.rs
                                                   ▼
                                        SurrealDB (SurrealKV)
                                 ┌─────────────────────────────────┐
                                 │ EXISTING                         │
                                 │  user, team, team_member        │
                                 │  match, player_match            │
                                 │  champion_pool, champion_note   │
                                 │  draft, game_plan, post_game... │
                                 ├─────────────────────────────────┤
                                 │ NEW (this milestone)            │
                                 │  solo_goal                      │
                                 │  ranked_snapshot                │
                                 │  personal_learning              │
                                 │  match_participant              │
                                 │  match_timeline_event           │
                                 │  practice_item                  │
                                 └─────────────────────────────────┘
```

---

## Solo / Team Toggle Design

### Decision: User-Level Flag, Not Route-Level Separation

Store `mode: string DEFAULT 'team'` on the `user` table (values: `'team'` | `'solo'`). The toggle is a persistent preference, not a transient UI state.

**Rationale:**
- Users switch between team prep and personal improvement across sessions
- A URL-based toggle (`/solo/*`) would require duplicating nav structure
- A Leptos context signal would reset on page refresh (WASM state is ephemeral)
- Team features remain available in solo mode — champion pool, draft page, etc. are useful solo
- The flag controls what the dashboard and stats page show as their primary view

**How it integrates with existing auth pattern:**

The `get_current_user()` server fn (in `profile.rs`) already runs on every nav render. Extend `PublicUser` to include `mode: String`. The Nav component already reads this resource — it can branch nav items based on mode without a new fetch.

```rust
// Extend existing PublicUser in src/models/user.rs
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub riot_summoner_name: Option<String>,
    pub mode: String,  // "team" | "solo"  — new field, serde(default)
}
```

**Schema change (additive, IF NOT EXISTS, safe on restart):**
```sql
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'team';
```

**Toggle server function:** A simple `set_user_mode(mode: String)` server fn that updates the field and causes the nav resource to refetch (version signal pattern already used by logout). No page redirect — the user stays where they are.

---

## New Database Tables

### `solo_goal` — Personal Goal Setting

```sql
DEFINE TABLE IF NOT EXISTS solo_goal SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON solo_goal TYPE record<user>;
DEFINE FIELD IF NOT EXISTS goal_type ON solo_goal TYPE string;  -- 'rank', 'cs_per_min', 'deaths_per_game', 'win_rate', 'custom'
DEFINE FIELD IF NOT EXISTS target_value ON solo_goal TYPE option<string>;  -- '{"tier":"Gold","division":"II"}' as JSON string for rank goals
DEFINE FIELD IF NOT EXISTS champion ON solo_goal TYPE option<string>;      -- if goal is champion-specific
DEFINE FIELD IF NOT EXISTS role ON solo_goal TYPE option<string>;
DEFINE FIELD IF NOT EXISTS notes ON solo_goal TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_active ON solo_goal TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON solo_goal TYPE datetime DEFAULT time::now();
```

**Why `target_value` is a string:** Goal types have heterogeneous structure (rank target vs numeric threshold). Storing as JSON string follows the established `mastery_data_json` / `role_distribution_json` pattern in `opponent_player` — avoids schema versioning complexity.

### `ranked_snapshot` — LP History

```sql
DEFINE TABLE IF NOT EXISTS ranked_snapshot SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON ranked_snapshot TYPE record<user>;
DEFINE FIELD IF NOT EXISTS queue_type ON ranked_snapshot TYPE string;  -- 'RANKED_SOLO_5x5'
DEFINE FIELD IF NOT EXISTS tier ON ranked_snapshot TYPE string;         -- 'GOLD', 'PLATINUM', etc.
DEFINE FIELD IF NOT EXISTS division ON ranked_snapshot TYPE string;     -- 'I', 'II', 'III', 'IV'
DEFINE FIELD IF NOT EXISTS lp ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS wins ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS losses ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS recorded_at ON ranked_snapshot TYPE datetime DEFAULT time::now();
```

**No unique index on (user, recorded_at)** — snapshots accumulate over time as a history. Query the most recent by ORDER BY recorded_at DESC LIMIT 1. LP gain/loss per session is computed in Rust by diffing consecutive snapshots.

### `personal_learning` — Solo Learnings Journal

```sql
DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_learning TYPE record<user>;
DEFINE FIELD IF NOT EXISTS match_riot_id ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS champion ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opponent_champion ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS content ON personal_learning TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS tags ON personal_learning TYPE array<string> DEFAULT [];  -- free-form tags
DEFINE FIELD IF NOT EXISTS created_at ON personal_learning TYPE datetime DEFAULT time::now();
```

**Rationale — separate from `post_game_learning`:** The existing `post_game_learning` table is `TYPE record<team>` in the `team` field — it cannot hold solo entries without a schema change that would break existing data. A new `personal_learning` table avoids migration. It is user-scoped (not team-scoped) and has simpler structure (free-text content + tags vs the structured what_went_well/improvements array fields).

### `match_participant` — Full 10-Player Match Data

```sql
DEFINE TABLE IF NOT EXISTS match_participant SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match ON match_participant TYPE record<match>;
DEFINE FIELD IF NOT EXISTS puuid ON match_participant TYPE string;
DEFINE FIELD IF NOT EXISTS summoner_name ON match_participant TYPE option<string>;
DEFINE FIELD IF NOT EXISTS team_id ON match_participant TYPE int;   -- 100 (blue) or 200 (red)
DEFINE FIELD IF NOT EXISTS champion ON match_participant TYPE string;
DEFINE FIELD IF NOT EXISTS team_position ON match_participant TYPE string;
DEFINE FIELD IF NOT EXISTS kills ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS deaths ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS assists ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS cs ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS gold_earned ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS damage_to_champions ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS vision_score ON match_participant TYPE int;
DEFINE FIELD IF NOT EXISTS items_json ON match_participant TYPE option<string>;   -- JSON: [item0..item6]
DEFINE FIELD IF NOT EXISTS win ON match_participant TYPE bool;
DEFINE INDEX IF NOT EXISTS idx_match_participant_match ON match_participant FIELDS match;
```

**Why `items_json` as string:** Item slots are `[i32; 7]` — SurrealDB handles arrays but the existing pattern (e.g., `mastery_data_json`) uses JSON strings for denormalized arrays that don't need querying. This avoids 7 separate int fields in the schema and simplifies deserialization.

### `match_timeline_event` — Key Events Only (Not Raw Timeline)

The riven `get_timeline()` call returns per-minute frame data and all events. Storing raw timeline frames (one per minute, 10 players, 30-minute game = 300 frame entries) is excessive for this use case.

**Store only meaningful events:**

```sql
DEFINE TABLE IF NOT EXISTS match_timeline_event SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match ON match_timeline_event TYPE record<match>;
DEFINE FIELD IF NOT EXISTS timestamp_ms ON match_timeline_event TYPE int;
DEFINE FIELD IF NOT EXISTS event_type ON match_timeline_event TYPE string;
-- 'DRAGON_KILL', 'BARON_NASHOR_KILL', 'TOWER_KILL', 'CHAMPION_KILL', 'INHIBITOR_KILL'
DEFINE FIELD IF NOT EXISTS team_id ON match_timeline_event TYPE option<int>;      -- 100 or 200
DEFINE FIELD IF NOT EXISTS killer_puuid ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS victim_puuid ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS position_json ON match_timeline_event TYPE option<string>;
DEFINE INDEX IF NOT EXISTS idx_timeline_match ON match_timeline_event FIELDS match;
```

**Filter events to:** `DRAGON_KILL`, `BARON_NASHOR_KILL`, `ELITE_MONSTER_KILL`, `TOWER_BUILDING_KILL`, `BUILDING_KILL`, `CHAMPION_KILL`. This gives objective timeline and kill feed without storing 300+ frame entries per game.

### `practice_item` — Practice List

```sql
DEFINE TABLE IF NOT EXISTS practice_item SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON practice_item TYPE record<user>;
DEFINE FIELD IF NOT EXISTS champion ON practice_item TYPE option<string>;
DEFINE FIELD IF NOT EXISTS matchup ON practice_item TYPE option<string>;   -- opponent champion
DEFINE FIELD IF NOT EXISTS skill ON practice_item TYPE string;             -- free text description
DEFINE FIELD IF NOT EXISTS status ON practice_item TYPE string DEFAULT 'active';  -- 'active' | 'completed'
DEFINE FIELD IF NOT EXISTS auto_tracked ON practice_item TYPE bool DEFAULT false;  -- set by system from match history
DEFINE FIELD IF NOT EXISTS games_tracked ON practice_item TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON practice_item TYPE datetime DEFAULT time::now();
```

---

## Existing Tables Modified

### `match` — Add `queue_type` for Solo Queue Filtering

The current `match` table has `queue_id: int` but no filtering mechanism by player. Solo queue is queue_id 420. Add a user reference so solo matches are queryable per user:

```sql
DEFINE FIELD IF NOT EXISTS synced_by ON match TYPE option<record<user>>;
```

This allows `SELECT * FROM match WHERE synced_by = type::record('user', $key) AND queue_id = 420`. The existing `team_id` remains optional — solo syncs set `synced_by` but leave `team_id` as None.

### `player_match` — No Change Needed

The `player_match` table is already user-scoped (`user ON player_match TYPE record<user>`). Solo stats already accumulate here from `sync_team_stats()` (which syncs the calling user's perspective regardless of team mode). No schema change needed — the solo stats page queries `player_match` filtered by `user` directly.

---

## Data Flow for New Features

### Flow 1: Match Detail

```
User clicks match in history list
    │
    ▼
StatsPage (or SoloStatsPage) — calls get_match_detail(match_id)
    │
    ▼ [server fn]
Check if match_participant records exist for this match_id
    │
    ├── YES → return cached match_participant rows + match_timeline_event rows
    │
    └── NO → call riven match_v5().get_match() + get_timeline()
             Extract all 10 participants → INSERT match_participant (10 rows)
             Filter events → INSERT match_timeline_event
             Return assembled MatchDetailView struct
```

**Return struct (lives in `src/models/match_data.rs`):**
```rust
pub struct MatchDetailView {
    pub match_id: String,
    pub game_duration_secs: i32,
    pub game_end_epoch_ms: Option<i64>,
    pub queue_id: i32,
    pub blue_team: Vec<ParticipantSummary>,
    pub red_team: Vec<ParticipantSummary>,
    pub my_puuid: String,
    pub timeline_events: Vec<TimelineEvent>,
}

pub struct ParticipantSummary {
    pub puuid: String,
    pub summoner_name: Option<String>,
    pub champion: String,
    pub position: String,
    pub kills: i32, pub deaths: i32, pub assists: i32,
    pub cs: i32,
    pub gold_earned: i32,
    pub damage_to_champions: i32,
    pub vision_score: i32,
    pub items: Vec<i32>,  // 7 item IDs
    pub win: bool,
    pub is_self: bool,  // computed: puuid == my_puuid
}
```

**Rate limit consideration:** Detail fetching is lazy (on demand, not on sync). The timeline call is a separate Riot API request — only make it if the user explicitly opens a match. Do not eagerly fetch all timelines during `sync_team_stats()` / `sync_solo_stats()`.

### Flow 2: Ranked Tracker

```
User navigates to solo dashboard (or clicks "Sync Ranked")
    │
    ▼
sync_ranked_snapshot() server fn
    │
    ▼
riven league_v4().get_league_entries_by_puuid(puuid, EUW1)
    │
    ▼
Filter for RANKED_SOLO_5x5 entry
INSERT INTO ranked_snapshot { user, tier, division, lp, wins, losses }
    │
    ▼
Return current snapshot + last N snapshots for graph
```

**LP delta computation:** Fetch last 2 snapshots in Rust, compute `lp_delta = current.lp - prev.lp` adjusted for tier/division changes. Division promotions are detected by string comparison of (tier, division) tuples.

### Flow 3: Match → Learning (One-Click)

```
User views match detail
    ├── clicks "Add Learning"
    │       │
    │       ▼
    │   personal_learning form pre-filled with:
    │     champion = participant.champion (for my puuid)
    │     match_riot_id = match_id
    │     role = participant.position
    │
    └── clicks "Quick Note" → minimal modal, just content field
        match_riot_id auto-set
```

This is a navigation pattern (URL params) rather than an API pattern:

```
/learnings?match_id=EUW1_1234&champion=Jinx&role=bot
```

`LearningsPage` reads these params and pre-fills the creation form, same as the existing `draft_id` pre-fill pattern in game plan.

### Flow 4: Matchup Notes Surfaced in Draft

```
DraftPage Intel sidebar
    │
    ▼
When user picks a champion (our_picks signal changes)
    │
    ▼
get_matchup_notes(champion: String) server fn
    │
    ▼
SELECT * FROM champion_note WHERE user = $user AND champion = $champion AND note_type = 'matchup'
SELECT * FROM personal_learning WHERE user = $user AND champion = $champion
    │
    ▼
Return ChampionNotesPanel data (existing champion_note + new personal_learning)
```

This integrates into the existing `DraftPage` Intel sidebar without a new page. The server function adds a `personal_learning` query alongside the existing `champion_note` query.

### Flow 5: Champion Performance Trends

```
SoloStatsPage champion trends section
    │
    ▼
get_champion_trends(user_id, queue_id, limit_games) server fn
    │
    ▼
SELECT champion, count() as games, math::sum(win::int) as wins,
       math::mean((kills + assists) / math::max(deaths, 1)) as avg_kda,
       math::mean(cs / (game_duration / 60.0)) as avg_cs_per_min
FROM player_match WHERE user = type::record('user', $key)
GROUP BY champion
ORDER BY games DESC
    │
    ▼ (aggregate in Rust if SurrealDB GROUP BY unreliable)
Return Vec<ChampionStatSummary>  -- this struct already exists in src/models/champion.rs
```

`ChampionStatSummary` already has the right shape (`champion`, `games`, `wins`, `avg_kda`, `avg_cs_per_min`). No model change needed.

---

## Component Boundaries: New vs Modified

### New Pages

| Page | File | Purpose | Server Fns |
|------|------|---------|-----------|
| `SoloDashboard` | `src/pages/solo_dashboard.rs` | Personal progress hub (mode=solo) | `get_solo_dashboard()` |
| `LearningsPage` | `src/pages/learnings.rs` | Personal learnings journal | `list_learnings()`, `save_learning()`, `delete_learning()` |
| `GoalsPage` | `src/pages/goals.rs` | Goal setting + ranked tracker | `get_goals()`, `save_goal()`, `get_ranked_snapshots()`, `sync_ranked_snapshot()` |

**Match detail is a drawer/modal, not a new route.** The match detail view opens inline from `StatsPage` (or `SoloDashboard`) when a match row is clicked. It does not need its own route — it uses a `RwSignal<Option<String>>` for `selected_match_id` and shows a drawer component. This avoids route complexity and keeps the match in context.

### Modified Pages

| Page | Change | Scope |
|------|--------|-------|
| `StatsPage` | Add match detail drawer; add queue_id filter for solo vs all | Moderate |
| `DraftPage` | Add matchup notes panel in Intel sidebar | Small — new server fn + display component |
| `HomePage` | Mode-aware dashboard section (team vs solo summary) | Small — conditional branch |
| `ProfilePage` | Add mode toggle (team/solo) | Small — one new server fn |
| `Nav` | Mode-aware link set (solo links visible in solo mode) | Small — conditional on `user.mode` |

### New Components

| Component | File | Purpose |
|-----------|------|---------|
| `MatchDetailDrawer` | `src/components/match_detail.rs` | 10-player scoreboard + timeline + performance bar |
| `RankedBadge` | `src/components/ranked_badge.rs` | Tier/division/LP display (reused in solo dashboard, goals page) |
| `LpHistoryGraph` | `src/components/lp_history_graph.rs` | SVG line graph of LP over time (similar to existing `tree_graph.rs` SVG approach) |
| `PerformanceBar` | `src/components/performance_bar.rs` | Damage share / gold share horizontal bars |

### Modified Components

| Component | Change |
|-----------|--------|
| `Nav` | Read `user.mode` from existing user resource; show solo-specific links when mode=solo |

---

## Integration with Existing Cross-Feature Connections

### Matchup Notes → Draft Intel

The `DraftPage` already has an Intel sidebar (`src/pages/draft.rs` contains an opponent intel section). The integration point is adding a new `Resource::new()` for `get_matchup_notes(champion)` that fires when `our_picks` signal changes. It queries both `champion_note` (existing) and `personal_learning` (new) for the selected champion.

**No changes to `champion_note` or `champion_pool` tables.** Personal learnings tagged with a champion surface alongside existing champion notes.

### Champion Performance Trends → Champion Pool

`ChampionStatSummary` already exists in `src/models/champion.rs`. The champion pool page currently shows pool tier but no performance data. Add a computed `win_rate` display to pool entries by joining `player_match` stats at query time. This requires one new `db::` function, not a schema change.

### Match → Personal Learning → Draft Notes Loop

```
Played match → Add learning from match detail
    → Learning tagged with champion X + role
    → Next time drafting champion X
    → Intel sidebar shows "3 learnings for Jinx"
    → Click to expand → see matchup notes
```

This loop works with the data model as described above. The key is that `personal_learning.champion` is indexed by champion name (same normalization as `champion_pool` and `champion_note`), enabling the draft page query.

---

## Build Order (Phase Dependencies)

### Phase 1: Solo Mode Infrastructure
**Deliverable:** Mode toggle works; solo dashboard skeleton; stats page shows solo queue filter.

- Add `mode` field to `user` schema + `PublicUser` model
- Add `set_user_mode()` server fn + toggle in ProfilePage nav
- Create `SoloDashboard` page (initially: ranked snapshot + recent matches panels)
- Modify `StatsPage` to accept `queue_id` filter param; show solo stats when mode=solo
- Add `sync_solo_stats()` server fn (mirrors `sync_team_stats()` but user-scoped, defaults to queue 420)
- Add `ranked_snapshot` table + `sync_ranked_snapshot()` server fn + `RankedBadge` component

**Depends on:** Nothing — all new tables and a user field extension.

### Phase 2: Match Detail View
**Deliverable:** Click a match → see full scoreboard + objectives.

- Add `match_participant` + `match_timeline_event` tables
- Add `get_match_detail(match_id)` server fn (lazy-fetch on demand, cached after first load)
- Build `MatchDetailDrawer` component (scoreboard table + timeline event list)
- Add `PerformanceBar` component for damage/gold share visualization
- Wire drawer to `StatsPage` match rows

**Depends on:** Phase 1 (needs `match` records to exist from solo sync).

### Phase 3: Learnings Journal
**Deliverable:** Add learnings from match detail; browse/tag/filter learnings.

- Add `personal_learning` table
- Create `LearningsPage` with list + creation form + tag filter
- Add URL param pre-fill from match detail (`/learnings?match_id=...&champion=...`)
- Add "Add Learning" button to `MatchDetailDrawer`

**Depends on:** Phase 2 (match detail provides the entry point for one-click learning).

### Phase 4: Goals + Practice Tracking
**Deliverable:** Set rank goal; track LP graph; practice list with auto-progress.

- Add `solo_goal` + `practice_item` tables
- Create `GoalsPage` with goal form + `LpHistoryGraph` component
- Add auto-tracking hook in `sync_solo_stats()` that updates `practice_item.games_tracked` when a champion is played
- Show goal progress summary on `SoloDashboard`

**Depends on:** Phase 1 (needs ranked snapshot data for LP graph).

### Phase 5: Draft Integration
**Deliverable:** Personal matchup notes surface in draft Intel sidebar; champion performance trends visible in champion pool.

- Add `get_matchup_notes(champion)` server fn querying both `champion_note` + `personal_learning`
- Add matchup notes panel to `DraftPage` Intel sidebar (new `Resource::new()` fired on pick)
- Add `get_champion_trends()` server fn for solo performance stats
- Show trends in champion pool cards (win rate badge overlay)

**Depends on:** Phase 3 (personal_learning data must exist) + Phase 1 (solo stats from player_match).

---

## Patterns to Follow

### Pattern: Lazy Match Detail Fetch with Cache

**What:** Check if `match_participant` rows exist before calling Riot API. Insert on first load; return cached on subsequent loads.

**When:** Any on-demand Riot API fetch that is expensive to repeat.

**Example (matches existing `sync_team_stats` incremental sync pattern):**
```rust
// In db.rs
pub async fn has_match_detail(db: &Surreal<Db>, match_id: &str) -> Result<bool, DbError> {
    let mut r = db.query("SELECT count() FROM match_participant WHERE match = type::record('match', $id) GROUP ALL")
        .bind(("id", match_id))
        .await?;
    let count: Option<i64> = r.take("count")?;
    Ok(count.unwrap_or(0) > 0)
}
```

### Pattern: JSON String for Heterogeneous Structures

**What:** Store complex sub-objects (item arrays, goal targets, position data) as JSON strings rather than SurrealDB nested objects.

**When:** Data is read/written as a unit, never queried by individual sub-fields.

**Why:** Matches existing `mastery_data_json` / `role_distribution_json` pattern. Avoids schema versioning. Safe for `serde_json::from_str` deserialization in Rust.

### Pattern: Solo/Team Divergence via Server-Side Branching

**What:** Server functions check `user.mode` (or accept an explicit `queue_id` param) and return appropriate data. The WASM client sends no mode logic — it passes what it has.

**When:** Pages that serve both solo and team users (e.g., StatsPage).

**Example:**
```rust
#[server]
pub async fn get_stats_summary() -> Result<StatsSummary, ServerFnError> {
    let user = /* auth extract */;
    // Mode-aware: solo users get personal stats, team users get team stats
    match user.mode.as_str() {
        "solo" => db::get_solo_stats_summary(&db, &user.id).await,
        _ => db::get_team_stats_summary(&db, &team_id).await,
    }
}
```

### Pattern: Independent Resources per Dashboard Panel (established)

The existing team dashboard pattern applies directly to `SoloDashboard`. Each panel (ranked progress, recent matches, recent learnings, goal status) uses its own `Resource::new()` so failures are isolated.

---

## Anti-Patterns to Avoid

### Anti-Pattern: Storing Full Timeline Frames

**What:** Storing every per-minute participant frame from the timeline API (gold, XP, CS per minute for all 10 players) in the database.

**Why bad:** A 30-minute game generates ~300 frame rows per match. At 100 matches, that is 30,000 rows of frame data that is rarely queried. The display value (showing a gold graph) doesn't justify the storage and indexing overhead for SurrealKV.

**Instead:** Extract only objective events and kill events from the timeline. Render performance breakdowns (damage share, gold share) from the final participant stats already in `match_participant`, not from timeline frames.

### Anti-Pattern: Eager Timeline Fetch on Match Sync

**What:** Calling `get_timeline()` for every match during `sync_solo_stats()`.

**Why bad:** Each timeline call is a separate rate-limited Riot API request. Syncing 20 matches would make 40 requests (match + timeline per game) instead of 21. With a development API key (20 requests/second), this hits limits and slows sync significantly.

**Instead:** Fetch timeline lazily — only when user opens a specific match detail. Check `match_timeline_event` table for existing records before fetching. Cache after first fetch.

### Anti-Pattern: Duplicating Match Sync Logic

**What:** Creating `sync_solo_stats()` as a full copy of `sync_team_stats()` with minor modifications.

**Why bad:** `db.rs` is already ~4,000 lines. Match ingestion logic is non-trivial (incremental sync, dedup by match_id). Duplicating it doubles the maintenance surface.

**Instead:** Extract shared match ingestion to `db::upsert_match_and_player_stat(db, match_data, user_id)` and call it from both `sync_team_stats()` and `sync_solo_stats()`. The difference is only the `team_id` vs `synced_by` field and the queue filter.

### Anti-Pattern: team_id Required on personal_learning

**What:** Adding `team ON personal_learning TYPE record<team>` to scope learnings to a team like `post_game_learning`.

**Why bad:** Solo players without a team cannot create learnings. The feature becomes inaccessible unless you join a team first, which contradicts the solo mode goal.

**Instead:** `personal_learning` is user-scoped only. Team context (if desired) can be added later as an optional reference field. The existing `champion_pool` table demonstrates this pattern — it's user-scoped and works for both solo and team users.

### Anti-Pattern: Mode as Leptos Context Signal

**What:** `provide_context(RwSignal::new("solo"))` at App root, updated by the toggle, read by all pages.

**Why bad:** WASM context signals reset on full page refresh (the hard-nav pattern used by login/logout wipes WASM state). The user's mode preference would be lost on every page load requiring a re-fetch anyway.

**Instead:** Persist mode in the database on the user record. Read it via `get_current_user()` which already runs on every nav render. One source of truth, survives hard navigation.

---

## Schema Change Safety

All new fields and tables use `IF NOT EXISTS` (tables) or `IF NOT EXISTS` (fields). The schema is applied on startup via `include_str!`. Additive changes are safe with this pattern — existing records get default values for new fields.

The one non-additive concern: extending `PublicUser` with `mode: String` requires `#[serde(default)]` so existing cached/serialized `PublicUser` without the field deserializes gracefully. This matches the existing `win_loss` / `rating` pattern on `PostGameLearning` (CLAUDE.md rule for backward compat).

---

## Scalability Considerations

| Concern | Solo Player (100 games) | Team + Solo (500 games) |
|---------|------------------------|------------------------|
| `player_match` full scan | Fine — ~100 rows | Add `DEFINE INDEX idx_player_match_user ON player_match FIELDS user` |
| `match_participant` growth | ~1000 rows per 100 games | Index on `match` field (already in schema above) |
| `ranked_snapshot` | ~50 rows/year | Never large; no index needed |
| `personal_learning` | ~200 rows/year | Index on `user` if full scan is slow |

For the current milestone (single user, dev environment), no indexes are needed beyond the `match_participant.match` index that enables match detail lookup.

---

## Sources

- Direct codebase inspection: `schema.surql`, `src/server/riot.rs`, `src/server/db.rs`, `src/models/`, `src/pages/stats.rs`, `src/pages/champion_pool.rs`, `src/app.rs` — HIGH confidence, ground truth
- riven 2.78.0 source (`~/.cargo/registry/src/.../riven-2.78.0/src/endpoints.rs`, `models.rs`): verified `match_v5().get_timeline()`, `league_v4().get_league_entries_by_puuid()`, `Participant` struct fields (items, gold, damage, position) — HIGH confidence
- CLAUDE.md patterns (SurrealDB 3.x gotchas, Leptos 0.8 signal lifecycle, server fn patterns) — HIGH confidence, project-specific ground truth
- Previous ARCHITECTURE.md (2026-03-14) — patterns for batch queries, independent resources, URL params — HIGH confidence, already validated in codebase

---
*Architecture research for: Solo Mode & Match Intelligence (v1.2 milestone)*
*Researched: 2026-03-25*
