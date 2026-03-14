# Phase 02: Aggregation Layer - Research

**Researched:** 2026-03-15
**Domain:** Rust / SurrealDB cross-table queries, champion name normalization, Leptos autocomplete enforcement
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Champion Pool Gap Detection**
- Gaps are about champion class/archetype diversity, not lane coverage
- Use Riot's Data Dragon class tags (Fighter, Mage, Assassin, Tank, Marksman, Support) to assess pool diversity per player per role
- Warn when a player's pool is dominated by a single class
- Cross-reference with opponent scouting data: if an opponent's ban/pick tendencies target a player's dominant class, escalate the warning
- Gap query joins champion_pool entries with opponent_player champion data

**Champion Performance Summary**
- Four data sources: Riot API match stats, post-game review outcomes, draft pick frequency, and game plan results
- Both per-player and team-wide aggregation queries (separate functions)
- Default time window: last 30 days or last 20 games (whichever is more); query accepts optional window parameter
- Win/loss attribution uses the post-game review outcome field (not win condition pass rate)
- Aggregation happens in Rust, not SurrealDB GROUP BY views

**Champion Name Normalization**
- Canonical form: Data Dragon key (e.g. `"KSante"`, `"AurelionSol"`, `"Jinx"`) — URL-safe, matches Riot API
- Migration: existing records with display names normalized (Claude decides startup script vs lazy migration)
- All champion input points must use validated autocomplete — no freeform champion name entry
- Applies to: drafts, champion pools, game plans (our_champions/enemy_champions), opponent scouting (recent_champions), tree drafter nodes

**Dashboard Summary Query**
- Single batched `get_dashboard_summary(team_id)` returning all panel data in one round-trip
- Action items: total open count + top 3 most recent items with titles
- Post-game summaries: last 5 reviews with patterns identified
- Pool gap warnings: per-player class diversity issues
- Workflow completeness: counts of drafts without game plans, game plans without post-game reviews
- Returns `Ok(empty defaults)` when team has no data — never `Err`

### Claude's Discretion
- Exact threshold for single-class dominance warning (e.g. >70% or missing class entirely)
- Migration approach (startup script vs lazy normalization)
- Exact return type shapes for summary structs
- How to batch the dashboard summary sub-queries efficiently
- Integration test structure and coverage depth

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

## Summary

Phase 2 adds cross-table aggregation queries to `src/server/db.rs` and enforces champion name normalization at all ingestion points. The codebase already has a strong pattern for Rust-side aggregation (`get_champion_stats_for_user`, `get_draft_outcome_stats`, `get_win_condition_stats`), all of which perform multi-query batching via chained `.query("SELECT...; SELECT...")` and positional `.take(N)`. This phase follows that exact pattern.

Champion name normalization is the highest-risk sub-task. The `champion_autocomplete` component already exists and filters by `c.name` (display name), but stores `value.set(name)` — it sets the display name, not the Data Dragon ID. All champion storage fields (`draft_action.champion`, `champion_pool.champion`, `game_plan.our_champions`, etc.) are free-form strings. The canonical form must be the Data Dragon `id` field (`Champion.id` in `src/models/champion.rs`), and the autocomplete must be updated to set the ID on selection while displaying the name.

The dashboard summary query is straightforward given existing `list_open_action_items`, `list_post_game_learnings`, and `get_game_plans_for_draft` patterns. The pool gap analysis requires loading all champion_pool entries for team members, calling `data_dragon::fetch_champions()` for class tags, and computing class diversity in Rust.

**Primary recommendation:** Implement normalization first (it unblocks correct aggregation), then add new query functions in `db.rs`, then update the autocomplete enforcement — in that order.

---

## Standard Stack

### Core (all already in project)

| Library | Version | Purpose | Role in Phase |
|---------|---------|---------|--------------|
| `surrealdb` | 3.x | DB queries | All new `SELECT` queries |
| `surrealdb-types-derive` | project-pin | `#[derive(SurrealValue)]` | All new `Db*` structs |
| `reqwest` | project-pin | HTTP | `data_dragon::fetch_champions()` for class tags |
| `chrono` | project-pin | Date filtering | 30-day window in performance queries |

### No New Dependencies

This phase adds no new crate dependencies. All tools exist.

---

## Architecture Patterns

### Pattern 1: Batched Query with Multiple Result Sets (HIGH confidence)

Already used in `get_draft_outcome_stats` and `list_drafts`. All new aggregate queries follow this.

