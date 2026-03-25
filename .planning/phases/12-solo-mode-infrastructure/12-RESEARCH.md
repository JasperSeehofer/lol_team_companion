# Phase 12: Solo Mode Infrastructure - Research

**Researched:** 2026-03-25
**Domain:** Leptos 0.8 / Rust — user mode toggle, multi-region Riot API, ranked data, solo dashboard, queue filtering
**Confidence:** HIGH (all findings verified against codebase or official riven docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Mode toggle lives in the **nav bar** — persistent, one-click switch between team and solo mode
- **D-02:** In solo mode, all nav links remain visible and clickable. Team-only pages (Team Dashboard, Roster, Opponents) show a "Switch to team mode to use this feature" gate message on click — not hidden or greyed out
- **D-03:** New users default to **solo mode** and land on the solo dashboard after login/register. They switch to team mode when they create/join a team
- **D-04:** Mode is persisted in the DB `user` record (per STATE.md decision), not a WASM signal — survives hard navigation and login/logout
- **D-05:** **Multi-region from day one** — support all major Riot API regions (EUW, NA, KR, EUNE, OCE, BR, LAN, LAS, TR, RU, JP, PH, SG, TH, TW, VN). Parameterize all hardcoded `EUROPE`/`EUW1` routes in `riot.rs`
- **D-06:** Region selection happens on the **profile page during Riot account linking** — dropdown appears alongside game_name/tag_line inputs. Region stored on user record
- **D-07:** **Single column, stacked sections** layout — top: ranked badge, middle: recent matches, bottom: goal placeholders
- **D-08:** Ranked badge shows **tier icon image + text + LP** (e.g. [Gold emblem] Gold II - 47 LP). Tier icons from Riot ranked emblems (Data Dragon or static assets)
- **D-09:** Dashboard shows **solo/duo queue rank only** — flex rank not displayed
- **D-10:** Match sync is **manual button + auto on page load** — auto-sync when visiting solo dashboard if last sync > 10 minutes ago, plus a manual "Sync Matches" button
- **D-11:** Goal status section shows **greyed-out placeholder cards** with "Coming in Phase 15" label — 2-3 cards teasing Rank Target, CS/min, Deaths goals
- **D-12:** Queue filter uses a **dropdown select** (All / Solo/Duo / Flex) — compact, two clicks to change
- **D-13:** Queue filter appears on **both the stats page and solo dashboard** — users can filter anywhere they see matches

### Claude's Discretion

- Auto-sync staleness threshold (10 min suggested, Claude can adjust based on Riot API rate limits)
- Exact region-to-route mapping implementation (lookup table, enum, etc.)
- How the "switch to team mode" gate message is styled on team-only pages
- Ranked emblem image source (Data Dragon CDN vs. bundled static assets)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SOLO-01 | User can toggle between team mode and solo mode, persisted across sessions | DB field `mode` on `user` table; server fn to read/write; nav bar toggle component |
| SOLO-02 | User can sync solo queue match history without being on a team | Extend `store_matches` to accept `synced_by` user ref; `get_current_user` provides user context |
| SOLO-03 | User can filter match history by queue type (solo queue, flex, all) | Queue IDs: solo=420, flex=440; filter param on stats server fn; dropdown component |
| SOLO-04 | User sees a personal solo dashboard with recent matches, ranked badge, learnings, and goal status | New `/solo` route; batched query for dashboard data; ranked snapshot read |
| SOLO-05 | User can select their region during Riot account linking (EUW, NA, KR, etc.) | Dropdown on profile Riot linking form; `region` field on `user` table; parameterize riot.rs |
| RANK-01 | User can see their current ranked tier, division, and LP | `league_v4().get_league_entries_by_puuid(platform, puuid)` → LeagueEntry fields |
| RANK-03 | System snapshots LP on each stats sync for history tracking | New `ranked_snapshot` table; insert on sync with tier/division/lp/timestamp |
</phase_requirements>

---

## Summary

Phase 12 builds "solo mode infrastructure" — the foundation that lets users use the app independently of a team. The work touches six areas: (1) a DB-persisted mode field on the user record with a nav toggle, (2) multi-region support by parameterizing all hardcoded `EUROPE`/`EUW1` constants in `riot.rs`, (3) a `region` field on the user record with a profile-page dropdown, (4) a new `ranked_snapshot` table and calls to the riven `league_v4` endpoint, (5) queue-type filtering on the stats page, and (6) a new `/solo` route for the personal dashboard.

The codebase already has all the scaffolding needed: `store_matches` in `db.rs` writes match records, `riot.rs` calls `match_v5`, and the `DbUser`/`AppUser` pattern is well-established. The key changes are additive: new DB fields, new schema tables, new riot.rs functions, a new page, and navigation changes.

The only architectural complexity is the mode-aware redirect at login/register (currently hardcoded to `/team/dashboard`) and the gate message pattern on team-only pages. Both follow established Leptos patterns already in the codebase.

**Primary recommendation:** Add `mode` and `riot_region` fields to `user` in `schema.surql`; add `ranked_snapshot` table; parameterize `riot.rs`; create `src/pages/solo_dashboard.rs`; update nav, login, register, and profile.

---

## Standard Stack

All libraries are already in `Cargo.toml`. No new dependencies required.

### Core (already present)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| riven | 2.78.0 (locked) | Riot API client | `league_v4().get_league_entries_by_puuid` for ranked; `PlatformRoute::to_regional()` for match routing |
| surrealdb | 3.x | DB queries | New tables follow `DEFINE TABLE IF NOT EXISTS` / `DEFINE FIELD IF NOT EXISTS` pattern |
| leptos | 0.8 | UI components | Mode toggle follows `ThemeToggle` pattern; dashboard follows team dashboard pattern |

### No New Dependencies

This phase requires zero new crate dependencies. All capabilities exist:
- Ranked data: `riven::endpoints::LeagueV4::get_league_entries_by_puuid`
- Region routing: `riven::consts::PlatformRoute` (all 16+ variants present in 2.78.0)
- Tier icons: static assets in `public/` directory or Data Dragon CDN URL pattern

---

## Architecture Patterns

### DB Schema Additions (schema.surql)

```surql
-- Add mode and riot_region to user (IF NOT EXISTS — idempotent)
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo';
DEFINE FIELD IF NOT EXISTS riot_region ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_solo_sync ON user TYPE option<datetime>;

-- Add synced_by to match (replaces team_id as the ownership anchor for solo matches)
-- Note: team_id already exists; synced_by is the user who triggered the sync
DEFINE FIELD IF NOT EXISTS synced_by ON match TYPE option<record<user>>;

-- Ranked snapshots table
DEFINE TABLE IF NOT EXISTS ranked_snapshot SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON ranked_snapshot TYPE record<user>;
DEFINE FIELD IF NOT EXISTS queue_type ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS tier ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS division ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS lp ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS wins ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS losses ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS snapshotted_at ON ranked_snapshot TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS ranked_snapshot_user_queue ON ranked_snapshot FIELDS user, queue_type;
```

**Critical note on `team_id` vs `synced_by`**: The existing `match` table has `team_id` (`option<record<team>>`). Solo syncs leave `team_id` as `NONE` and set `synced_by` to the user. This avoids a breaking migration on the existing team sync path. `get_team_match_stats` continues to filter by `user IN [team members]` — it does not need to change.

### Region Parameterization (riot.rs)

Current code has three hardcoded constants that must become function parameters:
- `RegionalRoute::EUROPE` in `get_puuid`, `fetch_match_history`, `fetch_player_champions`, `fetch_player_intel`
- `PlatformRoute::EUW1` in `fetch_champion_masteries`

**Pattern — region lookup table:**

```rust
// In riot.rs — add these two helpers
pub fn platform_route_from_str(region: &str) -> riven::consts::PlatformRoute {
    match region {
        "EUW"  => riven::consts::PlatformRoute::EUW1,
        "EUNE" => riven::consts::PlatformRoute::EUN1,
        "NA"   => riven::consts::PlatformRoute::NA1,
        "KR"   => riven::consts::PlatformRoute::KR,
        "BR"   => riven::consts::PlatformRoute::BR1,
        "LAN"  => riven::consts::PlatformRoute::LA1,
        "LAS"  => riven::consts::PlatformRoute::LA2,
        "OCE"  => riven::consts::PlatformRoute::OC1,
        "TR"   => riven::consts::PlatformRoute::TR1,
        "RU"   => riven::consts::PlatformRoute::RU,
        "JP"   => riven::consts::PlatformRoute::JP1,
        "SG"   => riven::consts::PlatformRoute::SG2,
        "TW"   => riven::consts::PlatformRoute::TW2,
        "VN"   => riven::consts::PlatformRoute::VN2,
        "ME"   => riven::consts::PlatformRoute::ME1,
        _      => riven::consts::PlatformRoute::EUW1, // safe fallback
    }
}

// PlatformRoute has .to_regional() — use it for match-v5 calls:
// platform.to_regional() returns the correct RegionalRoute
```

**All five functions in riot.rs need a `platform: PlatformRoute` parameter added:**
- `get_puuid(game_name, tag_line, platform)` — uses `platform.to_regional()` for `account_v1`
- `fetch_match_history(puuid, queue_id, platform)` — uses `platform.to_regional()` for `match_v5`
- `fetch_champion_masteries(puuid, platform)` — uses `platform` directly for `champion_mastery_v4`
- `fetch_player_champions(puuid, count, platform)` — uses `platform.to_regional()`
- `fetch_player_intel(puuid, match_count, platform)` — uses `platform.to_regional()`

**All call sites** (stats page sync, profile link, team intel) must pass a platform. Fall back to `EUW1` when user has no region set.

### Ranked Data (riot.rs — new function)

```rust
pub struct RankedEntry {
    pub queue_type: String,  // "RANKED_SOLO_5x5" or "RANKED_FLEX_SR"
    pub tier: String,        // "GOLD", "PLATINUM", etc.
    pub division: String,    // "I", "II", "III", "IV" (empty for Master+)
    pub lp: i32,
    pub wins: i32,
    pub losses: i32,
}

pub async fn fetch_ranked_data(
    puuid: &str,
    platform: riven::consts::PlatformRoute,
) -> Result<Vec<RankedEntry>, RiotError> {
    let api = api();
    let entries = api
        .league_v4()
        .get_league_entries_by_puuid(platform, puuid)
        .await?;
    Ok(entries.into_iter().map(|e| RankedEntry {
        queue_type: format!("{:?}", e.queue_type),
        tier: e.tier.map(|t| format!("{:?}", t)).unwrap_or_default(),
        division: e.rank.clone(),
        lp: e.league_points,
        wins: e.wins,
        losses: e.losses,
    }).collect())
}
```

**LeagueEntry fields confirmed** (riven 2.78.0, docs.rs verified):
- `tier: Option<Tier>` — enum: IRON, BRONZE, SILVER, GOLD, PLATINUM, EMERALD, DIAMOND, MASTER, GRANDMASTER, CHALLENGER
- `rank: String` — division string: "I", "II", "III", "IV" (empty string for Master+)
- `league_points: i32` — LP value
- `wins: i32`, `losses: i32`
- `queue_type: QueueType` — discriminate solo vs flex

### Mode Toggle Component

The `ThemeToggle` in `src/components/theme_toggle.rs` is the reference pattern for adding a toggle to the nav bar. The mode toggle is simpler — it calls a server fn to update the user record and reads back the current mode via `get_current_user()`.

**Key pattern — mode toggle in nav:**
```rust
// Mode toggle server fn (in nav.rs or a new user_mode.rs)
#[server]
pub async fn set_user_mode(mode: String) -> Result<(), ServerFnError> {
    // UPDATE user SET mode = $mode WHERE id = $user_id
    // Invalidates nothing — nav re-reads mode from get_current_user on next load
}

#[server]
pub async fn get_user_mode() -> Result<String, ServerFnError> {
    // Returns "solo" or "team"
}
```

`PublicUser` must gain a `mode` field (or a separate `get_user_mode` server fn is added). Either works; adding `mode` to `PublicUser` is consistent with the existing pattern (all user data read via `get_current_user`).

### Mode-Aware Redirect at Login/Register

Currently both `login_action` (login.rs) and `register_action` (register.rs) hardcode `redirect("/team/dashboard")`. These must become mode-aware:

```rust
// In login_action server fn body:
let mode = db::get_user_mode(&db, &user.id).await.unwrap_or("solo".to_string());
let dest = if mode == "team" { "/team/dashboard" } else { "/solo" };
redirect(dest);
```

Same for the `Effect` on the client side that calls `window.location().set_href(...)` — it must read the redirect destination from the server fn response or issue a separate `get_user_mode` call. **Simplest approach:** have the server fn return `Ok(String)` (the redirect URL) instead of `Ok(())`; the client Effect navigates there.

**Alternative:** Keep server fn returning `Ok(())` and add a separate `get_redirect_target` server fn called in the `Effect`. The first approach (return the URL) is simpler.

### Team-Only Page Gate

D-02 requires team-only pages (Team Dashboard, Roster, Opponents) to show a gate message in solo mode. These pages already do auth-redirect on `get_current_user() == None`. The gate message is an additional check:

```rust
// Pattern for team-only pages in solo mode:
// 1. get_current_user() returns Some(user)
// 2. user.mode == "solo" → render gate message instead of page content
// 3. Gate message: "Switch to team mode to use this feature"
//    Use existing ErrorBanner component or a custom GateMessage component
```

The gate check happens client-side (in the `Suspend::new` block after the user resource resolves), same as the auth redirect pattern.

### Queue Filter (SOLO-03)

Queue IDs in Riot API:
- `420` = RANKED_SOLO_5x5 (Solo/Duo)
- `440` = RANKED_FLEX_SR (Flex)
- `0` = custom / all

**Filter implementation:** Add an optional `queue_id: Option<i32>` parameter to the stats server fn (`get_team_stats` and new `get_solo_matches`). The SurrealDB query adds `AND match.queue_id = $queue_id` when set.

The dropdown is a `<select>` element with a `RwSignal<Option<i32>>`. Changing it calls `.set()` which triggers the Resource to refetch (via reactive dependency).

```rust
let queue_filter: RwSignal<Option<i32>> = RwSignal::new(None);
let matches = Resource::new(
    move || queue_filter.get(),
    |qf| get_solo_matches(qf),
);
```

### Solo Dashboard Route

New file: `src/pages/solo_dashboard.rs`
New route in `app.rs`: `<Route path=path!("/solo") view=SoloDashboardPage />`

**Batched dashboard query (db.rs):**
```rust
pub async fn get_solo_dashboard(
    db: &Surreal<Db>,
    user_id: &str,
    queue_filter: Option<i32>,
) -> DbResult<SoloDashboardData> {
    // Statement 0: latest ranked_snapshot for user, queue='RANKED_SOLO_5x5'
    // Statement 1: recent player_match records (20), filtered by queue if set
    // Statement 2: last_solo_sync timestamp from user record
    // Batch in one .query() call
}
```

### Auto-Sync Staleness Check (D-10)

The 10-minute threshold is appropriate for Riot API limits. `last_solo_sync` is stored on the `user` record (not a separate table). On page load:

```rust
// In get_solo_dashboard or a separate check_and_sync server fn:
// If now() - last_solo_sync > 10 minutes AND user has puuid AND API key present
//   → trigger sync inline (or return a flag to client to trigger)
```

**Recommended:** Return a `should_auto_sync: bool` from the dashboard server fn based on staleness; the client calls the sync server fn when `should_auto_sync == true`. This keeps sync async and non-blocking for the initial page render.

### Ranked Emblem Images

Decision D-08 defers the source to Claude's discretion. **Recommendation: Data Dragon CDN URLs** — no bundled assets, no `public/` clutter, consistent with existing `data_dragon.rs` CDN usage.

Data Dragon ranked emblem URL pattern (verified from Riot CDN):
```
https://ddragon.leagueoflegends.com/cdn/img/ranked-emblem/emblem-{tier_lower}.png
```
Examples:
- `emblem-gold.png`
- `emblem-platinum.png`
- `emblem-diamond.png`
- `emblem-challenger.png`

Tier name must be lowercased. Use `<img>` tag in the ranked badge component with `alt` fallback text.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Region → regional route mapping | Custom routing logic | `PlatformRoute::to_regional()` (riven built-in) | Handles all 16+ regions correctly, accounts for deprecated routes |
| Ranked tier formatting | Custom display strings | `format!("{:?}", entry.tier)` then `to_lowercase()` | Riven Tier enum Debug output is the canonical tier name |
| Match deduplication | Custom hash/set logic | Existing `SELECT id FROM match WHERE match_id = $match_id LIMIT 1` pattern in `store_matches` | Already handles idempotent upserts |
| Queue filter in DB | Application-level filtering | SurrealDB `AND match.queue_id = $queue_id` in SELECT | Push filtering to DB, not Rust code |

---

## Runtime State Inventory

> Not a rename/refactor phase. This section is included only to address the `synced_by` / `team_id` schema migration concern.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data — existing match records | All existing `match` records have `team_id` set and `synced_by = NONE` | No migration needed — `synced_by` field added with `IF NOT EXISTS`, existing records have `NONE` |
| Stored data — existing user records | All users lack `mode` and `riot_region` fields | No migration needed — `DEFAULT 'solo'` on `mode` field; new fields via `DEFINE FIELD IF NOT EXISTS` |
| Stored data — ranked_snapshot | Does not exist yet | New table, created on startup |
| Live service config | None relevant | None |
| OS-registered state | None | None |
| Secrets/env vars | `RIOT_API_KEY` unchanged | None |
| Build artifacts | None affected | None |

---

## Common Pitfalls

### Pitfall 1: PlatformRoute vs RegionalRoute — wrong one for match-v5

**What goes wrong:** `match_v5` requires `RegionalRoute` (EUROPE, AMERICAS, ASIA, SEA), not `PlatformRoute` (EUW1, NA1, KR). Passing `PlatformRoute` to `get_match_ids_by_puuid` is a type error.

**Why it happens:** The existing code passes `RegionalRoute::EUROPE` directly. After parameterization, only `PlatformRoute` is stored in the DB — `RegionalRoute` must be derived from it.

**How to avoid:** Call `platform.to_regional()` to convert. The riven `PlatformRoute` enum has this method; it maps all 16 platforms to their correct regional route.

**Warning signs:** Compile error `expected RegionalRoute, found PlatformRoute` if you mistakenly pass the wrong type.

### Pitfall 2: account-v1 uses RegionalRoute::EUROPE for all regions except SEA

**What goes wrong:** `get_puuid` uses `account_v1().get_by_riot_id(RegionalRoute::EUROPE, ...)`. For SEA-region players (OC1, SG2, VN2, TW2), the correct regional route for `account-v1` is `SEA`, not `EUROPE`.

**Why it happens:** `account-v1` is separate from `match-v5` routing — it does NOT use `platform.to_regional()`. It has its own regional grouping.

**How to avoid:** For `account_v1`, derive the regional route differently:
```rust
fn account_region_for(platform: PlatformRoute) -> RegionalRoute {
    match platform {
        PlatformRoute::OC1 | PlatformRoute::SG2 | PlatformRoute::TW2 | PlatformRoute::VN2 | PlatformRoute::ME1 => RegionalRoute::SEA,
        PlatformRoute::NA1 | PlatformRoute::BR1 | PlatformRoute::LA1 | PlatformRoute::LA2 => RegionalRoute::AMERICAS,
        PlatformRoute::KR | PlatformRoute::JP1 => RegionalRoute::ASIA,
        _ => RegionalRoute::EUROPE,
    }
}
```

**Warning signs:** Players from SEA regions fail PUUID lookup with 404 if EUROPE is used.

### Pitfall 3: Multiple closures capturing mode signal — clone before each

**What goes wrong:** The mode toggle in the nav bar needs the mode value in multiple closures (one for display, one for the click handler). `String` is not `Copy`.

**Why it happens:** Leptos rule 18 — clone before multiple `move` closures.

**How to avoid:** `let mode_for_display = mode_val.clone();` before each closure. Or use `StoredValue` if the value is large.

### Pitfall 4: mode field missing from PublicUser breaks nav toggle

**What goes wrong:** `get_current_user()` returns `PublicUser` which currently has `id`, `username`, `riot_summoner_name`. If `mode` is not added to `PublicUser`, the nav toggle cannot read the current mode without a separate server fn call.

**How to avoid:** Add `mode: String` to `PublicUser`. Update `get_current_user()` server fn to populate it from `AppUser`. Update `AppUser` and `DbUser` in `auth.rs` to include `mode`.

### Pitfall 5: Auto-sync blocks initial page render

**What goes wrong:** If the solo dashboard server fn performs the Riot API sync synchronously before returning, slow API calls (1-3 seconds each for 20 matches) block the entire page from loading.

**How to avoid:** The dashboard server fn returns `should_auto_sync: bool`. Client triggers a separate `sync_solo_matches()` server fn after initial render using `spawn_local`. The page renders immediately with whatever data is already in DB.

### Pitfall 6: `ORDER BY` on `snapshotted_at` requires it in SELECT

**What goes wrong:** SurrealDB 3.x rejects `ORDER BY snapshotted_at` if the field is not selected in a partial SELECT. See CLAUDE.md rule 40.

**How to avoid:** Use `SELECT *` on ranked_snapshot queries, or explicitly include `snapshotted_at` in partial SELECTs.

### Pitfall 7: Login/register redirect mismatch between server redirect and client Effect

**What goes wrong:** `login_action` calls `leptos_axum::redirect("/team/dashboard")`. The client `Effect` also calls `window.location().set_href("/team/dashboard")`. Both must use the same mode-aware destination, or the wrong redirect fires.

**How to avoid:** Change the server fn to return `Ok(String)` (the destination URL) instead of `Ok(())`. The client `Effect` reads `login.value().get()` → `Ok(dest)` and navigates there. Remove the server-side `redirect()` call (it fires before the session is fully written anyway — the hard-nav Effect is authoritative per CLAUDE.md rule 8).

---

## Code Examples

### Pattern: DB field addition with default (schema.surql)

```surql
-- Source: schema.surql existing pattern (all fields use IF NOT EXISTS)
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo';
DEFINE FIELD IF NOT EXISTS riot_region ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_solo_sync ON user TYPE option<datetime>;
```

### Pattern: server fn returning redirect URL instead of ()

```rust
// Source: adapted from login.rs pattern
#[server]
pub async fn login_action(email: String, password: String) -> Result<String, ServerFnError> {
    // ... authenticate ...
    let mode = db::get_user_mode(&db, &user.id).await.unwrap_or_default();
    let dest = if mode == "team" { "/team/dashboard".to_string() } else { "/solo".to_string() };
    Ok(dest) // client Effect navigates here
}

// Client Effect:
Effect::new(move || {
    if let Some(Ok(dest)) = login.value().get() {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(&dest);
        }
    }
});
```

### Pattern: ranked data fetch (riot.rs)

```rust
// Source: riven 2.78.0 docs.rs — get_league_entries_by_puuid
pub async fn fetch_ranked_data(
    puuid: &str,
    platform: riven::consts::PlatformRoute,
) -> Result<Vec<RankedEntry>, RiotError> {
    let api = api();
    let entries = api
        .league_v4()
        .get_league_entries_by_puuid(platform, puuid)
        .await?;
    Ok(entries.into_iter().map(|e| RankedEntry {
        queue_type: format!("{:?}", e.queue_type),
        tier: e.tier.map(|t| format!("{:?}", t)).unwrap_or_default(),
        division: e.rank.clone(),
        lp: e.league_points,
        wins: e.wins,
        losses: e.losses,
    }).collect())
}
```

### Pattern: PlatformRoute to_regional() for match-v5

```rust
// Source: riven 2.78.0 — PlatformRoute has .to_regional() method
let regional = platform.to_regional(); // RegionalRoute
api.match_v5()
    .get_match_ids_by_puuid(regional, puuid, Some(20), None, queue_filter, None, None, None)
    .await?;
```

### Pattern: queue filter signal → Resource

```rust
// Source: existing Resource pattern in draft.rs, adapted
let queue_filter: RwSignal<Option<i32>> = RwSignal::new(None);
let matches = Resource::new(
    move || queue_filter.get(),
    |qf| get_solo_matches(qf),
);

// Dropdown:
view! {
    <select
        on:change=move |ev| {
            let val = event_target_value(&ev);
            queue_filter.set(match val.as_str() {
                "420" => Some(420),
                "440" => Some(440),
                _ => None,
            });
        }
    >
        <option value="">"All"</option>
        <option value="420">"Solo/Duo"</option>
        <option value="440">"Flex"</option>
    </select>
}
```

### Pattern: ranked emblem image

```rust
// Source: Riot CDN URL pattern (Data Dragon)
fn tier_emblem_url(tier: &str) -> String {
    format!(
        "https://ddragon.leagueoflegends.com/cdn/img/ranked-emblem/emblem-{}.png",
        tier.to_lowercase()
    )
}
// Usage:
view! { <img src=tier_emblem_url(&ranked.tier) alt=ranked.tier.clone() class="w-16 h-16" /> }
```

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (Rust nightly) | All compilation | ✓ | 1.96.0-nightly | — |
| riven 2.78.0 | Riot API calls | ✓ (locked in Cargo.lock) | 2.78.0 | — |
| Riot API key (`RIOT_API_KEY`) | Ranked + match sync | Env-dependent | — | `has_api_key()` gate (already in code) |
| Data Dragon CDN | Ranked emblems | Requires internet at runtime | — | Fallback: text-only tier display |

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test + Playwright |
| Config file | `Cargo.toml` + `e2e/playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `just verify && just e2e` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SOLO-01 | Mode toggle persists after navigation | unit (DB round-trip) + e2e | `cargo test --features ssr --lib` | ❌ Wave 0 |
| SOLO-02 | Solo sync stores matches without team | unit (db::store_matches with no team_id) | `cargo test --features ssr --lib` | ❌ Wave 0 |
| SOLO-03 | Queue filter narrows results | unit (DB query filter) | `cargo test --features ssr --lib` | ❌ Wave 0 |
| SOLO-04 | Solo dashboard route renders | e2e smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ Wave 0 |
| SOLO-05 | Region stored and used for API calls | unit (platform_route_from_str) | `cargo test --features ssr --lib` | ❌ Wave 0 |
| RANK-01 | Ranked badge shows tier/LP | e2e + manual (requires API key) | manual | N/A |
| RANK-03 | Snapshot inserted on sync | unit (db::store_ranked_snapshot) | `cargo test --features ssr --lib` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** Full Playwright e2e suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit tests for `db::get_user_mode` / `db::set_user_mode`
- [ ] Unit test for `riot::platform_route_from_str` — all region codes map to correct `PlatformRoute`
- [ ] Unit test for `db::store_ranked_snapshot` — inserts with correct fields
- [ ] Unit test for `db::get_solo_matches` — queue filter applied correctly
- [ ] E2e: add `/solo` to `AUTHED_PAGES` array in `e2e/tests/pages.spec.ts`

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| `RegionalRoute::EUROPE` hardcoded | `platform.to_regional()` from stored user region | Enables multi-region |
| `PlatformRoute::EUW1` hardcoded | Passed as parameter from user record | Same |
| Redirect to `/team/dashboard` unconditionally | Mode-aware redirect to `/solo` or `/team/dashboard` | D-03 |
| No mode field on user | `mode: String DEFAULT 'solo'` on user record | D-04 |
| No ranked data | `league_v4().get_league_entries_by_puuid` | RANK-01 |

---

## Open Questions

1. **Master+ LP boundary display**
   - What we know: LeagueEntry `rank` field is empty string for Master/Grandmaster/Challenger
   - What's unclear: Should we display "Master 0 LP" or "Master 250 LP" (LP still returned)? Does `division` display as empty or require special handling?
   - Recommendation: Display tier name + LP only (no division text) when division is empty string. "Master — 250 LP" not "Master I — 250 LP".

2. **Regions deprecated in riven 2.78.0 (PH2, TH2 merged into SG2)**
   - What we know: PH2 and TH2 variants still exist in riven 2.78.0 but are deprecated; SG2 is the current route
   - What's unclear: Should the region dropdown show "PH", "TH" as options mapping to SG2, or drop them?
   - Recommendation: Omit PH/TH from the dropdown (too niche, SG covers them); include SG, TW, VN for SEA coverage.

3. **`get_current_user` vs separate mode server fn**
   - What we know: `get_current_user()` is called on every nav render. Adding `mode` to `PublicUser` is the lowest-friction approach.
   - What's unclear: Whether `AppUser` / `DbUser` in `auth.rs` should also carry `mode`, or only `PublicUser`.
   - Recommendation: Add `mode` to `DbUser`, `AppUser`, and `PublicUser`. The `auth.rs` `DbUser` struct is the source of truth; propagate through. Requires adding `mode` field to the `From<DbUser> for AppUser` impl.

---

## Sources

### Primary (HIGH confidence)
- `src/server/riot.rs` — full source read; all hardcoded routes identified
- `src/server/db.rs` — `store_matches` and `get_team_match_stats` read in full; existing match schema confirmed
- `schema.surql` — full source read; no `mode`, `riot_region`, `synced_by`, or `ranked_snapshot` present
- `src/server/auth.rs` — `DbUser`/`AppUser` struct fields confirmed; no `mode` field present
- `src/components/nav.rs` — full source read; toggle pattern understood
- `src/pages/auth/login.rs`, `register.rs` — hardcoded `/team/dashboard` redirects confirmed
- [riven 2.78.0 docs.rs — LeagueV4](https://docs.rs/riven/2.78.0/riven/endpoints/struct.LeagueV4.html) — `get_league_entries_by_puuid` signature confirmed
- [riven 2.78.0 docs.rs — LeagueEntry](https://docs.rs/riven/2.78.0/riven/models/league_v4/struct.LeagueEntry.html) — all fields confirmed
- [riven 2.78.0 docs.rs — PlatformRoute](https://docs.rs/riven/2.78.0/riven/consts/enum.PlatformRoute.html) — all 16+ variants confirmed, `to_regional()` method confirmed
- [riven 2.78.0 docs.rs — RegionalRoute](https://docs.rs/riven/2.78.0/riven/consts/enum.RegionalRoute.html) — EUROPE/AMERICAS/ASIA/SEA variants confirmed

### Secondary (MEDIUM confidence)
- account-v1 routing convention (EUROPE/AMERICAS/ASIA/SEA grouping) — standard Riot API pattern verified via riven docs; SEA region handling for account-v1 inferred from known API behavior
- Data Dragon ranked emblem URL pattern — well-established CDN convention; should be verified at implementation time

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing libraries confirmed in Cargo.lock
- Architecture: HIGH — all patterns derived from existing codebase code reads
- Riot API (ranked): HIGH — `get_league_entries_by_puuid` method signature confirmed via docs.rs
- Pitfalls: HIGH — identified from actual code inspection (hardcoded routes, redirect pattern, Leptos rules from CLAUDE.md)
- account-v1 SEA routing: MEDIUM — inferred from Riot API convention, not verified against riven source directly

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (riven 2.78.0 locked; Riot API routing stable)
