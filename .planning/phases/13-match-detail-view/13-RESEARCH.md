# Phase 13: Match Detail View - Research

**Researched:** 2026-03-25
**Domain:** Leptos 0.8 dynamic routing, Riot match-v5 API (all participants + timeline), SurrealDB on-demand caching, Data Dragon item images
**Confidence:** HIGH

## Summary

This phase adds a dedicated `/match/:id` route that shows the full 10-player scoreboard, a visual timeline, and a personal performance breakdown for any match in the user's history. The critical architectural decision (locked in CONTEXT.md) is that all data loads on-demand: the server function first checks a DB cache, and only calls the Riot API on a miss. This avoids rate-limit pressure during bulk sync.

The existing codebase already calls `match_v5().get_match()` in `riot.rs`, so extending it to extract all 10 participants is straightforward — the riven `Participant` struct already exposes `item0`–`item6`, `gold_earned`, `team_position`, `summoner_name`, and `total_damage_dealt_to_champions`. The timeline requires a second Riot call (`match_v5().get_timeline()`) which returns structured `EventsTimeLine` objects typed by a `type` string. Both calls should be batched per match and stored in new `match_detail` and `match_timeline_event` DB tables.

For the frontend, the Leptos 0.8 pattern for dynamic route params is `use_params_map()` returning a `Memo<ParamsMap>`, keyed on `"id"`. The component drives a `Resource` reactive to that memo, triggering the server function with the match id. All UI components (ScoreboardTable, TimelineBar, TimelineFilterBar, PerformanceBarChart) are inlined in `src/pages/match_detail.rs` as specified in the UI-SPEC.

**Primary recommendation:** One server function `fetch_match_detail(match_id: String)` handles both cache-hit and API-miss paths; it returns a single rich struct covering scoreboard + timeline + stats. No separate client-side calls needed.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Navigation & Layout**
- D-01: Dedicated route at `/match/:id` — full-page match detail view. Back button returns to history.
- D-02: Classic two-team table layout — Blue team table on top, Red team table below. Columns: champion icon, summoner name, KDA, items (6 icon slots), damage dealt, gold earned, vision score.
- D-03: User's own row highlighted with a subtle accent background.
- D-04: Item icons use Data Dragon images (small squares), with tooltip on hover showing item name.

**Timeline Presentation**
- D-05: Visual timeline bar — horizontal bar with event markers placed at timestamps. Color-coded by team color and event type.
- D-06: Hover shows tooltip; click expands a detail section below the bar. Both behaviors on the same markers.
- D-07: Event types shown: objectives (dragons, baron, herald), tower kills, champion kills (first blood, multikills, aces), inhibitors, ward placements, recalls, and teamfight indicators (4+ participants).
- D-08: Filter toggle buttons above the timeline bar to show/hide event categories.
- D-09: User's own events get a special highlight indicator.

**Performance Breakdown**
- D-10: Horizontal bar charts with game average as a marker/line.
- D-11: Metrics: damage share %, vision score, CS/min, gold earned.
- D-12: Comparison defaults to game average; toggle to role-based (vs lane opponent) when available.

**Loading & Caching UX**
- D-13: Skeleton placeholders while fetching.
- D-14: No cache indicator.
- D-15: Error banner + retry button on API failure.

