# Phase 11: Cross-Feature & Testing - Research

**Researched:** 2026-03-24
**Domain:** Analytics aggregation, PostGameLearning model extension, Riot API match auto-detection, Rust seed binary
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**XFEAT-02 Scope**
- D-12: XFEAT-02 (draft sidebar champion notes) is already satisfied by Phase 9's DRFT-05 implementation. Mark complete, exclude from Phase 11 work.

**Plan Effectiveness Metrics (XFEAT-01)**
- D-01: Add a `win_loss` outcome field (`win`/`loss`) to `PostGameLearning` model
- D-02: Add a `rating` field (1-5 integer stars) to `PostGameLearning`
- D-03: Auto-detect win/loss on post-game review creation: fetch recent Riot API matches, match against draft champions, auto-fill outcome
- D-04: Show a "Fetch result" button as fallback when game hasn't finished or wasn't auto-detected
- D-05: Effectiveness aggregates by BOTH strategy tag (top-level cards) AND individual game plan (table below)
- D-06: Clicking a game plan row expands inline (accordion) to show linked post-game reviews
- D-07: New dedicated page at `/analytics` with "Analytics" nav link (auth-required)
- D-08: Top row: strategy tag summary cards showing win rate, average rating, games played per tag
- D-09: Below cards: sortable table of individual game plans (name, tag, W-L record, average rating)
- D-10: No filters for v1.1
- D-11: Inline accordion expansion on row click

