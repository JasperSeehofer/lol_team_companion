# Phase 10: Opponents Redesign - Research

**Researched:** 2026-03-23
**Domain:** Leptos 0.8 UI redesign, Riot API data enrichment, SurrealDB schema extension
**Confidence:** HIGH — all findings come from reading the actual codebase

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**New Opponent Form (OPP-01)**
- D-01: Replace the current "name + add player one-at-a-time" flow with an inline 5-role form: team name at top, then 5 pre-labeled role rows (Top/Jg/Mid/Bot/Sup) each with a single Riot ID field (`Name#Tag` format)
- D-02: Saving creates the opponent + all 5 player slots at once via a single server function
- D-03: Partial entry allowed — any combination of filled/empty Riot ID fields is valid. Empty roles still create player slots (to be filled later)
- D-04: Single `Name#Tag` input per role (not split fields). Validates on `#` separator
- D-05: The detail panel for existing opponents shows the same 5-role layout, editable, with current data pre-filled

**Player Card Design (OPP-02, OPP-03)**
- D-06: OTP warning badge displayed in the card header next to the role label — orange/yellow `⚠ OTP: {champion}` badge. Triggers when >60% of recent games are on a single champion
- D-07: Champion mastery shown as level badges on champion pills: `Ahri M7`, `Zed M5`, etc. Champions sorted by mastery points descending
- D-08: Data recency shown as a relative time badge next to the summoner name: "Last fetched: 2d ago". Goes stale-orange after 7+ days. Uses existing `format_timestamp` helper
- D-09: Need to add `last_fetched` datetime field to `opponent_player` schema and model

**Pool Analysis (OPP-04)**
- D-10: Collapsible "Pool Analysis" section below the champion pills on each player card, collapsed by default
- D-11: Shows: pool size (X champions), role flexibility (weighted distribution), and comfort picks (top 3 most-played with percentages)
- D-12: Role flexibility uses weighted role distribution from match history — shows percentage per role (e.g., "Mid 60% / Top 25% / Sup 15%"). Requires storing per-match role data from Riot API `teamPosition` field
- D-13: Need to extend `opponent_player` model/schema to store per-match role data (or derive from enriched match data)

**Auto-Fetch Behavior (OPP-01)**
- D-14: "Save & Fetch" button saves the opponent and immediately kicks off Riot API fetches for all players with Riot IDs filled in
- D-15: Per-player status indicators during fetch: spinner while loading, green check on success, red X with error message on failure. Successfully fetched players keep their data; failed ones show specific error
- D-16: Individual player re-fetch via a small refresh icon button on each player card
- D-17: Team-level "Refresh All" button on the opponent detail panel header to re-fetch all 5 players at once
- D-18: Fetch pulls both recent match champions and mastery data (uses existing `fetch_player_champions` + `fetch_champion_masteries` from `riot.rs`)
- D-19: Fetch also needs to pull `teamPosition` per match to compute role distribution for pool analysis

### Claude's Discretion
- Exact card layout dimensions and spacing within the 5-role form
- Animation/transition for collapsible pool analysis section
- How many recent matches to fetch for role distribution (20 is current default for champion fetch)
- Whether to parallelize the 5 player fetches or run sequentially (rate limit consideration)
- Exact stale threshold for data recency coloring (7 days suggested, but tunable)
- Whether `last_fetched` updates on failed fetch attempts or only on success

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OPP-01 | New opponent form asks for team name and provides 5 role fields (top/jg/mid/bot/sup) with Riot ID input; on save, auto-fetches champion data from Riot API | New `create_opponent_with_players` server fn replaces separate `create_opponent` + `add_player` flow; `fetch_player_intel` new fn handles combined champion + mastery + role fetch |
| OPP-02 | Opponent players show OTP warning badge when >60% of recent games are on a single champion | `OpponentPlayerIntel.otp_champion` field already exists; computation in `compute_intel()` helper; wire into `PlayerCard` render |
| OPP-03 | Opponent player cards display champion mastery level badges and data recency indicator (last fetched date) | `mastery_data` field in `OpponentPlayerIntel` already has `(name, level, points)` tuples; need `last_fetched` schema field and `format_timestamp` reuse |
| OPP-04 | Each opponent player has a champion pool depth/breadth analysis showing pool size, role flexibility, and comfort picks | `champion_frequencies` field in `OpponentPlayerIntel` covers pool size + comfort picks; role distribution needs new `role_distribution` schema field storing `Vec<(String, u32)>` from `teamPosition` in match data |
</phase_requirements>