### Claude's Discretion
- Whether performance section includes short text verdicts ("Above average") alongside bar charts, or numbers only
- Exact icon shapes/symbols for different timeline event types
- How the special highlight for user's events is styled
- Teamfight detection heuristic details (4+ participants within time window)
- How role detection works for role-based comparison toggle

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MATCH-01 | User can view all 10 participants in a match (KDA, items, damage, gold, vision) | riven `Participant` struct has all fields; `match_v5().get_match()` already used; need full participant extraction, not just current user |
| MATCH-02 | User can view match timeline events (dragons, barons, towers, kills) | riven `match_v5().get_timeline()` returns `Timeline` with `FramesTimeLine` containing `EventsTimeLine` objects typed by string `type` field |
| MATCH-03 | User can see personal performance breakdown (damage share, vision score, CS vs game average) | Derivable from `Participant` data: sum all 10 players' values, compute percentages; role comparison via `team_position` field |
| MATCH-04 | Match detail loads on-demand and caches in DB after first fetch | New `match_detail` and `match_timeline_event` tables in schema.surql; cache-check-then-fetch pattern in server fn |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| riven | 2.78.0 (in Cargo.lock) | Riot API client | Already used throughout project |
| surrealdb | 3.x (project-wide) | DB caching for match detail | Already used; `match` table exists |
| leptos_router | 0.7.8 (transitive) | Dynamic route params via `use_params_map()` | Project standard |
| tailwindcss v4 | standalone binary | Styling | Project standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | existing | epoch ms → ISO datetime conversion | Already used in `store_matches_with_synced_by` |
| surrealdb-types-derive | existing | `#[derive(SurrealValue)]` on DB structs | Required for all new DbStruct types |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Storing timeline events as separate rows | Store as a single JSON string field | JSON blob is simpler but makes filtering by event type in DB impossible; separate rows enable future queries |
| Caching full riven structs | Store extracted fields only | riven structs are 500KB+ of timeline frames; extract only the event fields needed (timestamp, type, team_id, participant_id, monster type) |

