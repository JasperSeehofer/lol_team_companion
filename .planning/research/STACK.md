# Technology Stack

**Domain:** LoL Team Companion v1.2 — Solo Mode & Match Intelligence additions
**Researched:** 2026-03-25
**Confidence:** HIGH (all key findings verified against official docs and crates.io)

---

## Context

The stack is locked: Rust nightly, Leptos 0.8, Axum 0.8, SurrealDB 3.x, Tailwind CSS v4.
This research answers: "what additions or new API surfaces does the solo mode & match
intelligence milestone require?" Do not re-evaluate the core stack.

---

## Existing Stack — Unchanged

| Technology | Cargo.toml Version | Status |
|------------|-------------------|--------|
| Leptos | 0.8 | Unchanged |
| Axum | 0.8 | Unchanged |
| SurrealDB | 3 (SurrealKV) | Unchanged |
| Tailwind CSS | v4 standalone | Unchanged |
| riven | ^2 (resolves 2.78.0) | **Gains new endpoints — no version bump needed** |
| reqwest | 0.12 | Unchanged |
| chrono | 0.4 | Unchanged |
| serde / serde_json | 1 | Unchanged |

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| leptos-chartistry | 0.2.3 | SVG line/bar charts in Leptos components | Only charting library targeting Leptos 0.8 directly with an `ssr` feature flag. Renders pure SVG (no canvas, no JS runtime). Supports line charts with a `Timestamps` axis for time-series (CS/gold/XP over game minutes). 0.2.3 released 2026-01-23, no open SSR compatibility issues. Zero npm dependency — matches project constraint. |

This is the **only new crate** needed for the milestone. Ranked data, match detail, and timeline
data are all accessible through the already-pinned `riven = { version = "2" }` at its current
2.78.0 resolved version.

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| riven (existing) | ^2 | Riot API — now also ranked data and match timeline | Use `league_v4().get_league_entries_by_puuid()` for ranked; `match_v5().get_timeline()` for timeline. Both new to v1.2 but the crate is already in Cargo.toml. |

### Development Tools

No new tools needed. Existing `cargo leptos watch`, `just`, and Playwright suite are sufficient.

---

## Riot API Endpoints — Full Reference for v1.2

### Ranked Data: `league_v4`

```rust
// Get ranked entries for all queues by PUUID (available in riven 2.78.0)
api.league_v4()
    .get_league_entries_by_puuid(PlatformRoute::EUW1, puuid)
    .await?
// Returns: Vec<LeagueEntry>
```

**LeagueEntry fields:**

| Field | Type | Notes |
|-------|------|-------|
| `queue_type` | `String` | `"RANKED_SOLO_5x5"` or `"RANKED_FLEX_SR"` |
| `tier` | `Option<Tier>` | IRON through CHALLENGER |
| `rank` | `Option<Division>` | I, II, III, IV |
| `league_points` | `i32` | Current LP |
| `wins` | `i32` | Season wins |
| `losses` | `i32` | Season losses |
| `hot_streak` | `bool` | Win streak in progress |
| `veteran` | `bool` | Long time in current tier |
| `fresh_blood` | `bool` | Recently promoted |
| `inactive` | `bool` | Ranked decay warning |
| `mini_series` | `Option<MiniSeries>` | Promotion series data |

**LP history gap:** Riot does not expose historical LP over time. Snapshot it yourself:
after each ranked fetch, persist `(user_id, timestamp, lp, tier, division, wins, losses)`
to a new `ranked_snapshot` SurrealDB table. Derive LP delta from successive snapshots.
LP trend chart becomes available after the second stored snapshot.

**Route note:** `league_v4` uses `PlatformRoute` (e.g. `EUW1`), not `RegionalRoute`.
Consistent with the existing `champion_mastery_v4` call pattern in `riot.rs`.

### Match Detail: `match_v5().get_match()` (already used)

The existing `fetch_match_history()` in `riot.rs` only stores one participant's stats.
For the full 10-player match detail view, iterate `m.info.participants` (already fetched).

**Participant fields relevant to v1.2 (beyond what MatchData already captures):**

| Field | Notes |
|-------|-------|
| `team_id` | 100 = blue, 200 = red |
| `team_position` | `"TOP"`, `"JUNGLE"`, `"MIDDLE"`, `"BOTTOM"`, `"UTILITY"` |
| `item0`–`item6` | Item IDs; look up icons via Data Dragon CDN (already integrated) |
| `gold_earned`, `gold_spent` | For gold share breakdown |
| `total_damage_dealt_to_champions` | Already captured in MatchData |
| `total_damage_taken` | Tank performance metric |
| `total_time_cc_dealt` | CC contribution |
| `wards_placed`, `wards_killed`, `detector_wards_placed` | Vision breakdown |
| `total_minions_killed`, `neutral_minions_killed` | CS (already captured combined) |
| `champ_level` | Final level at game end |
| `summoner1Id`, `summoner2Id` | Summoner spell IDs |