---

## Summary

Phase 10 is a focused redesign of the existing `/opponents` page. The current implementation has a functional but incomplete foundation — `OpponentPlayer` model exists, `fetch_player_champions` fetches match history, and `fetch_champion_masteries` fetches mastery data. The `OpponentPlayerIntel` struct already models OTP detection and mastery badges conceptually but the computation is not wired up.

The work falls into four layers: (1) schema and model changes to add `last_fetched` and `role_distribution` fields; (2) Riot API extension to `fetch_player_champions` to also capture `teamPosition` per match and a new combined `fetch_player_intel` server fn; (3) `create_opponent` server fn replacement to accept 5 role slots at creation time; and (4) UI redesign replacing the one-at-a-time player add flow with a 5-role grid and enriched player cards showing OTP badges, mastery pills, recency indicators, and collapsible pool analysis.

No new crate dependencies are needed. The riven crate already exposes `team_position: String` on match participants (verified in riven-2.78.0). The `format_timestamp` helper in `models/utils.rs` is ready to use for recency display.

**Primary recommendation:** Build bottom-up: schema → db functions → server fns → UI. The most involved change is the combined fetch fn that replaces `fetch_champions` (currently fetches champion names only) with a fn that in a single PUUID → matches loop captures champion names, per-match positions, and then fetches mastery data.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8 | Reactive UI components | Project stack, all pages use it |
| SurrealDB | 3.x (SurrealKV) | Persistence | Project stack |
| riven | 2.78.0 | Riot API client | Already used in riot.rs; `team_position` available on match participants |
| chrono | (in Cargo.toml) | Datetime for `last_fetched` | Already used in utils.rs |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `format_timestamp` helper | internal | Relative time display for "Last fetched: Xd ago" | Reuse from `models/utils.rs` directly |
| `ToastContext` | internal | User feedback for save/fetch operations | Already in opponents.rs and all other pages |
| `spawn_local` | Leptos task | Async event handlers | Required for all async button clicks |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate fetch per field (champion name vs mastery vs role) | Combined single-pass loop | Combined is fewer API round-trips; same loop already fetches match data so role extraction is free |
| Parallel 5-player fetches | Sequential fetches | Riot API development keys are rate-limited at 20 req/s; sequential is safer; parallel could be offered as an optimization guarded by count check |

**Installation:** No new dependencies required.

---

## Architecture Patterns

### Recommended Project Structure

No new files needed. All changes go into existing files:

```
src/
├── pages/opponents.rs        # Full redesign of OpponentsPage, OpponentDetail, PlayerCard
├── models/opponent.rs        # Add last_fetched + role_distribution to OpponentPlayer; compute_intel() helper
├── server/db.rs              # New: create_opponent_with_players, update_opponent_player_intel
│                             # Modified: update_opponent_player_champions (add last_fetched, role_distribution)
├── server/riot.rs            # New: fetch_player_intel() returning combined champion + role + mastery data
└── schema.surql              # Add last_fetched, role_distribution, mastery_data fields to opponent_player
```

### Pattern 1: 5-Role Form State (new opponent creation)

The new creation flow uses a fixed array of 5 role signals rather than a dynamic list.

```rust
// One RwSignal per role slot for the creation form
let role_inputs: [RwSignal<String>; 5] = [
    RwSignal::new(String::new()), // top
    RwSignal::new(String::new()), // jungle
    RwSignal::new(String::new()), // mid
    RwSignal::new(String::new()), // bot
    RwSignal::new(String::new()), // support
];
let team_name_input: RwSignal<String> = RwSignal::new(String::new());
```

