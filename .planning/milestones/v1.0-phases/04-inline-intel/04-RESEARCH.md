# Phase 4: Inline Intel - Research

**Researched:** 2026-03-16
**Domain:** Leptos 0.8 reactive UI, Riot API (champion mastery), SurrealDB queries, Rust SSR/WASM patterns
**Confidence:** HIGH

## Summary

Phase 4 surfaces three pieces of contextual intelligence onto existing pages (draft planner, game plan) without adding new routes. Most infrastructure already exists: `get_team_pools()`, `get_opponent_intel()`, and `get_win_condition_stats()` are implemented and called from these pages. The delta work is: (1) adding pool-warning badge overlays to pick slots in `DraftBoard`, (2) enhancing the "Their Picks" intel tab with frequency counts, OTP detection, and Riot API champion mastery, and (3) wiring the win condition tracker panel into `game_plan.rs` using the already-implemented `get_strategy_win_rates()` server fn.

The Riot API champion mastery endpoint is available via `riven::champion_mastery_v4().get_all_champion_masteries_by_puuid()`. The `riven::champion_mastery_v4::ChampionMastery` struct provides `champion_id` (i64), `champion_points` (i32), and `champion_level` (i32). Champion IDs must be mapped to names via Data Dragon. This is new work — no mastery fetch exists in `riot.rs` yet.

**Primary recommendation:** Build incrementally from existing infrastructure. Pool warnings are pure client-side computation using already-loaded data. Opponent tendencies extend existing sidebar. Win condition tracker is a new panel in game_plan.rs backed by an already-working server fn.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Pool Warning Badges (PIPE-03)**
- Trigger: Champion not in player's pool AND exposes a class/archetype gap (using Phase 2 Data Dragon class tags)
- Player-to-slot matching: Best-guess role matching using champion's primary role from Data Dragon, with manual override available per slot
- Visual: Small warning icon badge at top-right of pick slot (mirrors existing "1st" badge pattern at top-left). Hover/click reveals tooltip with details: "Champion not in PlayerName's pool. No [Class] coverage."
- Scope: Our team's pick slots. Claude's discretion on whether to add indicators on opponent picks
- Computation: Client-side reactive — compare already-loaded pool data against current draft picks signal. No server refetch on pick changes

**Opponent Tendency Sidebar (PIPE-04)**
- Location: Persistent right sidebar panel on draft page
- Data: Opponent players grouped by role with recent champions + frequency counts
- One-trick detection: Highlight when a player has disproportionate games on one champion (OTP alert)
- Champion mastery: Fetch from Riot API using opponent player's riot_puuid. Show mastery level + points alongside recent pick frequency. This balances recency with long-term champion comfort
- Visibility: Appears when the draft has an opponent set. If no opponent selected, show a prompt to select one
- Refresh: Loads once when opponent is set/changed. Server call only on opponent change, not on pick changes

**Win Condition Tracker (INTL-02)**
- Location: Panel on game plan page, below or alongside the win conditions input section
- Pass/fail logic: Uses win_condition_tag + game outcome (win/loss) as proxy. get_win_condition_stats() already implements this
- Display: All historical win condition tags shown with wins/games/percentage. Current game plan's tag highlighted if set
- Opponent filter: Optional toggle — default shows all-time stats, can filter to "vs this opponent" when game plan has an opponent set. Useful for recurring matchups
- Load timing: Fetches on page load, batched with existing game plan data query