**No extra API call needed:** full 10-player data is already in the `get_match` response.
The v1.2 work is extracting and storing/displaying it, not adding new API calls.

### Match Timeline: `match_v5().get_timeline()`

```rust
// Returns: Option<MatchTimeline> — ~200KB per game
api.match_v5()
    .get_timeline(RegionalRoute::EUROPE, match_id)
    .await?
```

**Structure:**
```
MatchTimeline.info
  ├─ frame_interval: i32          (60000ms = 1 minute between frames)
  ├─ frames: Vec<MatchTimelineInfoFrame>
  │    ├─ timestamp: i32
  │    ├─ participant_frames      (per-minute: gold, xp, cs, position per player)
  │    └─ events: Vec<MatchTimelineInfoFrameEvent>
  └─ participants: Option<Vec<...>>
```

**Event types available in `MatchTimelineInfoFrameEvent.type` (String):**

| Event | Key Fields | v1.2 Use |
|-------|-----------|---------|
| `ELITE_MONSTER_KILL` | `monster_type`, `monster_sub_type`, `killer_team_id`, `timestamp` | Objectives timeline (dragon, baron, herald) |
| `BUILDING_KILL` | `building_type`, `tower_type`, `lane_type`, `team_id`, `timestamp` | Structures timeline |
| `DRAGON_SOUL_GIVEN` | `name`, `team_id` | Dragon soul event marker |
| `TURRET_PLATE_DESTROYED` | `lane_type`, `team_id` | Early-game pressure |
| `CHAMPION_KILL` | `killer_id`, `victim_id`, `assisting_participant_ids`, `position`, `bounty`, `shutdown_bounty` | Kill timeline |
| `CHAMPION_SPECIAL_KILL` | `kill_type`, `multi_kill_length`, `kill_streak_length` | Multi-kills, first blood |
| `ITEM_PURCHASED` | `participant_id`, `item_id`, `timestamp` | Build path reconstruction |
| `LEVEL_UP` | `participant_id`, `level` | Level timing |
| `WARD_PLACED` / `WARD_KILL` | `creator_id`, `ward_type` | Vision control |
| `GAME_END` | `winning_team` | Match conclusion |

**`participant_frames` at each minute provides:**
- `current_gold`, `total_gold`, `xp`, `level`
- `minions_killed`, `jungle_minions_killed` (CS-per-minute derivation)
- `position: {x, y}` (heatmap data if desired in future)
- `damage_stats.total_damage_done_to_champions` (damage over time)

**Critical:** Fetch timeline on-demand only (when user opens a specific match detail view).
Do NOT batch-fetch timelines during match list loading — at ~200KB each, 20 matches = 4MB
plus 20 extra API calls, hitting rate limits instantly.

---

## Charting — leptos-chartistry

**Use for:** CS/min over time, gold diff over time, LP trend, win rate trend per champion.

**Chart types available (v0.2.3):**
- Line charts with multiple series, linear/step interpolation, point markers
- Bar charts
- Stacked lines
- `Timestamps` struct for time-series X-axis (maps to game minutes in timeline data)
- Interactive tooltips on hover (WASM only; static SVG renders on SSR)

**Integration:** Add as shared dep (not SSR-only) so SSR renders the static SVG. The `ssr`
feature must be enabled for server-side rendering path.

**Where NOT to use:** The existing hand-rolled SVG tree graph in `tree_graph.rs`, simple
CSS-width bar rows in `analytics.rs` stats tables. Those are simpler as inline SVG/CSS.
Chartistry is worth adding only for multi-series time-series charts with tooltips.

---

## Installation

```toml
# Cargo.toml [dependencies] — add as shared (both SSR and hydrate targets)
leptos-chartistry = { version = "0.2", features = ["ssr"] }
```

No changes to `[features]` sections required — the dep is shared, not gated.