When "Save & Fetch" is clicked, collect all 5 values as `Vec<(String, String)>` (role, riot_id) and call a new `create_opponent_with_players` server fn. The fn creates the opponent record and 5 player records in one transaction, then returns the opponent ID and the 5 player IDs for use in subsequent fetch calls.

### Pattern 2: Per-Player Fetch State

Each player card needs its own fetch status signal. For the new creation flow (which fetches all 5 at once), the status signals live in the `OpponentDetail` component:

```rust
// In OpponentDetail: per-player fetch state
let fetch_states: Vec<RwSignal<FetchState>> = (0..5)
    .map(|_| RwSignal::new(FetchState::Idle))
    .collect();

#[derive(Clone, PartialEq)]
enum FetchState {
    Idle,
    Fetching,
    Success,
    Error(String),
}
```

For individual re-fetch (D-16), the `PlayerCard` component keeps its own `fetching: RwSignal<bool>` + `fetch_error: RwSignal<Option<String>>` (same pattern already in the existing `PlayerCard`).

### Pattern 3: Intel Computation (OTP + Pool Analysis)

The existing `OpponentPlayerIntel` struct is the right shape. Add a `compute_intel` free function in `models/opponent.rs`:

```rust
impl OpponentPlayer {
    /// Compute enriched intel from stored data. Pure function — no async.
    pub fn compute_intel(&self) -> OpponentPlayerIntel {
        // champion_frequencies: count occurrences in recent_champions
        let mut freq: HashMap<String, u32> = HashMap::new();
        for c in &self.recent_champions {
            *freq.entry(c.clone()).or_insert(0) += 1;
        }
        let total = self.recent_champions.len() as f32;
        let mut sorted_freq: Vec<(String, u32)> = freq.into_iter().collect();
        sorted_freq.sort_by(|a, b| b.1.cmp(&a.1));

        // OTP: top champion > 60% of games
        let otp = if total > 0.0 {
            sorted_freq.first()
                .filter(|(_, cnt)| (*cnt as f32 / total) > 0.6)
                .map(|(name, _)| name.clone())
        } else { None };

        OpponentPlayerIntel {
            player: self.clone(),
            champion_frequencies: sorted_freq,
            mastery_data: self.mastery_data.clone(),
            otp_champion: otp,
        }
    }
}
```

This runs client-side in the view — no server round-trip needed for display. The server fn stores raw data; the component computes derived display data.

### Pattern 4: Combined Riot Fetch Function

Replace `fetch_player_champions` usage with a new `fetch_player_intel` that captures all needed data in one match-history loop:

```rust
pub struct PlayerIntel {
    pub recent_champions: Vec<String>,          // unique champions seen
    pub champion_with_role: Vec<(String, String)>, // (champion, team_position) per match
    pub mastery_data: Vec<(String, i32, i32)>,   // from fetch_champion_masteries
}

pub async fn fetch_player_intel(puuid: &str, match_count: usize) -> Result<PlayerIntel, RiotError> {
    // 1. Fetch match IDs
    // 2. For each match: extract champion_name + p.team_position
    // 3. After match loop: call fetch_champion_masteries(puuid)
    // Return all three collected datasets
}
```

The `role_distribution` stored in DB is derived from `champion_with_role`: count occurrences of each `team_position` string and store as `Vec<(String, u32)>`.

### Pattern 5: Schema Extension (additive only)

All new fields use `DEFINE FIELD IF NOT EXISTS` (CLAUDE.md rule 30):

```sql
DEFINE FIELD IF NOT EXISTS last_fetched ON opponent_player TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS role_distribution ON opponent_player TYPE array DEFAULT [];
DEFINE FIELD IF NOT EXISTS mastery_data ON opponent_player TYPE array DEFAULT [];
```

