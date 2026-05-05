# Phase 15: Goals & LP History - Pattern Map

**Mapped:** 2026-05-05
**Files analyzed:** 10 (5 new, 5 modified)
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/models/match_data.rs` (extend) | model | — | `src/models/personal_learning.rs` | exact — same user-scoped shared struct pattern |
| `schema.surql` (extend) | config | — | `schema.surql` lines 286–302 (`personal_learning` block) | exact — identical `DEFINE TABLE … SCHEMAFULL` + `DEFINE INDEX … UNIQUE` |
| `src/server/db.rs` (extend — 5 fns) | service | CRUD + batch-query | `src/server/db.rs:4563` (`get_latest_ranked_snapshot`), `db.rs:464` (`get_champion_stats_for_user`), `db.rs:2285` (`set_ban_priorities`) | exact per-function |
| `src/pages/solo_dashboard.rs` (modify) | page | request-response | self — modify existing `GoalPlaceholders` + insert new sections | self-analog |
| `src/pages/stats.rs` (modify) | page | request-response | `src/pages/solo_dashboard.rs` (new section appended) | role-match |
| `src/components/lp_history_graph.rs` (inline in solo_dashboard.rs) | component | event-driven (hover) | `src/components/tree_graph.rs` | exact — SVG coordinate math + `var(--t-*)` CSS variables |
| `src/components/goal_cards.rs` (inline in solo_dashboard.rs) | component | CRUD | `src/pages/solo_dashboard.rs:406` (`GoalPlaceholders`) + `src/components/stat_card.rs` | role-match |
| `src/components/champion_trends_table.rs` (inline in stats.rs) | component | request-response + client-sort | `src/pages/stats.rs:474` (`StatsContent`) | exact — same table + filter-signal + `StoredValue` pattern |
| `tests/db_personal_goal.rs` | test | CRUD | `tests/db_champion_pool.rs` | exact |
| `tests/db_lp_history.rs` + `tests/db_champion_trends.rs` | test | CRUD | `tests/db_champion_pool.rs` + `tests/db_drafts.rs` | exact |

---

## Pattern Assignments

---

### `src/models/match_data.rs` — new structs (extend at end of file)

**Analog:** `src/models/personal_learning.rs` lines 15–36

**Struct derivation pattern** (lines 15–16 of personal_learning.rs):
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalLearning {
    pub id: Option<String>,
    pub user_id: String,
    // ... all fields pub, Option where nullable
    pub created_at: Option<String>,  // ISO string from DB datetime
}
```

**Shared struct rules:**
- Must compile for BOTH `ssr` and `hydrate` targets — no `#[cfg(feature = "ssr")]` here
- `use serde::{Deserialize, Serialize};` is already at top of `match_data.rs` (line 1)
- `SurrealValue` derive is `#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]` — see `match_data.rs:15` for the existing `PlayerMatchStats` pattern. However, new Phase 15 model structs (`RankedSnapshot`, `PersonalGoal`, etc.) do NOT need `SurrealValue` on the app-facing struct — only the `Db*` local structs in `db.rs` carry `SurrealValue`. Match the pattern of `PersonalLearning` (no `SurrealValue` on app struct) rather than `PlayerMatchStats` (which has it due to legacy use).