**Test Data Seeding (TEST-01)**
- D-13: Use real Riot accounts (user's own accounts + friends/alts)
- D-14: Rust binary seed script — clean slate: wipe test accounts, create 2 teams with 5 users each, link Riot accounts, populate all app data. Idempotent.
- D-15: Seed scope: champion pools with tiers + notes, 2-3 drafts per team with linked game plans, post-game reviews with win/loss + ratings, 1-2 opponent scouting profiles with 5-role player data
- D-16: Seed script fetches real champion mastery and match history from Riot API during seeding

### Claude's Discretion
- Exact star rating UI component (CSS stars, SVG, or emoji)
- Strategy card visual design (colors, icons per tag)
- Accordion animation/transition for plan row expansion
- Seed script CLI interface (args, flags, output format)
- How to handle Riot API rate limiting during seed (backoff strategy)
- Schema migration approach for new PostGameLearning fields (add with defaults for backward compat)

### Deferred Ideas (OUT OF SCOPE)
- Date range / time filters on analytics page
- Chart visualizations (bar charts, trend lines)
- Click strategy card to filter table to that tag
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| XFEAT-01 | Plan effectiveness view aggregates post-game review outcomes per game plan, showing strategy success rate | Model extension, analytics aggregation query, new `/analytics` page, accordion component pattern |
| XFEAT-02 | Draft sidebar shows champion pool notes when a pooled champion is selected (ALREADY COMPLETE via DRFT-05) | Verified complete in Phase 9 — no implementation required |
| TEST-01 | All legacy test accounts removed; 2 test teams with 5 real Riot-linked accounts each, populated with example data | Seed binary pattern, Riot API rate limiting, idempotent seeding strategy |
</phase_requirements>

## Summary

Phase 11 has three work streams. Two require implementation; one is a no-op.

**XFEAT-01** requires: (1) adding `win_loss: Option<String>` and `rating: Option<u8>` fields to the `PostGameLearning` model and all layers that touch it (schema.surql, DbPostGameLearning, save/update queries, post_game.rs form); (2) a server function that aggregates across `post_game_learning` and `game_plan` tables, joined by `game_plan_id`; and (3) a new `/analytics` page component with strategy tag cards, a sortable plan table, and inline accordion row expansion.

**XFEAT-02** is complete. Phase 9 DRFT-05 already implemented the Notes tab in the draft intel sidebar (lines ~943-1161 of `src/pages/draft.rs`). No work required.

**TEST-01** requires a Rust seed binary (`src/bin/seed.rs`) that: cleans up designated test accounts/teams by email prefix, creates 2 teams with 5 users each, links real Riot PUUIDs via the existing `get_puuid` function, populates champion pool entries, champion notes, drafts, game plans, post-game reviews (with win_loss/rating), and an opponent profile. The binary reuses `db::init_db()` and all existing `server::db` and `server::riot` functions directly.

**Primary recommendation:** Implement in this order: (1) model + schema migration, (2) analytics aggregation query + page, (3) win/loss auto-detect on post-game review form, (4) seed binary.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| surrealdb | 3.x | Analytics aggregation query | Already used; SurrealQL supports GROUP BY and math functions |
| leptos | 0.8 | Analytics page component | Project standard |
| riven | 2.x | Riot API for match auto-detect and seed data | Already integrated in `src/server/riot.rs` |
| tokio | 1.x | Async runtime for seed binary | Already in dev-dependencies |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_json | 1 | Seed binary: serialize review/plan data for DB insert | Existing pattern for server fn data passing |
| argon2 + password-hash | 0.5 | Seed binary: hash passwords for test users | Reuse existing `auth::hash_password()` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Rust bin in same crate | Standalone script in Python/shell | Rust bin reuses existing db/riot modules directly; no duplication |
| In-process aggregation in Rust | SurrealQL GROUP BY | SurrealQL aggregation is simpler, fewer round trips |

**Seed binary declaration in Cargo.toml:**
```toml
[[bin]]
name = "seed"
path = "src/bin/seed.rs"
required-features = ["ssr"]
```

Run with:
```bash
cargo run --features ssr --bin seed
```

## Architecture Patterns

### Recommended Project Structure (additions only)
```
src/
├── bin/
│   └── seed.rs              # NEW: seed binary, requires "ssr" feature
├── pages/
│   └── analytics.rs         # NEW: /analytics page component
models/game_plan.rs           # EDIT: add win_loss + rating to PostGameLearning
server/db.rs                  # EDIT: analytics query, update save/update functions
schema.surql                  # EDIT: DEFINE FIELD IF NOT EXISTS for two new fields
app.rs                        # EDIT: add /analytics Route
components/nav.rs             # EDIT: add Analytics nav link
e2e/tests/pages.spec.ts       # EDIT: add /analytics to AUTHED_PAGES
```

### Pattern 1: PostGameLearning Model Extension

Add two optional fields to the shared struct. `serde(default)` ensures backward-compat deserialization of records lacking these fields (same approach used for `DraftAction.role` in Phase 8):

```rust
// src/models/game_plan.rs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostGameLearning {
    // ... existing fields ...
    #[serde(default)]
    pub win_loss: Option<String>,   // "win" | "loss" | None
    #[serde(default)]
    pub rating: Option<u8>,         // 1-5 stars
}
```

Schema additions (idempotent per project rule 30):
```sql
DEFINE FIELD IF NOT EXISTS win_loss ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS rating ON post_game_learning TYPE option<int>;
```

### Pattern 2: Analytics Aggregation Query

The analytics page needs two aggregated views:

**Strategy tag summary** (for top cards): group post-game reviews by the `win_condition_tag` of their linked game plan.

```sql
SELECT
  gp.win_condition_tag AS tag,
  count() AS games_played,
  math::sum(IF pgl.win_loss = 'win' THEN 1 ELSE 0 END) AS wins,
  math::mean(pgl.rating) AS avg_rating
FROM post_game_learning AS pgl
JOIN game_plan AS gp ON pgl.game_plan_id = gp.id
WHERE pgl.team = type::record('team', $team_key)
  AND pgl.game_plan_id IS NOT NONE
GROUP BY gp.win_condition_tag
```

**Note:** SurrealDB 3.x JOIN syntax may require subquery or multi-query approach. The safer alternative (avoiding JOIN) is to fetch all game plans + reviews in two queries and join in Rust. This is the confirmed pattern elsewhere in the codebase (see `save_post_game_learning` referencing `game_plan_id` as a plain `Option<String>` — no SurrealDB relation, just a string ID).

**Recommended approach — two queries, Rust aggregation:**

```rust
// In db.rs: new analytics query function
pub struct StrategyTagSummary {
    pub tag: String,
    pub games_played: usize,
    pub wins: usize,
    pub losses: usize,
    pub avg_rating: Option<f32>,
}

pub struct GamePlanEffectiveness {
    pub plan_id: String,
    pub plan_name: String,
    pub tag: Option<String>,
    pub wins: usize,
    pub losses: usize,
    pub avg_rating: Option<f32>,
    pub reviews: Vec<PostGameLearning>,
}

pub async fn get_analytics(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<(Vec<StrategyTagSummary>, Vec<GamePlanEffectiveness>)>
```

Internally: fetch all `game_plan` rows for team, fetch all `post_game_learning` rows for team that have `game_plan_id IS NOT NONE`, build `HashMap<plan_id, (plan, reviews)>`, aggregate in Rust. This avoids JOIN syntax ambiguity with SurrealDB 3.x and follows the established multi-query pattern (rule 29).

Both aggregation structs need `Serialize, Deserialize` (they cross the server-fn boundary as JSON).

### Pattern 3: Analytics Page Component

Follow the standard protected page pattern (same as all other pages in `src/pages/`):

```rust
// src/pages/analytics.rs
#[server]
pub async fn get_analytics_data() -> Result<AnalyticsPayload, ServerFnError> { ... }

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    // get_current_user() check + redirect (same as all protected pages)
    // Resource::new(|| (), |_| get_analytics_data())
    // Suspense with tag cards + plan table
}
```

**Accordion pattern for plan row expansion:**

Use `RwSignal<Option<String>>` to track the currently-open plan ID. Clicking a row sets/toggles it; the expanded content is conditionally rendered:

```rust
let open_plan: RwSignal<Option<String>> = RwSignal::new(None);

// In each row:
let plan_id_for_click = plan.plan_id.clone();
let plan_id_for_expand = plan.plan_id.clone();
view! {
    <tr on:click=move |_| {
        let id = plan_id_for_click.clone();
        open_plan.update(|current| {
            if current.as_deref() == Some(&id) {
                *current = None;
            } else {
                *current = Some(id);
            }
        });
    }>
        // ... cells ...
    </tr>
    {move || if open_plan.get().as_deref() == Some(&plan_id_for_expand) {
        view! { <tr><td colspan="4">/* reviews */</td></tr> }.into_any()
    } else {
        view! { }.into_any()
    }}
}
```

**Star rating display** (Claude's discretion): Use Unicode filled/empty circles `●○` or `★☆`. Avoid SVG per-star for simplicity. Render server-fn returned `Option<f32>` rounded to nearest 0.5.

**Strategy tag card colors** (Claude's discretion): Map known tags ("teamfight", "split-push", "poke", "engage", "protect-the-adc") to Tailwind accent variants. Unknown tags fall back to a neutral color.

### Pattern 4: Win/Loss Auto-Detection

When creating a post-game review in `post_game.rs`, if the user has a linked Riot account (riot_puuid) and has selected a draft, auto-detect the match outcome:

1. Fetch recent match IDs for the user's PUUID (existing `fetch_match_history` function).
2. Extract the draft's picked champions from `draft.actions` (already fetched for the "link draft" dropdown).
3. Find a match where the user's champion appears in `our_champions` from the draft.
4. If found: set `win_loss` to `"win"` or `"loss"` from `MatchData.win`.
5. Store `match_riot_id` on the review.

The existing `get_recent_match_ids()` server function in `post_game.rs` already fetches match IDs — it can be extended or a new `auto_detect_outcome(draft_id: String) -> Result<Option<String>, ServerFnError>` can be added.

**Fallback "Fetch result" button**: Calls the same detection logic on demand. Only shown when:
- `win_loss` is `None` AND `game_plan_id` is set (we have a draft to match against).

### Pattern 5: Seed Binary

Rust binary in `src/bin/seed.rs` with `required-features = ["ssr"]`. It has direct access to all `crate::server::{db, riot, auth}` functions.

**Idempotency strategy**: Delete by known email prefix (e.g. `seed-team-a-*@example.com`) before re-creating. SurrealDB query:
```sql
DELETE user WHERE email STARTS WITH 'seed-team-'
```
Cascade-delete orphaned records (teams, members, drafts, etc.) after user deletion, or include explicit delete queries for each table scoped to the seed team IDs.

**Riot API rate limiting**: The Riot development API key has a 20 req/1s, 100 req/2min limit. The seed script fetches mastery + recent matches for 10 users. With sequential requests and a small `tokio::time::sleep(Duration::from_millis(100))` between calls, this fits within limits. The script should be tolerant of rate limit errors: wrap each Riot call in a retry loop with exponential backoff (2^n * 100ms, max 3 retries).

**Seed data structure** (per D-15):

For each of 2 teams:
- 5 users with hashed passwords, linked Riot PUUIDs via `get_puuid()`
- Champion pool entries: top 5 champions from mastery, assigned roles, tiers
- Champion notes: 1-2 notes per champion (matchup or power spike type)
- 2-3 drafts with 20 pick/ban actions each
- 1-2 game plans linked to the drafts, with `win_condition_tag` set
- 1-2 post-game reviews with `win_loss` and `rating` values
- 1 opponent profile with 5 role players populated from Riot API

**Environment**: Seed binary reads `RIOT_API_KEY` and `SURREAL_DATA_DIR` from `.env` (same as server). Uses `db::init_db()` which opens the same SurrealKV file. The server must NOT be running when the seed binary runs (both can't hold the SurrealKV file lock simultaneously).

**CLI interface** (Claude's discretion): Simple `--dry-run` flag that prints what would be created without writing. Default is to run and print progress.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Analytics aggregation query | Custom SurrealQL GROUP BY | Two-query + Rust HashMap aggregation | Avoids SurrealDB JOIN ambiguity; established codebase pattern |
| Password hashing in seed | Custom argon2 calls | `crate::server::auth::hash_password()` | Reuse existing tested function |
| Riot PUUID lookup in seed | Custom reqwest call | `crate::server::riot::get_puuid()` | Already handles error types and routing |
| Match history in seed | New API client | `crate::server::riot::fetch_match_history()` | Full implementation already exists |
| DB init in seed | New SurrealKV open | `crate::server::db::init_db()` | Applies schema, handles migrations |
| Accordion state | External state library | `RwSignal<Option<String>>` | Leptos signal is sufficient; no library needed |

**Key insight:** The seed binary's greatest advantage is that it calls `crate::server::db` and `crate::server::riot` directly. There is no HTTP layer — it's just async Rust calling the same functions the server uses.

## Common Pitfalls

### Pitfall 1: DbPostGameLearning Missing New Fields
**What goes wrong:** Adding `win_loss` and `rating` to `PostGameLearning` but not to `DbPostGameLearning` in `db.rs` causes deserialization failures when reading records that have these fields set.
**Why it happens:** There are two struct layers — the app model and the DB struct. Both must be updated.
**How to avoid:** Update `DbPostGameLearning`, its `From` impl, the `save_post_game_learning` query string, and `update_post_game_learning` query string in the same task.
**Warning signs:** Compiler error in the `From<DbPostGameLearning>` impl when `PostGameLearning` has new required fields that `DbPostGameLearning` doesn't.

### Pitfall 2: `serde(default)` Missing on New PostGameLearning Fields
**What goes wrong:** Existing records in the DB don't have `win_loss` or `rating`. Deserializing them fails unless `#[serde(default)]` is on the new fields.
**Why it happens:** SurrealDB SCHEMAFULL tables store what's defined. Existing rows have `NULL`/absent for new `option<>` fields. Without `serde(default)`, serde panics on absent keys.
**How to avoid:** Always use `#[serde(default)]` on new optional fields — same as the `DraftAction.role` fix in Phase 8 (STATE.md decision).

### Pitfall 3: Seed Binary Conflicts with Running Server
**What goes wrong:** Running the seed binary while `cargo leptos watch` is running causes SurrealKV file lock contention and either the seed or the server crashes.
**Why it happens:** SurrealKV uses an exclusive file lock on the data directory.
**How to avoid:** Document clearly that the server must be stopped before running `cargo run --features ssr --bin seed`. The seed binary can print a clear error message if DB init fails ("Is the server running?").

### Pitfall 4: ORDER BY on Non-Selected Field in SurrealDB 3.x
**What goes wrong:** Analytics query uses `ORDER BY` on a field not included in a partial `SELECT`. SurrealDB 3.x rejects this.
**Why it happens:** SurrealDB 3.x enforces that `ORDER BY` field must be in the SELECT (rule 40 in CLAUDE.md).
**How to avoid:** Either use `SELECT *` or include every `ORDER BY` field in the `SELECT` list.

### Pitfall 5: Accordion View Type Mismatch
**What goes wrong:** `if open_plan.get()...` arms return different view types (one has `<tr>...</tr>`, the other is empty), causing a Leptos compile error.
**Why it happens:** Leptos `{move || if/else}` requires structurally identical view types or `.into_any()` on each arm (rule 19 in CLAUDE.md).
**How to avoid:** Always call `.into_any()` on each arm when the views are structurally different.

### Pitfall 6: Riot API Rate Limit in Seed
**What goes wrong:** Seeding 10 users × (mastery call + 20 match IDs + up to 20 match detail calls) = ~220 API calls. At 100 req/2min dev key limit, this takes at least 2.5 minutes without rate limiting.
**Why it happens:** Development API keys have strict per-minute limits.
**How to avoid:** Use sequential calls with small delays between users (100ms+). Wrap calls in retry-with-backoff. For seed data quality, prefer `fetch_champion_masteries` over `fetch_match_history` (1 call vs. 21) for champion pool seeding — mastery data is sufficient.

### Pitfall 7: Analytics Page Returns Error for Users Without a Team
**What goes wrong:** `get_analytics_data()` server fn calls `get_user_team_id()`. If user has no team, returning `Err` breaks the Suspense and shows an error banner.
**Why it happens:** Consistent pattern issue seen across multiple pages (see rule 44 in CLAUDE.md and MEMORY.md bug #6).
**How to avoid:** Return `Ok(empty_analytics_payload)` when `get_user_team_id()` returns `None`.

## Code Examples

### Example 1: Updated save_post_game_learning with new fields

```rust
// Source: derived from existing pattern in src/server/db.rs lines 1833-1863
pub async fn save_post_game_learning(
    db: &Surreal<Db>,
    learning: PostGameLearning,
) -> DbResult<String> {
    let team_key = learning.team_id.strip_prefix("team:").unwrap_or(&learning.team_id).to_string();
    let created_by_key = learning.created_by.strip_prefix("user:").unwrap_or(&learning.created_by).to_string();
    let mut response = db.query(
        "CREATE post_game_learning SET team = type::record('team', $team_key), \
         match_riot_id = $match_riot_id, game_plan_id = $game_plan_id, draft_id = $draft_id, \
         what_went_well = $what_went_well, improvements = $improvements, \
         action_items = $action_items, open_notes = $open_notes, \
         created_by = type::record('user', $created_by_key), \
         win_loss = $win_loss, rating = $rating"
    )
    // ... binds including .bind(("win_loss", learning.win_loss)).bind(("rating", learning.rating))
    .await?;
    // ... take(0)?
}
```

### Example 2: Seed binary skeleton

```rust
// src/bin/seed.rs
#![cfg(feature = "ssr")]

use lol_team_companion::server::{auth, db, riot};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let data_dir = std::env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let surreal = db::init_db(&data_dir).await.expect("DB init failed — is the server running?");

    // Clean up existing seed data
    println!("Cleaning up existing seed accounts...");
    surreal.query("DELETE user WHERE string::starts_with(email, 'seed-')").await.unwrap().check().unwrap();

    // Create Team A
    let team_a_users = create_seed_team(&surreal, "Alpha", "EUW").await;
    // Create Team B
    let team_b_users = create_seed_team(&surreal, "Beta", "EUW").await;

    println!("Seed complete.");
}
```

### Example 3: Analytics aggregation in Rust (two-query approach)

```rust
// src/server/db.rs — new function
pub async fn get_analytics(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<(Vec<StrategyTagSummary>, Vec<GamePlanEffectiveness>)> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Single DB round-trip: fetch both tables
    let mut r = db.query(
        "SELECT * FROM game_plan WHERE team = type::record('team', $team_key); \
         SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key) AND game_plan_id IS NOT NONE;"
    )
    .bind(("team_key", team_key))
    .await?;

    let plans: Vec<DbGamePlan> = r.take(0).unwrap_or_default();
    let reviews: Vec<DbPostGameLearning> = r.take(1).unwrap_or_default();

    // Build plan map + aggregate in Rust
    // ...
}
```

### Example 4: New route and nav link

```rust
// src/app.rs — add inside Routes:
<Route path=path!("/analytics") view=AnalyticsPage />

// src/components/nav.rs — add in nav_links alongside existing auth-gated links:
<A href="/analytics" attr:class=cls_analytics on:click=move |_| close_link.run(())>
    "Analytics"
</A>
```

### Example 5: Leptos accordion with into_any()

```rust
// In analytics.rs — plan table row expansion
{move || {
    if open_plan.get().as_deref() == Some(&plan_id_expand) {
        view! {
            <tr class="bg-surface/30">
                <td colspan="4" class="p-4">
                    /* inline review list */
                </td>
            </tr>
        }.into_any()
    } else {
        view! { <></> }.into_any()
    }
}}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `type::thing()` for record IDs | `type::record()` | SurrealDB 2.x | All queries must use new form |
| Single query with JOIN | Two queries + Rust aggregation | Established pattern | Safer deserialization, avoids JOIN ambiguity |
| `#[cfg(test)]` only binary compilation | `[[bin]]` with `required-features` | Cargo standard | Seed binary compiles only with `ssr` feature |

## Open Questions

1. **Real Riot account credentials**
   - What we know: D-13 says use real accounts (user's own + friends/alts)
   - What's unclear: The planner cannot know which Riot IDs to hardcode; the implementer must provide them at seed time
   - Recommendation: Seed binary reads from a `seed_accounts.toml` (gitignored) or accepts `--team-a-accounts="name#tag,..."` CLI flags. Avoid hardcoding Riot IDs in source.

2. **Cascade delete for seed cleanup**
   - What we know: Deleting `user` records leaves orphaned `team_member`, `champion_pool`, `draft`, etc. records
   - What's unclear: SurrealDB 3.x doesn't auto-cascade on `DELETE user`. Manual cascade is needed.
   - Recommendation: Delete in reverse dependency order — `post_game_learning`, `game_plan`, `draft_action`, `draft`, `champion_note`, `champion_pool`, `team_member`, then `team`, then `user` — all filtered by the seed email prefix or known team IDs.

3. **Analytics aggregation structs serialization**
   - What we know: Server fn return types cross the wire as JSON
   - What's unclear: `StrategyTagSummary` and `GamePlanEffectiveness` are new types that need `Serialize + Deserialize`
   - Recommendation: Add these to `src/models/game_plan.rs` since they derive from game plan data. They must compile for both SSR and WASM (shared module).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| RIOT_API_KEY env var | Seed binary, win/loss auto-detect | User-provided | — | Auto-detect skipped if key absent; seed skips Riot calls |
| SurrealKV data dir | Seed binary | ✓ (same as server) | — | — |
| cargo | Seed binary compilation | ✓ | nightly | — |
| Riot dev API key limits | Seed data quality | User-dependent | 20/1s, 100/2min | Use sequential calls + delays |

**Missing dependencies with no fallback:**
- Real Riot account IDs/tags — the seed binary cannot fabricate PUUIDs; the user must provide accounts. Plan must accommodate this as an implementation-time input.

**Missing dependencies with fallback:**
- `RIOT_API_KEY` absent: auto-detect in post-game form should degrade gracefully (show manual win/loss selector without attempting API call). Seed binary should skip mastery/match fetch and use static champion pool data instead.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + Playwright |
| Config file | Implicit (Cargo.toml) + `e2e/playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cargo test --features ssr && cd e2e && npx playwright test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| XFEAT-01 | PostGameLearning model with win_loss + rating round-trips JSON | unit | `cargo test --features ssr --lib -- game_plan::tests` | ✅ `src/models/game_plan.rs` (add to existing test block) |
| XFEAT-01 | Analytics DB query returns aggregated data for a team | integration | `cargo test --features ssr --test db_game_plans` | ❌ Wave 0 — new test file |
| XFEAT-01 | /analytics page loads without JS errors | e2e smoke | `cd e2e && npx playwright test pages.spec.ts` | ✅ (need to add /analytics entry) |
| TEST-01 | Seed binary compiles with ssr feature | compile | `cargo check --features ssr` | ❌ Wave 0 — file doesn't exist yet |
| XFEAT-02 | No work required (already satisfied) | — | — | — |

### Sampling Rate
- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** `cargo test --features ssr` full suite + e2e pages.spec.ts green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/db_game_plans.rs` — covers analytics aggregation query (new integration test file)
- [ ] `src/bin/seed.rs` — created as part of TEST-01 implementation, not a test gap per se

*(Existing `src/models/game_plan.rs` test block already exists; new round-trip tests for win_loss/rating fields are added to it, not a gap.)*

## Project Constraints (from CLAUDE.md)

All of the following directives apply to this phase's implementation:

- **SurrealDB:** Use `type::record('table', $key)` (not `type::thing()`); strip table prefix before binding; `.check()` after writes; `take(0).unwrap_or_default()` for list reads; `DEFINE FIELD IF NOT EXISTS` for all schema additions
- **Leptos 0.8:** `into_any()` for divergent view branches; `get_untracked()` in event handlers; `prop:value` for controlled inputs; `attr:class` on `<A>`; `ActionForm` has no `class` prop
- **Server functions:** DB via `use_context::<Arc<Surreal<Db>>>()`, not `axum::extract::State`; SSR-only `use` inside server fn body; server fns before the `#[component]` that calls them
- **Protected pages:** Fetch `get_current_user()` on mount, redirect to `/auth/login` via `window.location().set_href()` if `None` (client-side only, `#[cfg(feature = "hydrate")]`)
- **Empty-state not error:** Return `Ok(empty_payload)` when `get_user_team_id()` returns `None`
- **WASM safety:** No `.unwrap()` in event handlers or WASM code; use `if let Some(...)` or `.unwrap_or_default()`
- **Theming:** Use semantic color tokens (`bg-surface`, `text-primary`, `border-divider`, etc.), not hardcoded colors
- **Recursion limit:** Both `src/lib.rs` and `src/main.rs` have `#![recursion_limit = "512"]` — do not lower
- **BFD linker:** `.cargo/config.toml` forces `bfd`; do not override
- **Seed binary:** `required-features = ["ssr"]` in `[[bin]]` declaration; cannot run while server holds SurrealKV lock
- **E2e smoke test:** Add `/analytics` to `AUTHED_PAGES` array in `e2e/tests/pages.spec.ts`
- **ORDER BY only on selected fields** (rule 40): Include `ORDER BY` field in SELECT or use `SELECT *`

## Sources

### Primary (HIGH confidence)
- `src/models/game_plan.rs` — PostGameLearning struct, existing fields, test patterns
- `src/server/db.rs` — DbPostGameLearning, save/update functions, init_db, aggregation patterns
- `src/server/riot.rs` — fetch_match_history, fetch_champion_masteries, get_puuid signatures
- `schema.surql` — post_game_learning table definition, existing field patterns
- `CLAUDE.md` — All critical patterns and gotchas (rules 1-57)
- `.planning/phases/11-cross-feature-testing/11-CONTEXT.md` — Locked decisions

### Secondary (MEDIUM confidence)
- `src/pages/post_game.rs` — Existing server fns (list_reviews, create_review, get_recent_match_ids) — shows integration patterns
- `src/app.rs` — Route registration pattern
- `src/components/nav.rs` — Nav link pattern
- `e2e/tests/pages.spec.ts` — AUTHED_PAGES pattern for smoke test addition
- `Cargo.toml` — `[[bin]]` section pattern for seed binary declaration

### Tertiary (LOW confidence)
- SurrealQL GROUP BY aggregation — not tested against SurrealDB 3.x; using Rust aggregation approach as primary recommendation to avoid uncertainty

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in use; no new dependencies needed
- Architecture: HIGH — all patterns derived from existing codebase
- Pitfalls: HIGH — most derived from existing CLAUDE.md rules and MEMORY.md documented bugs
- Analytics aggregation approach: MEDIUM — Rust aggregation recommended over SurrealQL GROUP BY to avoid untested JOIN/GROUP BY syntax in SurrealDB 3.x

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable stack, low churn expected)