`role_distribution` stores as `array` of arrays `[[role, count], ...]`. Alternatively, a SCHEMAFULL approach defines it as `array<array<string>>` — but SurrealDB 3.x is flexible on nested array element types. Storing as `Vec<Vec<String>>` on the Rust side (serialized via serde) or as a JSON-serialized string is the safest path given SurrealDB's nested array typing constraints.

**Recommended approach:** Store `role_distribution` as a JSON string field (`TYPE option<string>`) in the DB and deserialize in the `DbOpponentPlayer → OpponentPlayer` conversion. This sidesteps SurrealDB nested array typing entirely and is consistent with the `actions_json`/`comments_json` pattern in `draft.rs` (CLAUDE.md rule 33).

Similarly for `mastery_data`: store as JSON string rather than nested array.

### Pattern 6: Relative Time + Stale Coloring (D-08)

`format_timestamp` in `models/utils.rs` takes `&str` and returns a human-readable relative string. For "Last fetched: 2d ago" display, parse the `last_fetched` field and call `format_timestamp`. For stale-orange coloring after 7 days, compute the age in the view:

```rust
// In PlayerCard view:
{move || {
    let fetched = player.last_fetched.as_deref().unwrap_or("");
    let display = format_timestamp(fetched);
    let is_stale = /* parse fetched and check > 7 days */;
    let color = if is_stale { "text-orange-400" } else { "text-muted" };
    view! { <span class=format!("text-xs {color}")>"Last fetched: "{display}</span> }
}}
```

The stale check needs chrono on the WASM target. Chrono is already a dependency (used in utils.rs). The stale check is a pure age comparison — safe to compile for `hydrate` target.

### Anti-Patterns to Avoid

- **Don't add `teamPosition` to `recent_champions: Vec<String>`** — `recent_champions` is a flat list of unique champion names used by the draft intel sidebar (Phase 9 depends on this field). Adding role data would break downstream consumers.
- **Don't call mastery fetch per-match** — `fetch_champion_masteries` is one API call for all masteries. Call it once after the match loop, not inside the loop.
- **Don't return `Err` from server fn when player has no Riot ID** — return `Ok(())` and leave data unchanged (CLAUDE.md rule 44 pattern).
- **Don't use `attr:value` on controlled inputs** — use `prop:value` for all pre-filled role input fields (CLAUDE.md rule 21).
- **Don't `.unwrap()` in event handlers** — use `if let Some(...)` or `.unwrap_or_default()` (CLAUDE.md rule 35).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Relative time display | Custom timestamp formatter | `format_timestamp` from `models/utils.rs` | Already handles all cases: "just now", "X min ago", "X hours ago", absolute dates |
| Toast notifications | Custom feedback system | `ToastContext` + `ToastKind` | Already wired to `ToastProvider` in app shell |
| Champion mastery data | Custom mastery API | `fetch_champion_masteries(puuid)` in `riot.rs` | Returns `(name, level, points)` tuples already |
| PUUID lookup | Custom account API | `get_puuid(game_name, tag_line)` in `riot.rs` | Already handles regional routing |
| Async event handling | Blocking fetch | `spawn_local` | Required pattern for WASM async calls |

---

## Common Pitfalls

### Pitfall 1: `recent_champions` is consumed by Draft Intel
**What goes wrong:** The draft intel sidebar in Phase 9 reads `OpponentPlayer.recent_champions` directly to show which champions an opponent plays. If `recent_champions` is refactored to include role data (e.g., changing to `Vec<(String, String)>`), the draft intel panel breaks.
**Why it happens:** `recent_champions: Vec<String>` is in the shared `OpponentPlayer` model; the draft page server fn fetches it alongside the opponent data.
**How to avoid:** Keep `recent_champions: Vec<String>` unchanged. Add new fields (`role_distribution`, `mastery_data`) as separate schema columns. The DB conversion function `DbOpponentPlayer → OpponentPlayer` just gets new optional fields.
**Warning signs:** Compilation errors in `src/pages/draft.rs` when `recent_champions` type changes.