```rust
// Source: src/server/db.rs — get_win_condition_stats pattern
let mut result = db
    .query("SELECT ...; SELECT ...; SELECT ...;")
    .bind(("team_key", team_key))
    .await?;
let plans: Vec<PlanTag>      = result.take(0).unwrap_or_default();
let post_games: Vec<DbPostGame> = result.take(1).unwrap_or_default();
let matches: Vec<DbMatchRef> = result.take(2).unwrap_or_default();
// Aggregate in Rust with HashMap
```

### Pattern 2: Db* Struct with RecordId → App Struct Conversion (HIGH confidence)

All queries use `#[derive(Debug, Deserialize, SurrealValue)]` on DB-facing structs. Champion names in DB are stored as strings (no RecordId for champions).

```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct DbPoolEntry { id: RecordId, user: RecordId, champion: String, role: String, ... }
impl From<DbPoolEntry> for ChampionPoolEntry { ... }
```

### Pattern 3: Team-Scoped Queries via `type::record('team', $team_key)` (HIGH confidence)

All team-scoped queries strip the `"team:"` prefix and use `type::record('team', $team_key)`. New queries do the same.

### Pattern 4: Empty-Result Guard for Optional Team (HIGH confidence)

From CLAUDE.md rule 44 and confirmed in multiple existing functions:

```rust
let team_id = match db::get_user_team_id(db, user_id).await? {
    Some(id) => id,
    None => return Ok(DashboardSummary::default()),
};
```

### Pattern 5: Data Dragon Fetch for Champion Metadata (HIGH confidence)

`data_dragon::fetch_champions()` returns `Vec<Champion>` where `Champion.id` is the Data Dragon key and `Champion.tags` is the class list. This is the source of canonical names AND class tags.

```rust
// Source: src/server/data_dragon.rs
pub struct Champion {
    pub id: String,      // "KSante", "AurelionSol" — canonical form
    pub name: String,    // "K'Sante", "Aurelion Sol" — display name
    pub tags: Vec<String>, // ["Fighter", "Tank"]
    pub image_full: String,
}
```

### Pattern 6: Autocomplete `on_select` Callback (HIGH confidence)

`ChampionAutocomplete` already has `on_select: Option<Callback<String>>`. The enforcement fix: `select_champion` should set `value` to `champion.id` (not `champion.name`), and `filter_text` to `champion.name` for display. The component receives `champions: Vec<Champion>` already.

```rust
// Current (WRONG): sets value to display name
value.set(name.clone());
// Fixed: set value to ID, display the name
value.set(c.id.clone());   // stored value = canonical key
set_filter_text.set(c.name.clone());  // displayed text = display name
```

### Recommended New Struct Shapes (Claude's Discretion)

```rust
// src/models/ — new shared structs
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DashboardSummary {
    pub open_action_item_count: usize,
    pub recent_action_items: Vec<ActionItemPreview>,   // top 3
    pub recent_post_games: Vec<PostGamePreview>,        // last 5
    pub pool_gap_warnings: Vec<PoolGapWarning>,
    pub drafts_without_game_plan: usize,
    pub game_plans_without_post_game: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionItemPreview { pub id: String, pub text: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostGamePreview {
    pub id: String,
    pub improvements: Vec<String>,  // patterns identified
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolGapWarning {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub dominant_class: Option<String>,  // None = missing class(es)
    pub missing_classes: Vec<String>,
    pub opponent_escalated: bool,  // true if opponent targets this class
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ChampionPerformanceSummary {
    pub champion: String,
    pub games_in_draft: usize,
    pub games_in_match: usize,
    pub wins_in_match: usize,
    pub games_in_plan: usize,
    pub post_game_wins: usize,   // post-game reviews that recorded this champ as win
}
```

### Anti-Patterns to Avoid

- **SurrealDB GROUP BY views:** Explicitly locked out by context. Do all aggregation in Rust with `HashMap`.
- **Returning `Err` from summary queries when team is absent:** Must return `Ok(DashboardSummary::default())`.
- **Freeform champion text input after this phase:** All champion inputs must go through `ChampionAutocomplete` with `on_select` enforced.
- **Lazy normalization in the summary query:** Normalize at ingestion, not at query time. The dashboard query should not need to massage champion names.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Champion canonical lookup | Manual string normalization | `data_dragon::fetch_champions()` + `HashMap<String, String>` (name→id) | Data Dragon is authoritative; hand-normalization misses edge cases like `"K'Sante"` → `"KSante"` |
| Class tag lookup | Hard-code class lists | `Champion.tags` from Data Dragon | Tags evolve with new champions |
| Champion display vs storage mismatch | Any runtime lookup table in WASM | Store canonical ID always; look up display name only for rendering | Mixing display and canonical forms is the root cause of cross-feature join failures |