**Intel Refresh Behavior**
- Pool warnings: Reactive client-side computation (no server calls) — already-loaded team pool data compared against draft picks signal. Updates instantly as picks change
- Opponent sidebar: Server-loaded once per opponent selection. Refetches only when opponent changes
- Win condition tracker: Loads on game plan page load. Static for the session (historical data doesn't change mid-session)
- Principle: No server refetches triggered by reactive draft pick changes. Server calls only for initial load and explicit context changes (opponent switch)

### Claude's Discretion
- Whether opponent-side pick slots get any indicator (e.g. "counters your class gap")
- Exact OTP detection threshold (e.g. >60% of games on one champion)
- Mastery display format (icons, numbers, abbreviated)
- Tooltip styling and animation
- How manual role override UI works on pick slots (dropdown, click-to-assign, drag)
- Empty state for win condition tracker when no data exists
- How to handle opponent players without a linked Riot account (no mastery data available)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PIPE-03 | Draft planner warns when a drafted champion is not in the assigned player's pool | `PoolGapWarning` struct and `compute_pool_gaps()` exist; `get_team_pools()` already loaded in draft page; new prop on `DraftBoard` to pass warnings; badge overlay in `render_pick_slot` |
| PIPE-04 | Draft planner shows opponent tendency sidebar (historical picks/bans from scouting data) | `get_opponent_intel()` + `opponent_players` Resource already wired; enhance "Their Picks" tab with frequency counting, OTP detection, mastery via new `fetch_champion_masteries()` in riot.rs |
| INTL-02 | Win condition tracker shows whether declared win conditions were achieved over time | `get_win_condition_stats()` + `get_strategy_win_rates()` server fn already implemented; new panel in game_plan.rs component; opponent-filter variant needs new db query |
</phase_requirements>

## Standard Stack

### Core (already in use — no new dependencies needed)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8 | Reactive UI framework | Project stack |
| riven | latest | Riot API client | Already in Cargo.toml for match history |
| SurrealDB | 3.x | Database | Project stack |
| Tailwind CSS v4 | 4 | Styling | Project stack |

### No New Dependencies
Champion mastery uses riven (already a dependency). Frequency counting and OTP detection are pure Rust logic. No additional crates needed.

## Architecture Patterns

### Recommended File Modifications
```
src/
├── components/draft_board.rs    # Add warning_slots prop, badge overlay in render_pick_slot
├── pages/draft.rs               # Compute pool warnings client-side, pass to DraftBoard;
│                                #   enhance "Their Picks" tab with frequency/OTP/mastery
├── pages/game_plan.rs           # Add WinConditionTracker panel below win_conditions input
├── server/riot.rs               # Add fetch_champion_masteries(puuid) function
└── server/db.rs                 # Add get_win_condition_stats_vs_opponent(db, team_id, opponent_name)
```

### Pattern 1: Pool Warning Badge Overlay (PIPE-03)

**What:** Add an optional `warning_slots` prop to `DraftBoard`. In `render_pick_slot`, when a slot has a filled champion AND a warning entry exists for that slot index, render a badge at `absolute top-0 right-0` matching the "1st" badge position/style at `top-0 left-0`.

**When to use:** Only on blue/red pick slots (not ban slots). Only our-side slots initially.

**DraftBoard prop addition:**
```rust
// Source: existing draft_board.rs pattern (lines 36-41)
#[component]
pub fn DraftBoard(
    // ... existing props ...
    #[prop(optional)] warning_slots: Option<ReadSignal<Vec<Option<String>>>>,
    // warning_slots[slot_idx] = Some("PlayerName") means show warning badge
) -> impl IntoView {
```

**Badge rendering in render_pick_slot (mirrors "1st" badge at lines 175-177):**
```rust
// Source: existing draft_board.rs line 175-177 "1st" badge pattern
{warning_slots.and_then(|ws| {
    let w = ws.get();
    w.get(slot_idx).cloned().flatten().map(|player_name| {
        view! {
            <div
                class="absolute top-0 right-0 bg-amber-500 text-white text-xs font-bold px-1 leading-tight rounded-bl z-10 cursor-help"
                title=format!("Champion not in {player_name}'s pool")
            >"!"</div>
        }
    })
})}
```

**Client-side warning computation in draft.rs:**
```rust
// Source: CONTEXT.md — "Client-side reactive — compare already-loaded pool data"
// team_pools is already a Resource<Vec<(username, role, Vec<ChampionPoolEntry>)>>
let warning_slots = Memo::new(move |_| {
    let slots = draft_slots.get();
    let pools = match team_pools.get().and_then(|r| r.ok()) {
        Some(p) => p,
        None => return vec![None::<String>; 20],
    };
    let champ_data = match champions_resource.get().and_then(|r| r.ok()) {
        Some(c) => c,
        None => return vec![None::<String>; 20],
    };
    // Build champion -> primary_role map from Data Dragon
    // For each our-side pick slot: check if champion is in that player's pool
    // Return player username if warning applies, None otherwise
    compute_slot_warnings(&slots, &pools, &champ_data, &our_side.get())
});
```

**Role-to-slot mapping (blue side our picks):**
Blue picks use slots 6, 9, 10, 17, 18 (indices from `slot_meta()`). Red picks use 7, 8, 11, 16, 19. Map in-order picks to Top/Jng/Mid/Bot/Sup roles by pick order.

### Pattern 2: Opponent Tendency Enhancement (PIPE-04)

**What:** The "Their Picks" tab already renders `opponent_players` with `recent_champions`. Enhancement adds: frequency counts, OTP badge (>60% threshold — discretionary), and mastery data from Riot API.

**New model struct (add to `OpponentPlayer` or parallel struct in draft.rs):**
```rust
// Add to draft.rs server fn response type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpponentPlayerIntel {
    pub player: OpponentPlayer,
    pub champion_frequencies: Vec<(String, u32)>, // (champion, count), sorted desc
    pub mastery_data: Vec<(String, i32, i32)>,     // (champion_name, level, points)
    pub otp_champion: Option<String>,              // champion name if OTP detected
}
```

**New server fn to replace/supplement `get_opponent_intel`:**
```rust
#[server]
pub async fn get_opponent_intel_full(
    opponent_id: String,
) -> Result<Vec<OpponentPlayerIntel>, ServerFnError> {
    // 1. Fetch opponent players (existing: db::get_opponent)
    // 2. For each player with riot_puuid: call riot::fetch_champion_masteries(puuid)
    // 3. Compute frequency counts from recent_champions
    // 4. Detect OTP: count games on most-played / total > 0.60
    // 5. Map champion_id -> name via data_dragon champion map
}
```

**New riot.rs function:**
```rust
// Source: riven docs — champion_mastery_v4().get_all_champion_masteries_by_puuid()
pub async fn fetch_champion_masteries(
    puuid: &str,
) -> Result<Vec<(i64, i32, i32)>, RiotError> {
    // Returns Vec<(champion_id, level, points)>
    let api = api();
    let masteries = api
        .champion_mastery_v4()
        .get_all_champion_masteries_by_puuid(
            riven::consts::PlatformRoute::EUW1, // or configurable
            puuid,
        )
        .await?;
    Ok(masteries.into_iter()
        .map(|m| (m.champion_id, m.champion_level, m.champion_points))
        .collect())
}
```

**Champion ID to name mapping:** Data Dragon champions have a `key` field (numeric string matching Riot's champion ID). Build a `HashMap<i64, String>` from the champion list.

**Resource wiring (refetch only on opponent change — already correct):**
```rust
// Source: existing draft.rs lines 694-702 — already keys on selected_opponent_id
let opponent_players = Resource::new(
    move || selected_opponent_id.get(),
    move |opp_id| async move {
        if opp_id.is_empty() {
            Ok(Vec::<OpponentPlayerIntel>::new())
        } else {
            get_opponent_intel_full(opp_id).await
        }
    },
);
```

### Pattern 3: Win Condition Tracker Panel (INTL-02)

**What:** New collapsible panel in `game_plan.rs` below the win conditions textarea. Uses `strategy_win_rates` which already fetches `get_win_condition_stats()`. Add opponent-filter variant.

**The `strategy_win_rates` Resource is already wired:**
```rust
// Source: game_plan.rs line 541
let strategy_win_rates = Resource::new(|| (), |_| get_strategy_win_rates());
```

**Panel placement:** Below or alongside the "How We Win" section in the editor. Renders `Vec<(String, i32, i32)>` as `(tag, total_games, wins)`.

**Win rate display:**
```rust
// For each (tag, total, wins):
let pct = if total > 0 { wins * 100 / total } else { 0 };
let is_current = win_condition_tag.get() == *tag;
// Highlight current tag with accent color
// Show: tag label | W/L bar | X/Y games | pct%
```

**Opponent filter:** Add a new server fn:
```rust
#[server]
pub async fn get_strategy_win_rates_vs_opponent(
    opponent_name: String,
) -> Result<Vec<(String, i32, i32)>, ServerFnError> {
    // Filter game_plan WHERE opponent = opponent_name
    // Join same way as get_win_condition_stats
}
```

Or add opponent_name as optional param to existing `get_win_condition_stats()` in db.rs.

**Refresh timing:** Load on page load (already done by existing Resource). No refetch needed mid-session. Opponent-filtered view fetched only when user toggles the filter.

### Anti-Patterns to Avoid

- **Reactive draft picks triggering server calls:** Pool warnings MUST use `Memo::new()` or derived signal from already-loaded data. No server fn calls inside reactive closures that track `draft_slots`.
- **Mastery API rate limits:** Fetch mastery once per opponent selection, not per player separately. Batch all players' puuids in one pass. Cache in the server response.
- **`unwrap()` in WASM event handlers:** Mastery level/points display must use `unwrap_or_default()` or `if let`.
- **Passing `&str` to `.bind()`:** Opponent-filtered SurrealDB queries must bind owned `String`.
- **OTP detection in reactive closure:** Compute OTP flag server-side in `get_opponent_intel_full`, not client-side on each render.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Champion mastery fetch | Custom Riot API HTTP calls | `riven::champion_mastery_v4()` | Rate limiting, retry logic, type safety |
| Frequency counting | Custom analytics DB query | In-memory count from `recent_champions: Vec<String>` | Data already loaded, simpler |
| Champion ID-to-name lookup | Separate DB table | Data Dragon `key` field on existing `Champion` struct | Already fetched, `key` is the numeric Riot champion ID |
| OTP detection algorithm | ML scoring | Threshold ratio: most-played / total > 0.60 | Sufficient signal, explainable to users |

**Key insight:** Most of the hard work (pool data loading, opponent data loading, win condition stats) is already implemented. This phase is primarily UI wiring and presentation enhancement, plus one new Riot API call for mastery.

## Common Pitfalls

### Pitfall 1: champion_id vs champion_name mismatch
**What goes wrong:** Riot mastery API returns `champion_id` (i64), not champion name. Data Dragon returns `key` as a numeric String field.
**Why it happens:** Two different ID systems — internal name (e.g. "MonkeyKing") vs display name ("Wukong") vs numeric key.
**How to avoid:** Build `HashMap<i64, String>` by parsing `champion.key` as i64, mapping to `champion.name`. Fetch this in the server fn alongside mastery data.
**Warning signs:** Mastery data shows but champion names are blank or mismatched.

### Pitfall 2: Reactive pool warning triggers server refetch
**What goes wrong:** If pool warning computation uses a Resource or server fn keyed on `draft_slots`, every pick change fires a server call.
**Why it happens:** Easy to accidentally create a Resource with `draft_slots.get()` as the key.
**How to avoid:** Use `Memo::new()` with only local signal reads (`draft_slots`, `team_pools`, `champions_resource`). No async involved.
**Warning signs:** Network tab shows `/api/` calls firing on every champion pick.

### Pitfall 3: `warning_slots` Memo captures stale `our_side` signal
**What goes wrong:** Warning computation uses the wrong side's slots because `our_side` was not tracked.
**Why it happens:** Forgetting to track `our_side.get()` inside the Memo body.
**How to avoid:** Call `our_side.get()` inside the Memo closure so it re-runs when side toggles.
**Warning signs:** Warnings appear on wrong team's picks when switching blue/red.

### Pitfall 4: Mastery fetch fails silently when no RIOT_API_KEY
**What goes wrong:** `fetch_champion_masteries()` panics or returns error when key is absent. This breaks the intel sidebar.
**Why it happens:** `riot::has_api_key()` exists but isn't checked before calling.
**How to avoid:** In `get_opponent_intel_full`, check `riot::has_api_key()` before attempting mastery fetch. Return empty mastery data gracefully.
**Warning signs:** Intel sidebar crashes or shows error when server has no API key.

### Pitfall 5: DraftBoard prop signature break
**What goes wrong:** Adding `warning_slots` prop breaks all existing DraftBoard usages in other pages/components.
**Why it happens:** Required props without defaults.
**How to avoid:** Annotate with `#[prop(optional)]` so existing callers don't need to pass it.

### Pitfall 6: Win condition tracker OTP threshold inconsistency
**What goes wrong:** OTP threshold defined in multiple places (server-side OTP detection and client-side display label) drift apart.
**Why it happens:** Logic split between server fn and UI.
**How to avoid:** Compute `otp_champion: Option<String>` server-side only. UI simply renders the field — no threshold logic in WASM.

## Code Examples

Verified patterns from existing codebase:

### Existing "1st" badge overlay (mirrors warning badge placement)
```rust
// Source: src/components/draft_board.rs lines 175-177
{is_first_pick.then(|| view! {
    <div class="absolute top-0 left-0 bg-accent text-accent-contrast text-xs font-bold px-1 leading-tight rounded-br">"1st"</div>
})}
// Warning badge mirrors this at top-0 right-0 with rounded-bl
```

### Resource keyed on signal for opponent refetch
```rust
// Source: src/pages/draft.rs lines 694-702
let opponent_players = Resource::new(
    move || selected_opponent_id.get(),
    move |opp_id| async move {
        if opp_id.is_empty() {
            Ok(Vec::<OpponentPlayer>::new())
        } else {
            get_opponent_intel(opp_id).await
        }
    },
);
```

### Memo for client-side derived computation (no server call)
```rust
// Pattern: Leptos 0.8 Memo — recalculates when tracked signals change
// Use for pool warning computation — NO Resource, NO async
let warning_slots: Memo<Vec<Option<String>>> = Memo::new(move |_| {
    let slots = draft_slots.get();           // tracked
    let side = our_side.get();              // tracked
    let pools = team_pools.get()            // tracked
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let champs = champions_resource.get()   // tracked
        .and_then(|r| r.ok())
        .unwrap_or_default();
    compute_slot_warnings(&slots, &pools, &champs, &side)
    // Returns Vec<Option<String>> length 20 — Some(username) means show badge
});
```

### StoredValue for champion map shared across closures
```rust
// Source: src/components/draft_board.rs line 44
let champion_map = StoredValue::new(champion_map);
// Use same pattern for champion_id_to_name map in server fn
```

### Riot API champion mastery call
```rust
// Source: riven docs — https://docs.rs/riven/latest
// PlatformRoute may need to be configurable (EUW1 is project default from riot.rs)
pub async fn fetch_champion_masteries(puuid: &str) -> Result<Vec<(i64, i32, i32)>, RiotError> {
    let api = api();
    let masteries = api
        .champion_mastery_v4()
        .get_all_champion_masteries_by_puuid(
            riven::consts::PlatformRoute::EUW1,
            puuid,
        )
        .await?;
    Ok(masteries.iter()
        .map(|m| (m.champion_id, m.champion_level, m.champion_points))
        .collect())
}
```

### Existing win condition stats server fn
```rust
// Source: src/pages/game_plan.rs lines 435-460
// Already implemented — get_strategy_win_rates() calls db::get_win_condition_stats()
// Returns Vec<(String, i32, i32)> = (tag, total_games, wins)
let strategy_win_rates = Resource::new(|| (), |_| get_strategy_win_rates());
```

### Slot index to role mapping
```rust
// Source: slot_meta() in draft_board.rs — blue picks at indices 6, 9, 10, 17, 18
// Map pick order to role (convention: 1st pick=Top, 2nd=Jng, 3rd=Mid, 4th=Bot, 5th=Sup)
fn blue_pick_slots() -> [(usize, &'static str); 5] {
    [(6, "top"), (9, "jungle"), (10, "mid"), (17, "bot"), (18, "support")]
}
fn red_pick_slots() -> [(usize, &'static str); 5] {
    [(7, "top"), (8, "jungle"), (11, "mid"), (16, "bot"), (19, "support")]
}
```

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| `type::thing()` in SurrealQL | `type::record('table', $key)` | Removed in 2.x — always use record |
| Opponent intel shows raw recent_champions list | Frequency-sorted with OTP badge | Phase 4 enhancement |
| Win rate stats exist but not shown in context | Panel inline on game plan page | Phase 4 adds the panel |
| No mastery data anywhere | Mastery level + points via Riot API | New in Phase 4 |

## Open Questions

1. **PlatformRoute for mastery API**
   - What we know: Existing `riot.rs` hardcodes `RegionalRoute::EUROPE` for match history. Mastery uses `PlatformRoute` (e.g. EUW1, NA1).
   - What's unclear: Users may be on different platforms. No platform config exists.
   - Recommendation: Default to `PlatformRoute::EUW1` matching existing pattern. Add env var `RIOT_PLATFORM` for configurability if needed; this is a discretionary call.

2. **Opponent-filtered win condition stats query**
   - What we know: `get_win_condition_stats()` in db.rs queries all game plans for team. No opponent filter.
   - What's unclear: `GamePlan` model has no `opponent` field — the opponent is on the linked `Draft`, not the plan itself.
   - Recommendation: Check `GamePlan` struct for opponent field. If absent, the opponent filter toggle may need to join through draft. Alternatively, a simpler approach: filter by opponent_name in the in-memory aggregation after fetching plans that have a linked draft_id. Research the `GamePlan` model during Wave 0.

3. **Manual role override UI for pool warnings**
   - What we know: CONTEXT.md says "manual override available per slot" — this is Claude's discretion.
   - What's unclear: How complex this should be (simple dropdown vs drag assignment).
   - Recommendation: Start with simple per-slot role dropdown (select from 5 roles) stored in a `RwSignal<HashMap<usize, String>>`. Keep it minimal for v1.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + cargo test --features ssr --lib (unit) |
| Config file | `playwright.config.ts` (root), `e2e/tests/fixtures.ts` |
| Quick run command | `cargo test --features ssr --lib` (unit) or `just e2e` (e2e) |
| Full suite command | `just e2e` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PIPE-03 | Pool warning badge appears on pick slot when champion not in player's pool | unit | `cargo test --features ssr --lib -- compute_slot_warnings` | ❌ Wave 0 |
| PIPE-03 | Draft page renders without JS errors after warning badge addition | smoke | `just e2e -- --grep "draft"` | ✅ pages.spec.ts |
| PIPE-04 | Opponent tendency shows frequency counts and OTP marker | e2e | manual inspection via pages.spec.ts | ✅ pages.spec.ts |
| PIPE-04 | Mastery fetch handles missing API key gracefully | unit | `cargo test --features ssr --lib -- opponent_intel_no_key` | ❌ Wave 0 |
| INTL-02 | Win condition tracker panel renders with strategy stats | e2e | `just e2e -- --grep "game-plan"` | ✅ pages.spec.ts |
| INTL-02 | get_win_condition_stats returns correct tag/total/wins tuples | unit | `cargo test --features ssr --lib -- win_condition_stats` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `just e2e`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/server/db.rs` test: `test_compute_slot_warnings` — unit test for the pure function mapping draft slots to pool warnings (covers PIPE-03)
- [ ] `src/server/db.rs` test: `test_win_condition_stats_empty` — edge case: no post-games linked to plans (covers INTL-02)
- [ ] `src/server/riot.rs` test: cannot unit test without live API key — acceptance via pages.spec.ts smoke + graceful degradation check

*(Existing `test_compute_pool_gaps_*` tests in db.rs already cover gap logic. New `compute_slot_warnings` is a different function mapping slots to users.)*

## Sources

### Primary (HIGH confidence)
- Source code inspection — `src/pages/draft.rs`, `src/components/draft_board.rs`, `src/pages/game_plan.rs`, `src/server/db.rs`, `src/server/riot.rs`, `src/models/opponent.rs`, `src/models/game_plan.rs` — confirmed existing patterns and infrastructure
- riven 0.0.2 docs (https://docs.rs/riven/0.0.2/riven/champion_mastery_v4/struct.ChampionMastery.html) — ChampionMastery struct fields confirmed: champion_id (i64), champion_points (i32), champion_level (i32)

### Secondary (MEDIUM confidence)
- riven latest docs (https://docs.rs/riven/latest/riven/struct.RiotApi.html) — `champion_mastery_v4().get_all_champion_masteries_by_puuid()` method confirmed via search; struct fields verified from docs

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in Cargo.toml; riven mastery API verified from docs
- Architecture: HIGH — existing code patterns directly readable from source; new work is incremental extensions
- Pitfalls: HIGH — drawn from CLAUDE.md project rules + direct code reading
- Riot mastery API signature: MEDIUM — confirmed from older docs version; should verify current method name on riven latest before implementation

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable stack, riven API unlikely to change)