### Pitfall 2: SurrealDB Nested Array Typing
**What goes wrong:** Defining `role_distribution ON opponent_player TYPE array<array<string>>` may cause unexpected behavior when the inner arrays contain mixed types or when SurrealDB deserializes into Rust types.
**Why it happens:** SurrealDB 3.x nested array types interact with serde in ways that require exactly matching Rust types. The `DbOpponentPlayer` struct must match the schema exactly.
**How to avoid:** Store `role_distribution` and `mastery_data` as JSON strings (`TYPE option<string>`), serialize/deserialize manually in the DB conversion. Consistent with the `actions_json` pattern documented in CLAUDE.md rule 33.
**Warning signs:** Deserialization errors at runtime with "failed to deserialize" in logs.

### Pitfall 3: Riot API Rate Limits During 5-Player Fetch
**What goes wrong:** Fetching 5 players simultaneously (20 match histories × 5 = 100 API calls) hits the development key rate limit (20 req/s, 100 req/2min).
**Why it happens:** `fetch_player_intel` is called 5 times concurrently via `spawn_local` without throttling.
**How to avoid:** Run the 5 player fetches sequentially in a single `spawn_local` block. Per-player status updates are shown to the user as each completes. This is the safest approach for dev keys and still provides good UX with the per-player status indicators (D-15).
**Warning signs:** 429 errors in the fetch response for the 2nd–5th player when fetching in parallel.

### Pitfall 4: Server Function Returning `Err` for Empty Riot ID
**What goes wrong:** On "Save & Fetch", players with no Riot ID filled in would cause the server fn to error if `get_puuid` is called with an empty string.
**Why it happens:** The existing `fetch_champions` server fn returns early with `Err` if format validation fails. With 5 players, any empty Riot ID field must simply skip the fetch.
**How to avoid:** In the new combined fetch server fn, skip fetch for players whose `riot_summoner_name` is `None` or empty. Return `Ok(())` for those slots. The status indicator for those slots stays as `Idle` (not an error).
**Warning signs:** "Summoner name must be in Name#Tag format" error appearing for empty slots.