**Installation:** No new dependencies needed. All required crates are already in Cargo.toml.

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── pages/
│   └── match_detail.rs      # New page: MatchDetailPage + all sub-components + server fns
├── models/
│   └── match_data.rs        # Extended with MatchParticipant, MatchDetail, TimelineEvent
├── server/
│   ├── riot.rs              # Extended with fetch_full_match_detail()
│   └── db.rs                # Extended with get_cached_match_detail(), store_match_detail()
└── app.rs                   # New route: /match/:id
schema.surql                 # New tables: match_detail, match_timeline_event
```

### Pattern 1: Dynamic Route Parameter Extraction
**What:** Use `use_params_map()` to read `:id` from `/match/:id`
**When to use:** All dynamic routes with path params in this project

```rust
// Source: leptos_router 0.7.8 docs (lib.rs example)
let params = use_params_map();
let match_id = move || params.read().get("id").unwrap_or_default();
let detail = Resource::new(match_id, |id| fetch_match_detail(id));
```

**Route definition in app.rs:**
```rust
// Source: project pattern from app.rs
use crate::pages::match_detail::MatchDetailPage;
<Route path=path!("/match/:id") view=MatchDetailPage />
```

### Pattern 2: Cache-Check-Then-Fetch Server Function
**What:** Single server fn checks DB cache; calls Riot API only on miss; stores and returns result
**When to use:** All on-demand expensive API calls (MATCH-04 requirement)

```rust
// Pattern derived from store_matches_with_synced_by in db.rs
#[server]
pub async fn fetch_match_detail(match_id: String) -> Result<MatchDetail, ServerFnError> {
    use crate::server::{db, riot};
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if match_id.is_empty() {
        return Err(ServerFnError::new("No match ID provided"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // Cache hit path
    if let Some(cached) = db::get_cached_match_detail(&surreal, &match_id).await
        .map_err(|e| ServerFnError::new(e.to_string()))? {
        return Ok(cached);
    }

    // Cache miss: fetch from Riot API
    if !riot::has_api_key() {
        return Err(ServerFnError::new("Riot API key not configured."));
    }

    let platform = riot::platform_route_from_str(
        user.riot_region.as_deref().unwrap_or("EUW")
    );
    let detail = riot::fetch_full_match_detail(&match_id, platform).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Store in cache (fire-and-forget OK; error logged but not surfaced)
    if let Err(e) = db::store_match_detail(&surreal, &detail).await {
        tracing::warn!("Failed to cache match detail for {match_id}: {e}");
    }

    Ok(detail)
}
```

### Pattern 3: Riven Full Match Data Extraction
**What:** Extract all 10 participants + timeline events from `match_v5` responses
**When to use:** New `fetch_full_match_detail()` in `riot.rs`

Key riven fields per `Participant`:
- `p.participant_id: i32` — 1-indexed; 1-5 = Blue team, 6-10 = Red team
- `p.puuid: String` — for identifying the current user's row
- `p.summoner_name: String` — display name
- `p.champion_name: String` — for Data Dragon champion icon
- `p.kills, p.deaths, p.assists, p.cs (total_minions_killed + neutral_minions_killed)`
- `p.item0`–`p.item6: i32` — item IDs (0 = empty slot)
- `p.gold_earned: i32`
- `p.total_damage_dealt_to_champions: i32`
- `p.vision_score: i32`
- `p.win: bool`
- `p.team_position: String` — "TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY" (blank if unreliable)

Timeline `EventsTimeLine` key fields:
- `event.r#type: String` — "CHAMPION_KILL", "BUILDING_KILL", "ELITE_MONSTER_KILL", "WARD_PLACED", "GAME_END"
- `event.timestamp: i64` — milliseconds since game start
- `event.killer_id: Option<i32>` — participant_id of killer
- `event.victim_id: Option<i32>` — participant_id of victim
- `event.team_id: Option<Team>` — team that scored the objective
- `event.killer_team_id: Option<Team>` — for building/monster kills
- `event.monster_type: Option<String>` — "BARON_NASHOR", "HORDE" (Herald), "DRAGON"
- `event.monster_sub_type: Option<String>` — dragon element (FIRE, EARTH, WATER, AIR, CHEMTECH, etc.)
- `event.building_type: Option<String>` — "TOWER_BUILDING", "INHIBITOR_BUILDING"
- `event.multi_kill_length: Option<i32>` — for multikill events
- `event.assisting_participant_ids: Option<Vec<i32>>` — for teamfight detection

### Pattern 4: Teamfight Detection Heuristic
**What:** Group CHAMPION_KILL events within a 10-second sliding window; if 4+ unique participant IDs (killers + victims + assists), flag as teamfight
**When to use:** Building `TimelineEvent::Teamfight` variants server-side before caching

```rust
// Server-side post-processing after timeline fetch
fn detect_teamfights(events: &[RawEvent]) -> Vec<TimelineEvent> {
    // Collect kill events with all involved participants
    // Slide 10s window; if window contains 4+ unique participant IDs → teamfight
    // Emit single Teamfight event at center timestamp, suppress individual kills in that window
}
```

### Pattern 5: Performance Breakdown Computation
**What:** Compute damage share %, CS/min, and comparison values server-side before returning
**When to use:** When building the `MatchDetail` struct to return to the client

```rust
let total_damage: i32 = participants.iter().map(|p| p.damage).sum();
let damage_share_pct = (user_participant.damage as f32 / total_damage as f32) * 100.0;
let game_duration_min = game_duration_secs as f32 / 60.0;
let cs_per_min = user_participant.cs as f32 / game_duration_min;
let avg_vision = participants.iter().map(|p| p.vision_score as f32).sum::<f32>() / 10.0;
let avg_gold = participants.iter().map(|p| p.gold_earned as f32).sum::<f32>() / 10.0;

// Lane opponent: find participant with same team_position on opposite team
let lane_opponent = participants.iter().find(|p| {
    p.team_id != user_participant.team_id
    && p.team_position == user_participant.team_position
    && !user_participant.team_position.is_empty()
});
```

### Anti-Patterns to Avoid
- **Storing raw timeline JSON blobs:** The full timeline response is 300–500KB per match (per-minute frames). Extract only needed events and discard frames — documented in REQUIREMENTS.md out-of-scope section.
- **Fetching timeline from the client:** `get_timeline` must happen server-side only; timeline data is SSR-only.
- **Using `attr:value` on bar width:** Use `style="width: {pct}%"` string for percentage-based bar fills — `attr:` cannot set inline style dynamically.
- **Calling `match_v5().get_match()` again if scoreboard data already exists:** The `match` table already stores `match_id`. The new `match_detail` table stores the full participant data. Check both before hitting API.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Item name tooltips | Custom JS tooltip component | Native HTML `title` attribute on `<img>` | D-04 specifies this; consistent with champion icon pattern |
| Route params parsing | Manual URL parsing | `use_params_map()` from leptos_router | Project standard; `params.read().get("id")` is one line |
| Champion icon URLs | Custom CDN lookup | `format!(".../cdn/15.6.1/img/champion/{name}.png")` | Already used in stats.rs `champion_icon_url()` |
| Item icon URLs | Custom lookup | `format!(".../cdn/{version}/img/item/{item_id}.png")` | Data Dragon CDN; same pattern as champion icons |
| Percentage bar widths | SVG or canvas | `style="width: {pct}%"` on a `<div>` inside a track `<div>` | Trivial CSS, no library needed |
| Skeleton loading | Custom shimmer animation | `SkeletonCard` from `src/components/ui.rs` | Already exists with `animate-pulse bg-elevated` |
| Error display | Custom error component | `ErrorBanner` from `src/components/ui.rs` | Already exists with correct styling |

**Key insight:** All complex UI behaviors in this phase reduce to CSS positioning (timeline markers via absolute + percentage left), reactive signals (filter toggles, selected event, comparison mode), and Rust pattern matching (event type classification). No external JS components needed.

---

## Common Pitfalls

### Pitfall 1: item6 is the Trinket/Ward Slot
**What goes wrong:** The scoreboard shows 7 items, not 6. The `item6` field in riven `Participant` is slot 6 (trinket/ward item), distinct from the 6 main item slots (`item0`–`item5`).
**Why it happens:** The Riot API uses 0-indexed item slots; `item6` is the ward slot at position 7 in the UI.
**How to avoid:** When building `MatchParticipant.items: [i32; 7]`, include `item6` as the last slot. The UI-SPEC shows 6 slots for main items — if trinket should be omitted, store only `item0`–`item5`. Decision: follow UI-SPEC (6 slots) and omit `item6` (ward/trinket is low-value display).
**Warning signs:** ScoreboardTable shows 7 columns of items instead of 6.

### Pitfall 2: Timeline Timestamps are Milliseconds, Not Seconds
**What goes wrong:** Timeline event timestamps from `EventsTimeLine.timestamp` are in milliseconds since game start. `game_duration` in the `match` table is in seconds. Computing position percentage with mismatched units places markers at 0.1% of the bar.
**Why it happens:** Different units between timeline API and match API.
**How to avoid:** Convert `game_duration_secs * 1000` to ms, then `(event.timestamp as f64 / game_duration_ms as f64) * 100.0` for percentage.
**Warning signs:** All timeline markers cluster at far left of the bar.

### Pitfall 3: Empty `team_position` for Some Participants
**What goes wrong:** Role-based comparison fails or panics when `team_position` is empty string.
**Why it happens:** Riot API sets `team_position = ""` for ARAM, custom games, or when role detection fails.
**How to avoid:** Check `!team_position.is_empty()` before enabling the "vs Lane Opponent" toggle. When empty, disable the toggle with `opacity-40 cursor-not-allowed` (specified in UI-SPEC interaction contract).
**Warning signs:** Comparison toggle active but showing no opponent data.

### Pitfall 4: `match_id` vs DB Internal ID
**What goes wrong:** The route uses the Riot match ID (e.g. `EUW1_7654321`), not the SurrealDB internal `match:` record ID. Passing the wrong one to DB queries fails silently (no record found).
**Why it happens:** The `match` table has both a SurrealDB auto-generated `id` and a `match_id: string` field. Navigation from stats page uses `riot_match_id`, not `match_db_id`.
**How to avoid:** The cache lookup must query `WHERE match_id = $match_id` (the Riot string), not by record ID. Same pattern already used in `store_matches_with_synced_by`.
**Warning signs:** Cache always misses even after first load.

### Pitfall 5: ORDER BY Fields Not in Partial SELECT (SurrealDB 3.x)
**What goes wrong:** SurrealDB 3.x rejects `ORDER BY <field>` if field is not in a partial SELECT. Affects `get_cached_match_detail` if it uses partial SELECT with ORDER BY.
**Why it happens:** SurrealDB 3.x validation rule (documented in CLAUDE.md rule 40).
**How to avoid:** Use `SELECT *` for match_detail queries, or include any ORDER BY field in the SELECT list explicitly.

### Pitfall 6: `.check()` After CREATE/UPDATE on New Tables
**What goes wrong:** Silent failures when storing match_detail or timeline events.
**Why it happens:** Without `.check()`, SurrealDB constraint violations return `Ok`.
**How to avoid:** Always chain `.check()` after write queries (CLAUDE.md rule 27). Use `BEGIN TRANSACTION / COMMIT TRANSACTION` when storing multiple records for one match detail.

### Pitfall 7: Divergent View Branches in Timeline Filter
**What goes wrong:** Compiler error "mismatched types" in `{move || if has_events { view! { <div...> } } else { view! { <p>...</p> } }}`
**Why it happens:** Leptos 0.8 requires `.into_any()` on divergent branch types (CLAUDE.md rule 19).
**How to avoid:** Add `.into_any()` to every arm of `if`/`match` inside reactive closures.

---

## Code Examples

### Route Param Extraction (Leptos 0.8 / leptos_router 0.7.8)
```rust
// Source: leptos_router 0.7.8 lib.rs example
use leptos_router::hooks::use_params_map;

#[component]
pub fn MatchDetailPage() -> impl IntoView {
    let params = use_params_map();
    let match_id = move || params.read().get("id").unwrap_or_default();
    let detail = Resource::new(match_id, |id| fetch_match_detail(id));
    // ...
}
```

### Data Dragon Item Icon URL
```rust
// Pattern from stats.rs champion_icon_url() — same CDN, different subfolder
fn item_icon_url(item_id: i32) -> String {
    // item_id 0 = empty slot
    format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/item/{item_id}.png")
}
// Empty slot: return empty string, render <div class="w-6 h-6 rounded bg-elevated border border-divider/30" />
```

### Timeline Position Percentage
```rust
// timeline_ms = event.timestamp (already in ms)
// game_duration_ms = game_duration_secs * 1000 (from match table)
fn timeline_pct(event_timestamp_ms: i64, game_duration_secs: i32) -> f64 {
    let game_ms = game_duration_secs as f64 * 1000.0;
    if game_ms <= 0.0 { return 0.0; }
    ((event_timestamp_ms as f64 / game_ms) * 100.0).clamp(0.0, 100.0)
}
```

### Timeline Filter Signal Pattern
```rust
// Each filter category is a separate bool signal
let (show_objectives, set_show_objectives) = signal(true);
let (show_towers, set_show_towers) = signal(true);
let (show_kills, set_show_kills) = signal(true);
let (show_wards, set_show_wards) = signal(true);
let (show_teamfights, set_show_teamfights) = signal(true);

let visible_events = move || {
    events.iter().filter(|e| match e.category {
        EventCategory::Objective => show_objectives.get(),
        EventCategory::Tower => show_towers.get(),
        EventCategory::Kill => show_kills.get(),
        EventCategory::Ward => show_wards.get(),
        EventCategory::Teamfight => show_teamfights.get(),
    }).cloned().collect::<Vec<_>>()
};
```

### Performance Bar Chart (inline style for dynamic width)
```rust
// Source: UI-SPEC PerformanceBarChart spec — bar fill via inline style
view! {
    <div class="bg-elevated h-4 rounded-full relative">
        <div
            class="bg-accent/70 h-full rounded-full"
            style=move || format!("width: {}%", pct.get())
        />
        <div
            class="absolute top-0 h-full w-0.5 bg-muted/50"
            style=move || format!("left: {}%", avg_pct.get())
        />
    </div>
}
```

### SurrealDB Schema for Match Detail Cache
```sql
-- New tables in schema.surql
DEFINE TABLE IF NOT EXISTS match_detail SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match_id ON match_detail TYPE string;
DEFINE FIELD IF NOT EXISTS participants_json ON match_detail TYPE string; -- JSON array of MatchParticipant
DEFINE FIELD IF NOT EXISTS game_duration ON match_detail TYPE int;
DEFINE FIELD IF NOT EXISTS fetched_at ON match_detail TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS match_detail_match_id ON match_detail FIELDS match_id UNIQUE;

DEFINE TABLE IF NOT EXISTS match_timeline_event SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match_id ON match_timeline_event TYPE string;
DEFINE FIELD IF NOT EXISTS timestamp_ms ON match_timeline_event TYPE int;
DEFINE FIELD IF NOT EXISTS event_type ON match_timeline_event TYPE string;
DEFINE FIELD IF NOT EXISTS team_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS killer_participant_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS victim_participant_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS monster_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS monster_sub_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS building_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_first_blood ON match_timeline_event TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS multi_kill_length ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS is_teamfight ON match_timeline_event TYPE bool DEFAULT false;
DEFINE INDEX IF NOT EXISTS timeline_match_id ON match_timeline_event FIELDS match_id;
```

**Storage design note:** `participants_json` stores all 10 participants as a JSON string (serialized `Vec<MatchParticipant>`) — avoids 10 separate DB rows for participant data, keeping the query pattern to two tables (match_detail + match_timeline_event) instead of three.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `type::thing()` SurrealDB | `type::record()` | SurrealDB 2.x | Project already uses correct form |
| `attr:value` for controlled inputs | `prop:value` | Leptos 0.7+ | Use `style` attribute for dynamic inline styles |
| Collecting views as `Vec<_>` | `.collect_view()` | Leptos 0.7+ | Required for iterator fragments in `view!` |

**Note:** The stats.rs file hardcodes DDragon version `15.6.1` in `champion_icon_url()`. This same hardcoded version is fine for item icons in this phase — same CDN, same version assumption. A future phase can make version dynamic via `fetch_latest_version()` from `data_dragon.rs`.

---

## Open Questions

1. **participants_json vs separate participant rows**
   - What we know: UI-SPEC calls for 10-row scoreboards; data is only read as a unit
   - What's unclear: Whether storing as a JSON string in `match_detail` is cleaner than a `match_participant` table
   - Recommendation: Use `participants_json` string field. Avoids 10-row fan-out per match, keeps DB query count to 2 (match_detail + timeline_events), and participants are always read as a complete set. No filtering by individual participant in this phase.

2. **Region for API calls from `/match/:id` route**
   - What we know: Match IDs are prefixed with region (e.g. `EUW1_...`, `NA1_...`). The `user.riot_region` field exists from Phase 12.
   - What's unclear: Whether the platform route should be derived from the match ID prefix or from `user.riot_region`
   - Recommendation: Derive platform from `user.riot_region` (same as sync flow). The match ID prefix could be parsed as a fallback but adds complexity. The user who synced the match is in the same region.

3. **Timeline "recall" event type**
   - What we know: D-07 lists recalls as an event type. The riven `EventsTimeLine.r#type` string includes "CHAMPION_RECALL" in some API versions.
   - What's unclear: Whether `CHAMPION_RECALL` is consistently present in match-v5 timeline data or version-dependent
   - Recommendation: Include recall in the filter enum but mark events as optional — if no recall events appear, the filter button still works (shows empty filtered state). Do not error if zero recalls found.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| riven | Riot API calls | Already in Cargo.toml | 2.78.0 | — |
| SurrealDB | DB caching | Running in dev | 3.x | — |
| RIOT_API_KEY env var | match_v5 API | Set in .env | — | API key missing error (D-15) |
| Data Dragon CDN | Item icons | Public CDN | n/a | Empty item slot `<div>` |

**Missing dependencies with no fallback:** None — all required components exist.

**Missing dependencies with fallback:** RIOT_API_KEY may be absent in test environments — the `has_api_key()` check already handles this; return error banner.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests + integration tests via tests/) |
| Config file | none — uses `cargo test --features ssr --lib` |
| Quick run command | `cargo test --features ssr --lib -- match` |
| Full suite command | `cargo test --features ssr --lib` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MATCH-01 | All 10 participants extracted from riven response | unit | `cargo test --features ssr --lib -- test_extract_participants` | No — Wave 0 |
| MATCH-02 | Timeline events extracted and classified by type | unit | `cargo test --features ssr --lib -- test_classify_timeline_events` | No — Wave 0 |
| MATCH-03 | Performance stats computed correctly (damage share %, CS/min) | unit | `cargo test --features ssr --lib -- test_compute_performance_stats` | No — Wave 0 |
| MATCH-04 | Cache hit returns cached data, cache miss triggers API | integration | `cargo test --features ssr --lib -- test_match_detail_cache` | No — Wave 0 |
| MATCH-01 | `/match/:id` route renders without crash (smoke) | e2e | `cd e2e && npx playwright test audit-match-detail.spec.ts` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib -- match`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** Full suite green + e2e smoke test before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/server/db.rs` — `#[cfg(test)]` block: `test_match_detail_cache`, `test_get_cached_match_detail_miss`
- [ ] Pure function tests (no DB): `test_extract_participants`, `test_classify_timeline_events`, `test_compute_performance_stats`, `test_timeline_pct`, `test_teamfight_detection`
- [ ] `e2e/tests/audit-match-detail.spec.ts` — smoke test: page loads at `/match/test_id`, shows skeleton or empty state without crash

---

## Sources

### Primary (HIGH confidence)
- riven 2.78.0 source (`~/.cargo/registry/src/.../riven-2.78.0/src/models.rs`) — verified all participant fields (item0–item6, gold_earned, team_position, summoner_name), timeline structs (EventsTimeLine, FramesTimeLine), get_timeline signature
- leptos_router 0.7.8 source (`~/.cargo/registry/src/.../leptos_router-0.7.8/src/`) — verified `use_params_map()`, `Memo<ParamsMap>`, `.read().get("id")` pattern
- Project codebase (src/server/riot.rs, db.rs, schema.surql) — verified existing patterns, IDs, table structure

### Secondary (MEDIUM confidence)
- `13-CONTEXT.md` (user decisions) — all locked decisions treated as requirements
- `13-UI-SPEC.md` (UI design contract) — component specs, class names, interaction contract
- CLAUDE.md gotchas — applied rules 19, 27, 28, 31, 40 directly to pitfalls

### Tertiary (LOW confidence)
- CHAMPION_RECALL event type availability: inferred from Riot API v5 documentation knowledge; not verified against live API responses. Mark as optional in event classification.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified from Cargo.lock and existing code
- Architecture: HIGH — derived directly from verified riven types and project patterns
- Pitfalls: HIGH — items 1–6 verified from riven source code and CLAUDE.md rules; item 7 (CHAMPION_RECALL) is MEDIUM
- Timeline event types: HIGH for CHAMPION_KILL, BUILDING_KILL, ELITE_MONSTER_KILL, WARD_PLACED; MEDIUM for CHAMPION_RECALL

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable riven API; DDragon version hardcode is a known limitation)