---

## Common Pitfalls

### Pitfall 1: Autocomplete Stores Display Name, Not ID
**What goes wrong:** `ChampionAutocomplete` currently calls `value.set(name.clone())` where `name` is `c.name` (display name like "K'Sante"). Cross-feature joins on `champion = "K'Sante"` fail against champion_pool records that have `"KSante"`.
**Why it happens:** The component was built before normalization was a requirement.
**How to avoid:** Change `select_champion` to take a `Champion` struct (not just `name: String`), set `value.set(c.id.clone())` for storage and `set_filter_text.set(c.name.clone())` for display. All callers of `on_select` receive the canonical ID.
**Warning signs:** `filter_text` and `value` signals contain different strings after selection.

### Pitfall 2: Migration Scope Unknown Until Data Is Queried
**What goes wrong:** Existing champion_pool, game_plan, draft_action, and opponent_player records may have inconsistent forms ("K'Sante", "Ksante", "KSante"). A migration that assumes a single existing form will leave some records unnormalized.
**Why it happens:** Multiple input paths existed before enforcement.
**How to avoid:** Startup migration script loads Data Dragon, builds a `HashMap<String, String>` (lowercase-stripped → canonical ID) for fuzzy matching, then bulk-updates any record where `champion` does not exactly match a known canonical ID. Log every update. Run as part of `init_db`.
**Warning signs:** After migration, `SELECT DISTINCT champion FROM champion_pool` returns any value not in the Data Dragon champion list.

### Pitfall 3: `ORDER BY` on Fields Not in Partial SELECT (CLAUDE.md rule 40)
**What goes wrong:** SurrealDB 3.x rejects `ORDER BY created_at DESC` if the partial `SELECT` clause doesn't include `created_at`.
**Why it happens:** New queries that project only specific fields (e.g. `SELECT id, text FROM action_item ORDER BY created_at DESC`) hit this.
**How to avoid:** Either `SELECT *` or explicitly include the ORDER BY field in the SELECT clause.

### Pitfall 4: `data_dragon::fetch_champions()` Network Call in Aggregation Hot Path
**What goes wrong:** `get_dashboard_summary` calls `fetch_champions()` on every invocation. This adds ~200-500ms of network latency on each dashboard load.
**Why it happens:** No caching layer exists for Data Dragon data.
**How to avoid:** In Phase 2, accept this cost — dashboard is not a hot path. Document it as a known limitation. Phase 3 can add in-memory caching (`tokio::sync::OnceCell` or `Arc<RwLock<Vec<Champion>>>`).

### Pitfall 5: `SurrealValue` Derive on Inline Structs Inside Functions
**What goes wrong:** SurrealDB deserialization requires `#[derive(SurrealValue)]` on structs used with `.take()`. Inline structs defined inside async functions work fine for this (see existing pattern in `get_champion_stats_for_user`).
**Why it happens:** Not a pitfall — this is the established project pattern. Document so planners don't over-think it.
**How to avoid:** Define inline `Db*` structs inside the function body if they're only used there. Move to top-level `Db*` structs only when shared.

### Pitfall 6: Champion Pool Gap Analysis Requires Team Member → User → Pool Join
**What goes wrong:** `champion_pool` is user-scoped, not team-scoped. Analyzing pool gaps for a team requires: get team members → for each member, get their pool → aggregate per member per role.
**Why it happens:** Pool is personal data; team membership is the bridge.
**How to avoid:** First query gets team member user IDs (single query). Second query gets all pool entries for those user IDs using `user IN [list]` syntax (as in `get_team_match_stats`). Then aggregate in Rust per user per role.

---

## Code Examples

### Example 1: Team-scoped Champion Pool Fetch (Rust-side join)

```rust
// Pattern from get_team_match_stats — build user ID list, use IN clause
let user_ids: Vec<String> = user_refs.iter().map(|u| u.user.to_sql()).collect();
let user_id_list = user_ids.join(", ");
let query = format!(
    "SELECT user, champion, role, tier FROM champion_pool WHERE user IN [{user_id_list}]"
);
let mut result = db.query(&query).await?;
let entries: Vec<DbPoolEntry> = result.take(0).unwrap_or_default();
```

### Example 2: Dashboard Summary — 5-Statement Batch