### Pitfall 5: `last_fetched` Field Missing from `OpponentPlayer` Causes Compile Error
**What goes wrong:** Adding `last_fetched` to `schema.surql` but forgetting to add it to `DbOpponentPlayer` in `db.rs` causes a deserialization panic at runtime (SurrealDB returns the field but the struct doesn't have it).
**Why it happens:** `DbOpponentPlayer` is a schemafull struct with `SurrealValue` derive. Extra fields in the DB response that aren't in the struct may cause serde errors.
**How to avoid:** Add `last_fetched: Option<String>` to `DbOpponentPlayer` AND `OpponentPlayer` model in the same PR. The schema, DB struct, model struct, and server fn must all be updated atomically.

### Pitfall 6: `collect_view()` vs `.collect::<Vec<_>>()`
**What goes wrong:** Building the 5-role form rows with `.collect::<Vec<_>>()` in a `view!` macro causes type errors.
**Why it happens:** Leptos view fragments need `.collect_view()` (CLAUDE.md rule 25).
**How to avoid:** Always use `.collect_view()` at the end of iterators inside `view!`.

---

## Code Examples

### Role Distribution Computation from Champion+Role Data
```rust
// Source: Derived from existing fetch_player_champions pattern in riot.rs
// In fetch_player_intel: accumulate per-match role data
let mut role_counts: HashMap<String, u32> = HashMap::new();
for (_champion, position) in &champion_with_role {
    if !position.is_empty() {
        *role_counts.entry(position.clone()).or_insert(0) += 1;
    }
}
// Riot API team_position values: "TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"
// Map to app role names when storing
```

### OTP Detection (60% threshold)
```rust
// Source: CONTEXT.md D-06, computed from champion_frequencies
let total_games = recent_champions.len() as f32;
let otp_champion = if total_games > 0.0 {
    // champion_frequencies is sorted desc by count
    champion_frequencies.first()
        .filter(|(_, cnt)| (*cnt as f32 / total_games) > 0.6)
        .map(|(name, _)| name.clone())
} else {
    None
};
```

### Mastery Badge Rendering
```rust
// Source: CONTEXT.md D-07
// champion pills: "Ahri M7", "Zed M5" sorted by mastery points desc
// mastery_data: Vec<(String, i32, i32)> = (name, level, points)
{champs_with_mastery.into_iter().map(|(name, level)| {
    view! {
        <span class="text-xs bg-surface border border-divider/50 text-secondary rounded px-2 py-0.5">
            {format!("{name} M{level}")}
        </span>
    }
}).collect_view()}
```

### Stale Recency Check
```rust
// Source: Derived from models/utils.rs chrono usage
// In view, compute stale flag from last_fetched ISO string
fn is_stale(last_fetched: &str) -> bool {
    last_fetched.parse::<chrono::DateTime<chrono::Utc>>()
        .map(|dt| {
            let age = chrono::Utc::now().signed_duration_since(dt);
            age.num_days() >= 7
        })
        .unwrap_or(false)
}
```

### SurrealDB: Store role_distribution as JSON String
```rust
// Source: CLAUDE.md rule 33 pattern (actions_json/comments_json in draft.rs)
// Serialize before write:
let role_dist_json = serde_json::to_string(&role_distribution).unwrap_or_default();

// Deserialize in DbOpponentPlayer -> OpponentPlayer conversion:
let role_distribution: Vec<(String, u32)> = p.role_distribution_json
    .as_deref()
    .and_then(|s| serde_json::from_str(s).ok())
    .unwrap_or_default();
```

### Create Opponent with 5 Players (single transaction)
```rust
// Source: CLAUDE.md rule 31 (BEGIN/COMMIT for multi-step writes)
// Create opponent + 5 player slots atomically
db.query(
    "BEGIN TRANSACTION;
     CREATE opponent SET name = $name, team = type::record('team', $team_key), notes = $notes;
     -- Then 5 CREATE opponent_player statements for each role
     COMMIT TRANSACTION;"
)
.bind(...)
.await?.check()?;
```
Note: SurrealDB `BEGIN TRANSACTION` returns the results of each statement. Use batched query with indexed `take()` calls to retrieve the new IDs.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `add_player` one-at-a-time | 5-role form creates all slots at once | Phase 10 | Creates opponent + players in single server call |
| Champion fetch returns names only | Combined fetch returns names + roles + mastery | Phase 10 | Enables pool analysis and mastery badges |
| `recent_champions` is only stored field | Add `last_fetched`, `role_distribution_json`, `mastery_data_json` | Phase 10 | Persists enriched intel across page reloads |

**Deprecated/outdated after this phase:**
- `add_player` server fn: replaced by the 5-role creation flow. The fn can remain for backward-compat but is not called from the redesigned UI.
- `fetch_champions` server fn: replaced by `fetch_player_intel`. Can remain but is superseded.
- `save_player` server fn: still needed for editing existing player Riot IDs via the detail panel (D-05).

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond project's own code + riven crate already in Cargo.toml)

The Riot API key (`RIOT_API_KEY`) is required at runtime but is already handled by the existing `riot::has_api_key()` guard. The opponents page currently shows an error toast when the key is absent — same behavior continues.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (unit) + Playwright (e2e) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cd e2e && npx playwright test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OPP-01 | 5-role form renders; save creates opponent + 5 players | e2e smoke | `cd e2e && npx playwright test audit-misc-pages.spec.ts -g "opponents"` | Existing (basic load check) |
| OPP-02 | OTP badge appears for players with >60% single-champion games | unit | `cargo test --features ssr --lib -- test_otp_detection` | Wave 0 gap |
| OPP-03 | Mastery badges render; stale coloring applies after 7 days | unit | `cargo test --features ssr --lib -- test_stale_detection` | Wave 0 gap |
| OPP-04 | Pool analysis section shows pool size, role flex, comfort picks | unit | `cargo test --features ssr --lib -- test_compute_intel` | Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo test --features ssr --lib && cd e2e && npx playwright test audit-misc-pages.spec.ts`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit tests for `compute_intel()` covering OTP threshold edge cases (exactly 60%, just over, just under) — covers OPP-02
- [ ] Unit tests for `is_stale()` covering 6-day (not stale) and 7-day (stale) cases — covers OPP-03
- [ ] Unit tests for role distribution computation from `champion_with_role` data — covers OPP-04
- [ ] The existing `audit-misc-pages.spec.ts` opponents test only checks page load; consider expanding to verify h1 + at least one role label visible after redesign

---

## Sources

### Primary (HIGH confidence)
- `/home/jasper/Repositories/lol_team_companion/src/pages/opponents.rs` — full current implementation read
- `/home/jasper/Repositories/lol_team_companion/src/models/opponent.rs` — model structs
- `/home/jasper/Repositories/lol_team_companion/src/server/riot.rs` — all Riot API fns
- `/home/jasper/Repositories/lol_team_companion/src/server/db.rs` lines 2222-2438 — all opponent DB fns
- `/home/jasper/Repositories/lol_team_companion/schema.surql` lines 173-187 — opponent schema
- `/home/jasper/Repositories/lol_team_companion/src/models/utils.rs` — `format_timestamp` signature
- `~/.cargo/registry/src/.../riven-2.78.0/src/models.rs` — confirmed `team_position: String` field on match participant
- `CLAUDE.md` — rules 21, 25, 30, 31, 33, 35, 44 applied directly

### Secondary (MEDIUM confidence)
- Phase 9 CONTEXT.md D-05 — confirmed `Draft.opponent` stores opponent ID; `recent_champions` is consumed by draft intel, cannot change type

---

## Project Constraints (from CLAUDE.md)

All of the following CLAUDE.md rules apply directly to this phase:

- **Rule 1:** `type::record('table', $key)` not `type::thing()` in all SurrealDB queries
- **Rule 2:** Strip table prefix before passing keys (e.g., `player_id.strip_prefix("opponent_player:")`)
- **Rule 3:** `DbOpponentPlayer` struct with `RecordId` fields; convert to `OpponentPlayer` with `String` IDs
- **Rule 4:** `.bind()` requires owned `String` values
- **Rule 5:** No `string()` cast in SurQL
- **Rule 9:** `use` statements for server-only crates inside `#[server]` fn body
- **Rule 21:** `prop:value` for controlled inputs (all 5 role input fields)
- **Rule 24:** `spawn_local` for async event handlers (fetch buttons)
- **Rule 25:** `collect_view()` for iterators in `view!` macro
- **Rule 27:** `.check()` on write queries (CREATE/UPDATE for new player slots)
- **Rule 28:** `.take(0).unwrap_or_default()` for list queries
- **Rule 29:** Batch multiple queries (opponent + 5 player CREATEs in one `.query()` call)
- **Rule 30:** `DEFINE FIELD IF NOT EXISTS` for all new schema fields
- **Rule 31:** `BEGIN`/`COMMIT` for the atomic opponent + 5-player creation
- **Rule 33:** Complex nested data (role_distribution, mastery_data) as JSON strings
- **Rule 35:** No `.unwrap()` in event handlers or WASM code
- **Rule 44:** Return `Ok(Vec::new())` not `Err` when optional resource is absent (players with no Riot ID)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reading existing code, no external research needed
- Architecture patterns: HIGH — derived from existing patterns in db.rs, riot.rs, and CLAUDE.md rules
- Pitfalls: HIGH — identified from reading current code and CLAUDE.md project-specific gotchas
- Riot API team_position field: HIGH — verified directly in riven-2.78.0 source in cargo registry

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable domain — riven, Leptos, SurrealDB versions are pinned in Cargo.lock)