**Concrete structs to add** (adapt from RESEARCH.md code examples):
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RankedSnapshot {
    pub id: Option<String>,
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub snapshotted_at: String,  // ISO datetime string
    pub rank_score: i32,         // computed server-side, not stored in DB
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalGoal {
    pub id: Option<String>,
    pub goal_type: String,    // "rank_target" | "cs_per_min" | "deaths_per_game"
    pub target_value: String, // "DIAMOND:IV" | "7.5" | "4"
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalProgress {
    pub goal: PersonalGoal,
    pub current_value: Option<f32>, // None = insufficient data (< 5 games)
    pub game_count: i32,            // games used for average (max 20)
    pub achieved: bool,
}

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
    pub avg_kda: f32,     // (kills + assists) / max(deaths, 1), per-game avg
    pub cs_per_min: f32,  // total_cs / (total_game_duration_sec / 60.0)
    pub avg_damage: i32,  // total_damage / games
}
```

**`rank_score()` free function** — add alongside structs in `match_data.rs`:
```rust
/// Converts tier/division/lp to a single continuous scale.
/// Iron 4 LP 0 = 0, +100/division, +400/tier. Master+ = 2800 + raw lp.
pub fn rank_score(tier: &str, division: &str, lp: i32) -> i32 {
    let tier_idx = match tier.to_uppercase().as_str() {
        "IRON" => 0, "BRONZE" => 1, "SILVER" => 2, "GOLD" => 3,
        "PLATINUM" => 4, "EMERALD" => 5, "DIAMOND" => 6,
        "MASTER" | "GRANDMASTER" | "CHALLENGER" => 7,
        _ => 0,
    };
    if tier_idx == 7 { return 2800 + lp; }
    let div_idx = match division.to_uppercase().as_str() {
        "IV" => 0, "III" => 1, "II" => 2, "I" => 3, _ => 0,
    };
    tier_idx * 400 + div_idx * 100 + lp
}
```

**Unit test pattern** (from `personal_learning.rs:39–74`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rank_score_gold2_47lp() {
        assert_eq!(rank_score("GOLD", "II", 47), 3 * 400 + 2 * 100 + 47); // 1547
    }
    #[test]
    fn rank_score_master_300lp() {
        assert_eq!(rank_score("MASTER", "", 300), 2800 + 300); // 3100
    }
    #[test]
    fn rank_score_iron4_0lp() {
        assert_eq!(rank_score("IRON", "IV", 0), 0);
    }
}
```

---

### `schema.surql` — new `personal_goal` table block

**Analog:** `schema.surql` lines 286–302 (`personal_learning` block)

**Pattern to copy verbatim, then adapt:**
```sql
-- Personal Learnings (Phase 14) — user-scoped, NOT team-scoped
DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_learning TYPE record<user>;
...
DEFINE INDEX IF NOT EXISTS personal_learning_user ON personal_learning FIELDS user;
```

**New block (Phase 15):**
```sql
-- Personal Goals (Phase 15) — one active goal per type per user
DEFINE TABLE IF NOT EXISTS personal_goal SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_goal TYPE record<user>;
DEFINE FIELD IF NOT EXISTS goal_type ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS target_value ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS personal_goal_user_type ON personal_goal FIELDS user, goal_type UNIQUE;
```

Key differences from `personal_learning`:
- Composite `UNIQUE` index on two fields (`user, goal_type`) — not single-field
- Simpler field set (no `option<…>` fields)
- `updated_at` field (new — `personal_learning` only has `created_at`)

---

### `src/server/db.rs` — five new DB functions

#### `get_lp_history(db, user_id, window)`

**Analog:** `db.rs:4563` (`get_latest_ranked_snapshot`)

**Pattern** (lines 4563–4594):
```rust
pub async fn get_latest_ranked_snapshot(
    db: &Surreal<Db>,
    user_id: &str,
    queue_type: &str,
) -> DbResult<Option<crate::models::user::RankedInfo>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbRankedSnapshot {
        queue_type: String,
        tier: String,
        division: String,
        lp: i32,
        wins: i32,
        losses: i32,
    }

    let mut result = db
        .query("SELECT queue_type, tier, division, lp, wins, losses FROM ranked_snapshot \
                WHERE user = type::record('user', $user_key) AND queue_type = $queue_type \
                ORDER BY snapshotted_at DESC LIMIT 1")
        .bind(("user_key", user_key))
        .bind(("queue_type", queue_type.to_string()))
        .await?;

    let row: Option<DbRankedSnapshot> = result.take(0)?;
    Ok(row.map(|r| crate::models::user::RankedInfo { ... }))
}
```

**`get_lp_history` specific adaptations:**
- Local `Db*` struct needs `tier`, `division`, `lp`, `snapshotted_at` — NOT `wins`/`losses`
- `snapshotted_at` MUST be in SELECT list (Rule 40 — ordering field must be selected)
- Use `SELECT *` or explicitly include `snapshotted_at` in partial SELECT
- Returns `Vec<RankedSnapshot>` (not `Option<…>`) — use `r.take(0).unwrap_or_default()` (Rule 28)
- Add `rank_score` field computed in the Rust `From<Db*>` conversion (not stored in DB)
- Window filtering: pass `Option<chrono::DateTime>` cutoff computed in Rust, bind as `$cutoff`; omit `WHERE … >= $cutoff` for `AllTime` variant
- On empty result: `Ok(Vec::new())` — never `Err` (Rule 44)
- `ORDER BY snapshotted_at ASC` (oldest→newest for chart left-to-right)

**Concrete Db struct + From impl pattern** (copy shape from `DbPersonalLearning:4789–4831`):
```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct DbRankedSnapshotRow {
    // local name — not to conflict with existing DbRankedSnapshot at line 4571
    tier: String,
    division: String,
    lp: i32,
    snapshotted_at: String,
}

pub async fn get_lp_history(
    db: &Surreal<Db>,
    user_id: &str,
    cutoff: Option<String>, // ISO datetime string or None for all-time
) -> DbResult<Vec<crate::models::match_data::RankedSnapshot>> {
    use crate::models::match_data::rank_score;
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let rows: Vec<DbRankedSnapshotRow> = if let Some(cutoff_val) = cutoff {
        let mut r = db
            .query("SELECT tier, division, lp, <string>snapshotted_at AS snapshotted_at \
                    FROM ranked_snapshot \
                    WHERE user = type::record('user', $user_key) \
                    AND queue_type = 'RANKED_SOLO_5x5' \
                    AND snapshotted_at >= $cutoff \
                    ORDER BY snapshotted_at ASC")
            .bind(("user_key", user_key))
            .bind(("cutoff", cutoff_val))
            .await?;
        r.take(0).unwrap_or_default()
    } else {
        let mut r = db
            .query("SELECT tier, division, lp, <string>snapshotted_at AS snapshotted_at \
                    FROM ranked_snapshot \
                    WHERE user = type::record('user', $user_key) \
                    AND queue_type = 'RANKED_SOLO_5x5' \
                    ORDER BY snapshotted_at ASC")
            .bind(("user_key", user_key))
            .await?;
        r.take(0).unwrap_or_default()
    };

    Ok(rows.into_iter().map(|r| {
        let score = rank_score(&r.tier, &r.division, r.lp);
        crate::models::match_data::RankedSnapshot {
            id: None,
            tier: r.tier,
            division: r.division,
            lp: r.lp,
            snapshotted_at: r.snapshotted_at,
            rank_score: score,
        }
    }).collect())
}
```

---

#### `get_personal_goals(db, user_id)` and `upsert_personal_goal(db, user_id, goal_type, target_value)`

**Analog for read:** `db.rs:4883` (`list_personal_learnings`) — same `SELECT * FROM <table> WHERE user = type::record(...)` pattern

**Analog for upsert:** `db.rs:2285–2311` (`set_ban_priorities`) — BEGIN TRANSACTION / DELETE old / CREATE new / COMMIT

**`get_personal_goals` pattern** (copy from `list_personal_learnings:4883–4895`):
```rust
pub async fn list_personal_learnings(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<PersonalLearning>> {
    let user_key = user_id.strip_prefix("user:")...;
    let mut r = db
        .query("SELECT * FROM personal_learning WHERE user = type::record('user', $user_key)")
        .bind(("user_key", user_key)).await?;
    let rows: Vec<DbPersonalLearning> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(PersonalLearning::from).collect())
}
```

**`upsert_personal_goal` pattern** (adapt from `set_ban_priorities:2285–2311`):
```rust
pub async fn set_ban_priorities(db: &Surreal<Db>, ...) -> DbResult<()> {
    let mut query = String::from(
        "BEGIN TRANSACTION; DELETE ban_priority WHERE team = type::record('team', $team_key);"
    );
    // ... build CREATE statements ...
    query.push_str(" COMMIT TRANSACTION;");
    db.query(&query).bind(...).await?.check()?;
    Ok(())
}
```

For `upsert_personal_goal` (single record, not a Vec):
```rust
pub async fn upsert_personal_goal(
    db: &Surreal<Db>,
    user_id: &str,
    goal_type: &str,
    target_value: &str,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query(
        "BEGIN TRANSACTION; \
         DELETE personal_goal WHERE user = type::record('user', $user_key) \
           AND goal_type = $goal_type; \
         CREATE personal_goal SET \
           user = type::record('user', $user_key), \
           goal_type = $goal_type, \
           target_value = $target_value, \
           created_at = time::now(), \
           updated_at = time::now(); \
         COMMIT TRANSACTION;"
    )
    .bind(("user_key", user_key))
    .bind(("goal_type", goal_type.to_string()))
    .bind(("target_value", target_value.to_string()))
    .await?.check()?;
    Ok(())
}
```

---

#### `compute_goal_progress(db, user_id)` — batched multi-query

**Analog:** `db.rs:4580–4594` for single `.query()` call structure; RESEARCH.md Pattern 5 for batched approach

**Batched query pattern** (from `db.rs:29` rule — batch multiple queries in one call):
```rust
// Rule 29: index results by statement order
let mut r = db.query("SELECT ...; SELECT ...; SELECT ...;")
    .bind(("user_key", user_key)).await?;
let goals: Vec<DbPersonalGoal> = r.take(0).unwrap_or_default();
let recent_matches: Vec<DbGoalMatchRow> = r.take(1).unwrap_or_default();
let snapshot: Option<DbSnapshotRow> = r.take(2)?;
```

**Queue link-traversal pattern** (from `db.rs:4611` `get_solo_matches`):
```rust
"SELECT * FROM player_match WHERE user = type::record('user', $user_key) AND match.queue_id = $queue_id LIMIT $limit"
```

**Aggregation in Rust** (from `db.rs:464–528` `get_champion_stats_for_user`):
- Fetch rows, then aggregate in Rust with `HashMap` — not SurrealQL GROUP BY
- `avg_kda = (kills + assists) / max(deaths, 1.0)`
- CS/min uses `game_duration` from the linked `match` record: `cs as f32 / (game_duration as f32 / 60.0)` — seconds, not minutes (Pitfall 1 in RESEARCH.md)

**Return `Ok(Vec::new())` / `Ok(GoalProgressPayload::empty())` for absent data** (Rule 44 — same as `get_solo_dashboard` returns data even for empty)

---

#### `get_champion_trends(db, user_id, cutoff)` — queue filter + time window

**Analog:** `db.rs:464–528` (`get_champion_stats_for_user`) — exact same pattern but with `match.queue_id IN [420, 440]` and `match.game_end >= $cutoff`

**Pattern to copy:**
```rust
pub async fn get_champion_stats_for_user(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<ChampionStatSummary>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlayerMatchRow {
        champion: String,
        kills: i64,
        deaths: i64,
        assists: i64,
        cs: i64,
        win: bool,
    }

    let mut r = db
        .query("SELECT champion, kills, deaths, assists, cs, win FROM player_match \
                WHERE user = type::record('user', $user_key)")
        .bind(("user_key", user_key)).await?;

    let rows: Vec<PlayerMatchRow> = r.take(0).unwrap_or_default();

    let mut by_champ: HashMap<String, Vec<(f64, f64, f64, f64, bool)>> = HashMap::new();
    for row in &rows { by_champ.entry(row.champion.clone()).or_default().push(...); }

    let mut results: Vec<ChampionStatSummary> = by_champ.into_iter().map(|(champion, matches)| {
        let games = matches.len() as i32;
        let wins = matches.iter().filter(|m| m.4).count() as i32;
        let avg_kda = if total_deaths > 0.0 { (kills + assists) / deaths } else { kills + assists };
        ChampionStatSummary { champion, games, wins,
            avg_kda: (avg_kda * 10.0).round() / 10.0,
            avg_cs_per_min: (avg_cs * 10.0).round() / 10.0,
        }
    }).collect();
    results.sort_by(|a, b| b.games.cmp(&a.games));
    Ok(results)
}
```

**`get_champion_trends` adaptations:**
- Local `Db*` row struct needs `damage: i64` and `game_duration: i64` (from linked `match` record via `match.game_duration as game_duration`)
- Queue filter: `AND (match.queue_id = 420 OR match.queue_id = 440)`
- Time window: `AND match.game_end >= $cutoff` (omit for AllTime; include `OR match.game_end IS NONE` for AllTime per RESEARCH.md Pitfall 4)
- CS/min: `total_cs / (total_game_duration_sec / 60.0)` — NOT `total_cs / total_game_duration_sec`
- Returns `Vec<ChampionTrend>` (new model struct, not `ChampionStatSummary`)
- `Ok(Vec::new())` when no qualifying rows

---

### `src/pages/solo_dashboard.rs` — modify GoalPlaceholders + add LpHistoryGraph

**Section being modified:** Lines 244–270 (Suspense block and GoalPlaceholders call), and lines 406–444 (GoalPlaceholders component body).

**Existing `Suspense` + `into_any()` pattern** (lines 244–267):
```rust
<Suspense fallback=|| view! {
    <div class="flex flex-col gap-4">
        <SkeletonCard height="h-28" />
        // ... more skeletons
    </div>
}>
    {move || dashboard_resource.get().map(|result| match result {
        Err(e) => view! {
            <ErrorBanner message=format!("Failed to load dashboard: {e}") />
        }.into_any(),
        Ok(data) => view! {
            <div class="flex flex-col gap-8">
                <RankedBadgeSection ranked=data.ranked />
                <MatchListSection matches=data.matches queue_filter=queue_filter />
                <GoalPlaceholders />  // ← REPLACE with <GoalCards progress=... />
            </div>
        }.into_any(),
    })}
</Suspense>
```

**Where to insert `<LpHistoryGraph>`** — between `<RankedBadgeSection>` and `<MatchListSection>`, inside the `Ok(data)` arm.

**New server functions** (define before the `#[component]` — Rule 34):
- `get_lp_history(window: String) -> Result<Vec<RankedSnapshot>, ServerFnError>`
- `compute_goal_progress() -> Result<GoalProgressPayload, ServerFnError>`
- `upsert_personal_goal(goal_type: String, target_value: String) -> Result<(), ServerFnError>`

Each follows the **server fn body pattern** from `get_solo_dashboard` (lines 22–63):
```rust
#[server]
pub async fn get_solo_dashboard(queue_filter: Option<i32>) -> Result<SoloDashboardData, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // ... db:: calls ...
    db::some_fn(&surreal, &user.id, ...).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(SoloDashboardData { ... })
}
```

**`spawn_local` mutation pattern** for Save Goal button (from `solo_dashboard.rs:200–218`):
```rust
leptos::task::spawn_local(async move {
    match upsert_personal_goal(goal_type, target_value).await {
        Ok(_) => {
            set_editing.set(false);
            goal_progress_resource.refetch();   // Rule 23
        }
        Err(e) => set_error.set(Some(format!("{e}"))),
    }
});
```

**`into_any()` for divergent goal card branches** (Rule 19):
```rust
{move || if editing_rank.get() {
    view! { /* edit form */ }.into_any()
} else if goal_progress.rank.is_none() {
    view! { /* unset state */ }.into_any()
} else if goal_progress.rank.as_ref().map(|g| g.game_count < 5).unwrap_or(true) {
    view! { /* insufficient data state */ }.into_any()
} else {
    view! { /* active state with progress */ }.into_any()
}}
```

**Inline edit form input pattern** (from `stats.rs:300–314` select + `stats.rs` form classes):
```rust
<select
    class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm \
           focus:outline-none focus:border-accent/50 transition-colors w-full"
    on:change=move |ev| set_tier_edit.set(event_target_value(&ev))
>
    <option value="GOLD">"Gold"</option>
    // ...
</select>
```

**Controlled input pattern** (Rule 21 — `prop:value` not `attr:value`):
```rust
<input
    type="number"
    prop:value=move || cs_edit.get()
    on:input=move |ev| set_cs_edit.set(event_target_value(&ev))
    class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 \
           text-primary text-sm focus:outline-none focus:border-accent/50 w-full"
    min="0" max="15" step="0.1"
/>
```

---

### `src/pages/solo_dashboard.rs` — inline `LpHistoryGraph` component

**Analog:** `src/components/tree_graph.rs` — SVG coordinate math + CSS token approach

**SVG color token pattern** (tree_graph.rs lines 257–280):
```rust
// CORRECT: SVG stroke/fill use var(--t-*) CSS custom properties, NOT Tailwind class names
stroke="var(--t-divider)"       // grid lines
fill="var(--t-elevated)"        // node backgrounds
stroke="var(--t-accent, #22c55e)"  // accent elements
fill="var(--t-muted)"           // muted text elements
```

**SVG viewBox + responsive sizing** (adapt from tree_graph.rs component open):
```rust
<svg
    viewBox="0 0 800 160"
    preserveAspectRatio="none"
    style="width: 100%; height: 160px; display: block;"
    // on:mousemove handled below
>
```

**Hover signal pattern** (hydrate-only, per RESEARCH.md Pattern 9):
```rust
// Declare signal unconditionally (SSR must compile)
let tooltip: RwSignal<Option<(f64, f64, RankedSnapshot)>> = RwSignal::new(None);

// on:mousemove body guards WASM-only operations
on:mousemove=move |ev| {
    #[cfg(feature = "hydrate")]
    {
        // compute nearest data point...
        // use if let Some(win) = web_sys::window() — never .unwrap() (Rule 35)
        let rect = ...;
        let cx = ev.client_x() as f64 - rect.left();
        // find nearest point within 12px
        tooltip.set(Some((px, py, snapshot)));
    }
}

on:mouseleave=move |_| tooltip.set(None)
```

**SVG `<path>` area fill pattern** (copy from tree_graph.rs edges, adapt for line chart):
```rust
// Area fill (render before line so it's below)
let area_d = format!(
    "M {},{} {} L {},{} L {},{} Z",
    first_x, first_y,
    // ... L x,y for each point ...
    last_x, bottom_y,
    first_x, bottom_y,
);
view! {
    <path d=area_d fill="var(--t-accent)" opacity="0.1" />
    <polyline
        points=point_string
        fill="none"
        stroke="var(--t-accent)"
        stroke-width="2"
        stroke-linejoin="round"
        stroke-linecap="round"
    />
}
```

**Coordinate math constants** (adapt from tree_graph.rs layout constants at lines 5–12):
```rust
const SVG_W: f64 = 800.0;
const SVG_H: f64 = 160.0;
const Y_AXIS_W: f64 = 48.0;
const X_AXIS_H: f64 = 24.0;

// x_i = Y_AXIS_W + i * ((SVG_W - Y_AXIS_W) / max(n - 1, 1))
// y_i = SVG_H - X_AXIS_H - (score_i - min_score) * y_scale
// y_scale = (SVG_H - X_AXIS_H) / max(max_score - min_score, 1)
```

**Time-window toggle pill pattern** (no existing analog — use button + RwSignal):
```rust
let lp_window: RwSignal<&'static str> = RwSignal::new("30d");
let lp_history_resource = Resource::new(
    move || lp_window.get(),
    |w| get_lp_history(w.to_string()),
);

// Active pill: "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
// Inactive pill: "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
{["7d", "30d", "90d", "All-time"].iter().map(|&w| {
    let is_active = move || lp_window.get() == w;
    view! {
        <button
            class=move || if is_active() {
                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
            } else {
                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
            }
            on:click=move |_| lp_window.set(w)
        >{w}</button>
    }
}).collect_view()}
```

---

### `src/pages/stats.rs` — new `ChampionTrendsSection` component

**Analog:** `stats.rs:474–522` (`StatsContent`) — `StoredValue` for non-reactive data + filter signals + computed closure

**Key patterns from StatsContent** (lines 487–522):
```rust
#[component]
fn StatsContent(
    all_matches: Vec<MatchGroup>,
    // ... signals as props
) -> impl IntoView {
    let all_matches = StoredValue::new(all_matches);  // Rule 22 — StoredValue for large non-Copy data

    let filtered = move || {
        all_matches.with_value(|matches| {
            matches.iter().filter(|m| { ... }).cloned().collect::<Vec<_>>()
        })
    };
    // ... view uses filtered() in closures
}
```

**Client-side sort with Memo** (from RESEARCH.md Pattern — client-side sort):
```rust
let sort_col: RwSignal<SortColumn> = RwSignal::new(SortColumn::Games);
let sort_dir: RwSignal<bool> = RwSignal::new(false); // false=desc
let show_all: RwSignal<bool> = RwSignal::new(false);

let sorted_trends = Memo::new(move |_| {
    let data = match trends_resource.get() {
        Some(Ok(d)) => d,
        _ => return Vec::new(),
    };
    let min = if show_all.get() { 0 } else { 3 };
    let mut filtered: Vec<ChampionTrend> = data.into_iter().filter(|t| t.games >= min).collect();
    filtered.sort_by(|a, b| {
        let ord = match sort_col.get() {
            SortColumn::Games => a.games.cmp(&b.games),
            SortColumn::WinPct => (a.wins * 100 / a.games.max(1)).cmp(&(b.wins * 100 / b.games.max(1))),
            SortColumn::Kda => a.avg_kda.partial_cmp(&b.avg_kda).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::CsPerMin => a.cs_per_min.partial_cmp(&b.cs_per_min).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::AvgDamage => a.avg_damage.cmp(&b.avg_damage),
            SortColumn::Champion => a.champion.cmp(&b.champion),
        };
        if sort_dir.get() { ord } else { ord.reverse() }  // false=desc = reversed
    });
    filtered
});
```

**Table styling pattern** (from `stats.rs` StatsContent scoreboard rows, around line 830–854):
```rust
// Header cell
<th class="text-xs font-normal text-muted uppercase tracking-wider cursor-pointer ...">
// Body cell
<td class="py-2 text-center text-secondary text-sm">
// Row
<tr class="border-t border-divider/30 hover:bg-elevated/30 transition-colors">
```

**`format_damage` reuse** (stats.rs:237–243):
```rust
fn format_damage(damage: i32) -> String {
    if damage >= 1000 { format!("{:.1}k", damage as f64 / 1000.0) }
    else { damage.to_string() }
}
```
This function is already in `stats.rs` — call it directly from the new `ChampionTrendsSection` inline component in the same file.

**`champion_icon_url` reuse** (stats.rs:233–235):
```rust
fn champion_icon_url(champion_name: &str) -> String {
    format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{champion_name}.png")
}
```
Already in `stats.rs` — reuse directly.

**Champion icon fallback pattern** (Rule — `into_any()` + `on:error`):
```rust
let icon_errored: RwSignal<bool> = RwSignal::new(false);
{move || if icon_errored.get() {
    view! { <div class="w-5 h-5 rounded bg-elevated border border-divider/30" /> }.into_any()
} else {
    view! {
        <img
            src=champion_icon_url(&champ)
            class="w-5 h-5 rounded object-contain"
            on:error=move |_| icon_errored.set(true)
        />
    }.into_any()
}}
```

**Page-level `Suspense` + `ErrorBanner` + `EmptyState` pattern** (stats.rs:414–469):
```rust
<Suspense fallback=|| view! { <SkeletonCard height="h-48" /> }>
    {move || trends_resource.get().map(|result| match result {
        Err(e) => view! {
            <ErrorBanner message=format!("Could not load champion trends. Refresh to try again.") />
        }.into_any(),
        Ok(rows) if rows.is_empty() => view! {
            <EmptyState message="Sync your match history to see champion trends." />
        }.into_any(),
        Ok(_) => view! { <ChampionTrendsTable ... /> }.into_any(),
    })}
</Suspense>
```

**Server fn for champion trends** (same shape as `get_solo_dashboard:22–63`):
```rust
#[server]
pub async fn get_champion_trends(window: String) -> Result<Vec<ChampionTrend>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let cutoff = window_to_cutoff(&window); // compute Option<String> from "7d"/"30d"/etc.
    db::get_champion_trends(&surreal, &user.id, cutoff).await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
```

---

### `tests/db_personal_goal.rs`, `tests/db_lp_history.rs`, `tests/db_champion_trends.rs`

**Analog:** `tests/db_champion_pool.rs` — exact same file-level structure

**File header pattern** (db_champion_pool.rs lines 1–16):
```rust
#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> String {
    db::create_user(db, "testuser".into(), "test@example.com".into(), "hash".into())
        .await.unwrap()
}
```

**Test case pattern** (db_champion_pool.rs lines 18–31):
```rust
#[tokio::test]
async fn test_add_champion_to_pool() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    db::add_to_champion_pool(&db, &user_id, "Jinx".into(), "bot".into()).await.unwrap();

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert_eq!(pool.len(), 1);
    assert_eq!(pool[0].champion, "Jinx");
}
```

**For `db_personal_goal.rs` — key test cases:**
- `upsert_creates_new_goal` — upsert where none exists → `get_personal_goals` returns 1 entry
- `upsert_overwrites_existing_goal` — upsert same `goal_type` twice → still 1 entry, new value
- `upsert_different_types_creates_separate_goals` — 3 different `goal_type` values → 3 entries
- `get_personal_goals_empty_returns_empty_vec` — user with no goals → `Ok(vec![])`

**For `db_lp_history.rs` — key test cases:**
- `get_lp_history_empty_returns_empty_vec`
- `get_lp_history_sorted_ascending` — multiple snapshots returned oldest→newest
- `rank_score_unit_test` — inline test of `rank_score()` function

**For `db_champion_trends.rs` — key test cases:**
- `get_champion_trends_empty_returns_empty_vec`
- `get_champion_trends_aggregates_correctly` — insert 2 player_match rows for same champion, verify wins/kda computed right
- `get_champion_trends_cs_per_min_uses_seconds` — cs=180, game_duration=1800 → cs_per_min=6.0 (not 0.1)

**`common::test_db()` is already in `tests/common/mod.rs`** — import with `mod common;` exactly as db_champion_pool.rs does.

---

## Shared Patterns

### Authentication in server functions
**Source:** `src/pages/solo_dashboard.rs:31–36`
**Apply to:** All 3 new server functions (`get_lp_history`, `compute_goal_progress`, `upsert_personal_goal`, `get_champion_trends`)
```rust
let auth: AuthSession = leptos_axum::extract().await?;
let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
let surreal = use_context::<Arc<Surreal<Db>>>()
    .ok_or_else(|| ServerFnError::new("No DB context"))?;
```

### Error mapping
**Source:** `src/pages/solo_dashboard.rs:47–48`
**Apply to:** All `db::` calls in server functions
```rust
db::some_fn(&surreal, &user.id).await
    .map_err(|e| ServerFnError::new(e.to_string()))?
```

### Empty list for absent data (not Err)
**Source:** Rule 44 (leptos-patterns.md); `solo_dashboard.rs:53–55`
**Apply to:** `get_lp_history`, `get_personal_goals`, `get_champion_trends`
```rust
// When user has no matching rows, return Ok(Vec::new()), never Err
Ok(Vec::new())
```

### `resource.refetch()` after mutations
**Source:** `src/pages/solo_dashboard.rs:186` + Rule 23
**Apply to:** `upsert_personal_goal` success path in GoalCards
```rust
leptos::task::spawn_local(async move {
    match upsert_personal_goal(goal_type, target_value).await {
        Ok(_) => {
            set_editing.set(false);
            goal_progress_resource.refetch();  // mandatory after mutation
        }
        Err(e) => set_error.set(Some(e.to_string())),
    }
});
```

### SVG CSS custom properties (not Tailwind class names)
**Source:** `src/components/tree_graph.rs:257–280`
**Apply to:** `LpHistoryGraph` SVG element attributes
```rust
// CORRECT for SVG attributes — var(--t-*) not class="stroke-accent"
stroke="var(--t-divider)"
fill="var(--t-elevated)"
stroke="var(--t-accent)"
fill="var(--t-muted)"
```

### SurrealDB write pattern (`.check()` after writes)
**Source:** Rule 27 (surreal-patterns.md); `db.rs:276`
**Apply to:** `upsert_personal_goal` transaction
```rust
db.query("...").bind(...).await?.check()?;
```

### `ORDER BY` on selected fields only
**Source:** Rule 40 (surreal-patterns.md); MEMORY.md bug fix note
**Apply to:** `get_lp_history` query — always include `snapshotted_at` in SELECT
```sql
-- BAD: ORDER BY snapshotted_at without selecting it
SELECT tier, division, lp FROM ranked_snapshot ... ORDER BY snapshotted_at ASC

-- GOOD: either SELECT * or include the sort field
SELECT tier, division, lp, <string>snapshotted_at AS snapshotted_at FROM ranked_snapshot
... ORDER BY snapshotted_at ASC
```

### StatusMessage for inline form errors
**Source:** `src/components/ui.rs:19–29`
**Apply to:** GoalCards edit form save error; message must start with "Error" to show red
```rust
// src/components/ui.rs:20 — "Error" prefix triggers red styling
let is_err = message.starts_with("Error");
// Usage:
set_error.set(Some(format!("Error: Failed to save goal. Try again.")));
<StatusMessage message=error_msg />
```

### SkeletonCard / SkeletonGrid for loading states
**Source:** `src/components/ui.rs:170–193`
**Apply to:**
- LP graph loading: `<SkeletonCard height="h-48" />`
- GoalCards loading: `<SkeletonGrid cols=3 rows=1 card_height="h-28" />`
- Champion Trends loading: `<SkeletonCard height="h-48" />`

---

## No Analog Found

All files have close analogs. No "no analog" entries.

---

## Metadata

**Analog search scope:** `src/models/`, `src/server/db.rs`, `src/pages/`, `src/components/`, `tests/`, `schema.surql`
**Files scanned:** 14 source files read directly; ~20 additional files grep-searched
**Pattern extraction date:** 2026-05-05