```rust
// All five sub-queries in one round-trip
let mut result = db.query(
    "SELECT *, <string>created_at AS created_at FROM action_item WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] ORDER BY created_at DESC LIMIT 3;
     SELECT count() as n FROM action_item WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] GROUP ALL;
     SELECT id, improvements, <string>created_at AS created_at FROM post_game_learning WHERE team = type::record('team', $team_key) ORDER BY created_at DESC LIMIT 5;
     SELECT count() as n FROM draft WHERE team = type::record('team', $team_key) AND draft NOT IN (SELECT VALUE draft FROM game_plan WHERE draft != NONE) GROUP ALL;
     SELECT count() as n FROM game_plan WHERE team = type::record('team', $team_key) AND id NOT IN (SELECT VALUE game_plan_id FROM post_game_learning WHERE game_plan_id != NONE) GROUP ALL"
)
.bind(("team_key", team_key))
.await?;
// result.take(0) = top 3 open action items
// result.take(1) = total open count
// result.take(2) = last 5 post-game previews
// result.take(3) = drafts without game plans count
// result.take(4) = game plans without post-game count
```

Note: The "drafts without game plans" sub-query relies on the Phase 1 FK link. `game_plan.draft` is stored as a plain `option<string>` (not a record reference), so subquery approach with `draft NOT IN (SELECT VALUE draft FROM game_plan)` is correct.

### Example 3: Class Diversity Analysis (Rust)

```rust
// Recommended threshold: warn if a player has >= 70% of their pool from one class,
// OR if they are entirely missing a class that the opponent frequently targets.
fn compute_pool_gap(
    entries: &[ChampionPoolEntry],
    champion_classes: &HashMap<String, Vec<String>>,  // id -> tags
    opponent_champion_classes: &[String],             // flattened class list opponents play
) -> Vec<String> { // returns missing/dominant class warnings
    let mut class_counts: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        if let Some(tags) = champion_classes.get(&entry.champion) {
            for tag in tags {
                *class_counts.entry(tag.as_str()).or_default() += 1;
            }
        }
    }
    let total: usize = class_counts.values().sum();
    let all_classes = ["Fighter", "Mage", "Assassin", "Tank", "Marksman", "Support"];
    let mut warnings = Vec::new();
    for class in all_classes {
        let count = class_counts.get(class).copied().unwrap_or(0);
        if total > 0 && count as f64 / total as f64 >= 0.70 {
            warnings.push(format!("Pool dominated by {class} ({count}/{total})"));
        }
        if count == 0 && opponent_champion_classes.contains(&class.to_string()) {
            warnings.push(format!("No {class} coverage (opponents exploit this)"));
        }
    }
    warnings
}
```

### Example 4: Normalize Champion Name at Ingestion Point

```rust
// Utility function in src/server/data_dragon.rs or a new src/server/champion_normalize.rs
pub fn normalize_champion_name(input: &str, champions: &[Champion]) -> Option<String> {
    // Exact ID match
    if champions.iter().any(|c| c.id == input) {
        return Some(input.to_string());
    }
    // Case-insensitive display name match
    let lower = input.to_lowercase();
    champions.iter()
        .find(|c| c.name.to_lowercase() == lower)
        .map(|c| c.id.clone())
}
```

### Example 5: ChampionAutocomplete Enforcement Fix

```rust
// In champion_autocomplete.rs — change select_champion signature
let select_champion = move |champ: Champion| {
    value.set(champ.id.clone());           // store canonical ID
    set_filter_text.set(champ.name.clone()); // display human name
    set_open.set(false);
    if let Some(cb) = on_select {
        cb.run(champ.id);   // callback receives ID
    }
};
// In dropdown items:
on:mousedown=move |ev| {
    ev.prevent_default();
    select_champion(c.clone());  // pass Champion struct, not just name
}
```

---

## State of the Art

| Old Approach | Current Approach | Phase 2 Change |
|--------------|-----------------|----------------|
| `get_champion_stats_for_user` (per-user only) | Extends to team-wide aggregation | New `get_champion_performance_summary(team_id)` |
| Freeform champion string entry in autocomplete | Still freeform (display name stored) | Enforce ID at storage, display name in UI |
| No cross-table dashboard summary | Multiple individual queries per panel | Single `get_dashboard_summary(team_id)` batch |
| No pool gap analysis | N/A | New gap detection with Data Dragon class tags |

---

## Implementation Order

The correct sequencing within Phase 2:

1. **Champion name normalization** — update `ChampionAutocomplete` to store ID not display name; add startup migration; add `normalize_champion_name` utility
2. **New model structs** — add `DashboardSummary`, `ChampionPerformanceSummary`, `PoolGapWarning` to `src/models/`
3. **`get_dashboard_summary`** — batched 5-statement query + pool gap computation
4. **`get_champion_performance_summary`** — per-player and team-wide variants
5. **Integration tests** — `#[cfg(test)]` blocks in `db.rs` using in-memory SurrealDB

---

## Open Questions

1. **Migration completeness: Which tables have freeform champion names?**
   - Known: `champion_pool.champion`, `game_plan.our_champions[]`, `game_plan.enemy_champions[]`, `opponent_player.recent_champions[]`, `draft_action.champion`, `tree_node_action.champion`, `ban_priority.champion`
   - Unknown: Whether real data contains inconsistent forms (no production data to check)
   - Recommendation: Startup migration script covers all tables; log each update so operator can verify

2. **"Recent champions" from Riot API in opponent_player — these come from Riot, already using canonical forms?**
   - What we know: `store_matches` saves `m.champion` from Riot's API. The `riven` crate returns champion names as `String` — likely display names, not Data Dragon keys.
   - What's unclear: Whether `riven`'s champion field matches Data Dragon ID or display name.
   - Recommendation: Apply normalization at `store_matches` ingestion too; add `normalize_champion_name` call there.

3. **`data_dragon::fetch_champions()` caching**
   - What we know: Currently makes a live HTTP request every call.
   - What's unclear: Whether concurrent dashboard loads would overwhelm Data Dragon CDN.
   - Recommendation: Accept the per-call cost in Phase 2. Add a `OnceCell<Vec<Champion>>` cache in Phase 3 if it becomes a problem.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]` / `#[tokio::test]`) |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cargo test --features ssr --lib` |

Integration tests that require a live SurrealDB instance are run using `surrealdb::engine::local::Mem` (in-memory) — this is the pattern to use for Phase 2 db.rs integration tests.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `get_dashboard_summary` returns correct shape with seeded data | unit/integration | `cargo test --features ssr --lib -- db::tests::test_dashboard_summary` | ❌ Wave 0 |
| SC-1 | `get_dashboard_summary` returns `Ok(default)` with no data | unit/integration | `cargo test --features ssr --lib -- db::tests::test_dashboard_summary_empty` | ❌ Wave 0 |
| SC-2 | `get_champion_performance_summary` aggregates correctly | unit/integration | `cargo test --features ssr --lib -- db::tests::test_champion_perf_summary` | ❌ Wave 0 |
| SC-3 | `normalize_champion_name` maps display to ID correctly | unit | `cargo test --features ssr --lib -- data_dragon::tests::test_normalize` | ❌ Wave 0 |
| SC-4 | All summary queries return `Ok(empty)` when no data | unit/integration | Covered in SC-1 empty test | ❌ Wave 0 |
| SC-5 | Integration tests cover each new query function | integration | `cargo test --features ssr --lib` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Integration test helpers in `db.rs` tests mod — `create_test_db()` using `surrealdb::engine::local::Mem`
- [ ] `tests::test_dashboard_summary` — covers SC-1
- [ ] `tests::test_dashboard_summary_empty` — covers SC-1 empty case
- [ ] `tests::test_champion_perf_summary` — covers SC-2
- [ ] `data_dragon::tests::test_normalize` — covers SC-3

Note: `surrealdb::engine::local::Mem` (in-memory engine) is confirmed available in SurrealDB 3.x for test isolation.

---

## Sources

### Primary (HIGH confidence)
- Direct code reading of `src/server/db.rs` (3,285 lines) — established patterns for batching, Db* structs, aggregation
- Direct code reading of `src/server/data_dragon.rs` — Champion struct with `id` (canonical) and `tags` fields
- Direct code reading of `src/components/champion_autocomplete.rs` — current behavior storing display name
- Direct code reading of `schema.surql` — all table definitions, confirmed champion fields are `string` type
- `CLAUDE.md` rules 1-5 (SurrealDB patterns), 27-31 (query patterns), 40 (ORDER BY)

### Secondary (MEDIUM confidence)
- `02-CONTEXT.md` — user decisions that constrain implementation

### Tertiary (LOW confidence)
- `riven` crate champion name format assumption (display vs ID) — needs runtime verification

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in use, no new dependencies
- Architecture: HIGH — all new patterns extend confirmed existing patterns
- Pitfalls: HIGH — derived from direct code reading and CLAUDE.md rules
- Normalization scope: MEDIUM — existing data content unknown until migration script probes it

**Research date:** 2026-03-15
**Valid until:** 2026-06-15 (stable stack, no fast-moving dependencies)