No new `web-sys` features needed (chartistry renders SVG, not canvas).

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| leptos-chartistry | apexcharts-rs (JS wrapper) | Never in this project — pulls in a JS runtime, conflicts with no-npm constraint |
| leptos-chartistry | plotters | If you need bitmap/PNG output — not applicable here; plotters has no Leptos SSR path |
| leptos-chartistry | leptos_charts (Panaetius) | Only bar and pie charts; acceptable if time-series line charts are not needed |
| leptos-chartistry | Hand-rolled SVG | Acceptable for simple single-metric bars (CS% row = CSS width); use chartistry only for multi-line time-series |
| `get_league_entries_by_puuid` | `get_league_entries_for_summoner` | Never — requires an extra `summoner_v4` round-trip to convert PUUID to summoner_id; PUUID method is direct |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| D3.js / Chart.js via WASM bindings | JS interop bloats WASM bundle, breaks SSR path, adds npm dependency | leptos-chartistry pure-Rust SVG |
| Background/polling LP sync | Riot API rate limits (dev: 100 req/2min); no background jobs in single-binary architecture | On-demand fetch triggered by user action (match refresh button) |
| Fetching timeline for every match in list | ~200KB per call; 20 matches = 4MB + 20 extra API calls = rate-limit abuse | Lazy per-match fetch when user clicks "view detail" |
| Storing raw timeline JSON in SurrealDB | Too large; schema drift risk if Riot changes the structure | Cache only derived data (objectives list, per-minute gold/CS arrays) after first parse |
| `get_league_entries_for_summoner` | Extra API call to resolve summoner_id from PUUID | `get_league_entries_by_puuid` |
| A separate `riot_cache.rs` caching layer for v1.2 | Premature; existing on-demand fetch + SurrealDB persistence is sufficient | Store fetched data in SurrealDB records with `fetched_at` timestamp; re-fetch when stale |

---

## Stack Patterns by Variant

**For CS/gold-over-time line charts (performance breakdown):**
- Fetch `get_timeline()` on demand when user opens match detail
- Extract `participant_frames` at each frame timestamp → `Vec<(f64, f64)>` (minute, value)
- Pass to leptos-chartistry `Series` with `Timestamps` X-axis
- Gate interactive tooltips with `#[cfg(feature = "hydrate")]`; static SVG renders on SSR

**For LP history trend chart:**
- Riot does not expose LP history
- After each `get_league_entries_by_puuid()` call, persist snapshot to `ranked_snapshot` table
- Query `SELECT * FROM ranked_snapshot WHERE user = $user ORDER BY timestamp ASC`
- Derive `lp_delta = current_lp - previous_lp` per game; render as line chart over time
- On first use (no history), show only current rank badge; chart appears after ≥2 snapshots

**For champion performance trends (win rate, KDA over N games):**
- No new API calls: aggregate from already-stored `player_match` table records
- Use Rust-side `HashMap<champion, Stats>` aggregation (same pattern as `analytics.rs`)
- Return as `Vec<ChampionTrendEntry>` from a server function

**For objectives timeline display:**
- Filter timeline events for `ELITE_MONSTER_KILL`, `BUILDING_KILL`, `DRAGON_SOUL_GIVEN`
- Map to a `Vec<ObjectiveEvent> { timestamp_secs, team, event_type, description }` struct
- Render as a horizontal timeline SVG (custom or chartistry guide-line markers)

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| leptos-chartistry 0.2.3 | leptos 0.8 | Verified — depends on `leptos = "0.8"`; ssr feature confirmed |
| riven ^2 (2.78.0) | reqwest 0.12, tokio 1 | Already in use; `get_league_entries_by_puuid` available in latest 2.x |
| `get_timeline` | `RegionalRoute::EUROPE` | Same route as existing `get_match` calls — no change |
| `get_league_entries_by_puuid` | `PlatformRoute::EUW1` | Same route as existing `champion_mastery_v4` calls — no change |

---

## Sources

- https://docs.rs/riven/latest/riven/ — version 2.78.0 confirmed; `get_league_entries_by_puuid` confirmed — HIGH confidence
- https://docs.rs/riven/latest/riven/endpoints/struct.LeagueV4.html — all LeagueV4 methods including `get_league_entries_by_puuid` — HIGH confidence
- https://docs.rs/riven/2.24.0/riven/models/league_v4/struct.LeagueEntry.html — LeagueEntry fields verified — HIGH confidence
- https://docs.rs/riven/latest/riven/endpoints/struct.MatchV5.html — `get_timeline` signature confirmed — HIGH confidence
- https://docs.rs/riven/2.24.0/riven/models/match_v5/struct.MatchTimelineInfoFrameEvent.html — all 20 event types verified — HIGH confidence
- https://docs.rs/riven/2.24.0/riven/models/match_v5/struct.MatchTimelineInfoFrame.html — frame structure (participant_frames, events, timestamp) — HIGH confidence
- https://lib.rs/crates/leptos-chartistry — version 0.2.3, January 2026, leptos 0.8 dependency — HIGH confidence
- https://docs.rs/leptos-chartistry/latest/leptos_chartistry/ — chart types, Timestamps struct, SSR feature confirmed — HIGH confidence
- https://github.com/feral-dot-io/leptos-chartistry/issues — no open SSR or Leptos 0.8 compatibility issues — MEDIUM confidence
- Codebase: `src/server/riot.rs`, `Cargo.toml` — direct source of truth for existing patterns — HIGH confidence

---
*Stack research for: LoL Team Companion v1.2 — Solo Mode & Match Intelligence*
*Researched: 2026-03-25*
